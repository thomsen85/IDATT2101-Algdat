use std::cmp::{Ord, Ordering};
use std::collections::{BinaryHeap, HashMap};
use std::fs::File;
use std::{io, thread};
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::thread::Thread;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq)]
struct EdgeTo {
    to: u32,
    drive_time: u32,
    length: u32,
    speed_limit: u16,
}

impl EdgeTo {
    fn new(to: u32, drive_time: u32, length: u32, speed_limit: u16) -> Self {
        Self {
            to,
            drive_time,
            length,
            speed_limit,
        }
    }
}

#[derive(Clone)]
struct Node {
    id: u32,
    latitude: f64,
    longitude: f64,
}

impl Node {
    fn new(id: u32, latitude: f64, longitude: f64) -> Self {
        Self {
            id,
            latitude,
            longitude,
        }
    }
}

struct Waypoint {
    source: u32,
    distances_to: Vec<u32>,
    distances_from: Vec<u32>,
}

impl Waypoint {
    fn new(source: u32, distances_to: Vec<u32>, distances_from: Vec<u32>) -> Self {
        Self {
            source,
            distances_to,
            distances_from,
        }
    }
}

#[derive(Clone)]
struct Map {
    nodes: Vec<Node>,
    edges: Vec<Vec<EdgeTo>>,
    points_of_interest: HashMap<u32, (u8, String)>,
}

impl Map {
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            points_of_interest: HashMap::new(),
        }
    }

    fn from_nodes_edges_and_poi(
        nodes: Vec<Node>,
        edges: Vec<Vec<EdgeTo>>,
        points_of_interest: HashMap<u32, (u8, String)>,
    ) -> Self {
        Self {
            nodes,
            edges,
            points_of_interest,
        }
    }

    fn get_coordinates_from_node(&self, node: usize) -> (f64, f64) {
        let n = &self.nodes[node];
        (n.latitude, n.longitude)
    }

    fn get_reverse_copy(&self) -> Self {
        let mut edges = vec![Vec::new(); self.edges.len()];
        for i in 0..self.edges.len() {
            for edge in &self.edges[i] {
                edges[edge.to as usize].push(EdgeTo::new(
                    i as u32,
                    edge.drive_time,
                    edge.length,
                    edge.speed_limit,
                ));
            }
        }
        Map::from_nodes_edges_and_poi(self.nodes.clone(), edges, self.points_of_interest.clone())
    }

    fn get_name(&self, node_id: u32) -> String{
        self.points_of_interest
        .get(&node_id)
        .get_or_insert(&(0_u8, "Custom Waypoint".to_owned()))
        .1.to_owned()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Priority<'a, T> {
    number: usize,
    edges: &'a Vec<EdgeTo>,
    cost: T,
}

impl<'a, T> Priority<'a, T> {
    fn new(number: usize, cost: T, edges: &'a Vec<EdgeTo>) -> Self {
        Self {
            number,
            edges,
            cost,
        }
    }
}

impl<T: Ord> Ord for Priority<'_, T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cost.cmp(&other.cost).reverse()
    }
}

impl<T: Ord> PartialOrd for Priority<'_, T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn get_waypoint_cost(source: usize, goal: usize, waypoints: &Vec<Waypoint>) -> u32 {
    let mut diffs = Vec::new();
    for waypoint in waypoints {
        let diff1: u32 = {
            if waypoint.distances_to[source] > waypoint.distances_to[goal] {
                0
            } else {
                waypoint.distances_to[goal] - waypoint.distances_to[source]
            }
        };
        diffs.push(diff1);

        let diff2: u32 = {
            if waypoint.distances_from[goal] > waypoint.distances_from[source] {
                0
            } else {
                waypoint.distances_from[source] - waypoint.distances_from[goal]
            }
        };
        diffs.push(diff2);
    }
    diffs.into_iter().max().expect("No waypoints found")
}

fn alt(
    map: &Map,
    waypoints: &Vec<Waypoint>,
    source: usize,
    goal: usize,
) -> (usize, Vec<u32>, Vec<u32>) {
    // Init variables
    let length = map.edges.len();
    let mut shortest_distances = vec![usize::MAX / 2; length];
    let mut previous: Vec<Option<usize>> = vec![None; length];
    let mut priority_queue: BinaryHeap<Priority<usize>> = BinaryHeap::new();
    shortest_distances[source] = 0;

    // Push source variable
    priority_queue.push(Priority::new(source, 0, &map.edges[source]));
    let mut visited = Vec::new();
    while let Some(priority) = priority_queue.pop() {
        visited.push(priority.number as u32);
        if priority.number == goal {
            break;
        }
        for neighbour in priority.edges {
            let alt = shortest_distances[priority.number] + neighbour.drive_time as usize;
            if alt < shortest_distances[neighbour.to as usize] {
                shortest_distances[neighbour.to as usize] = alt;
                previous[neighbour.to as usize] = Some(priority.number);
                let cost = get_waypoint_cost(neighbour.to as usize, goal, waypoints) as usize + alt;
                priority_queue.push(Priority::new(
                    neighbour.to as usize,
                    cost,
                    &map.edges[neighbour.to as usize],
                ));
            }
            // println!("\n\n###################\n{:?}", priority_queue.iter()
            // .map(|l|  (map.get_coordinates_from_node(l.number), centi_seconds_to_time_format(l.cost)))
            // .collect::<Vec<((f64, f64), String)>>());
        }
    }

    let mut path = Vec::new();
    let mut prev = goal;
    loop {
        path.push(prev as u32);
        if let Some(p) = previous[prev] {
            prev = p;
        } else {
            panic!("Something went wrong with finding path");
        }
        if prev == source {
            break;
        }
    }
    path.reverse();

    (shortest_distances[goal], path, visited)
}

fn closest_dijkstra(map: &Map, from: usize, to: usize) -> (usize, Vec<u32>, Vec<u32>) {
    // Init variables
    let length = map.edges.len();
    let mut shortest_distances = vec![usize::MAX / 2; length];
    let mut previous: Vec<Option<usize>> = vec![None; length];
    let mut priority_queue: BinaryHeap<Priority<usize>> = BinaryHeap::new();
    shortest_distances[from] = 0;

    // Push source variable
    priority_queue.push(Priority::new(from, 0, &map.edges[from]));
    let mut visited = Vec::new();
    while let Some(priority) = priority_queue.pop() {
        visited.push(priority.number as u32);
        if priority.number == to {
            break;
        }
        for neighbour in priority.edges {
            let alt = shortest_distances[priority.number] + neighbour.drive_time as usize;
            if alt < shortest_distances[neighbour.to as usize] {
                shortest_distances[neighbour.to as usize] = alt;
                previous[neighbour.to as usize] = Some(priority.number);
                priority_queue.push(Priority::new(
                    neighbour.to as usize,
                    alt,
                    &map.edges[neighbour.to as usize],
                ));
            }
        }
    }

    let mut path = Vec::new();
    let mut prev = to;
    loop {
        path.push(prev as u32);
        if let Some(p) = previous[prev] {
            prev = p;
        } else {
            panic!("Something went wrong with finding path");
        }
        if prev == from {
            break;
        }
    }
    path.reverse();

    (shortest_distances[to], path, visited)
}

fn category_based_dijkstra(map: &Map, source: usize, category: u8, amount: u32) -> Vec<u32> {
    // Init variables
    let length = map.edges.len();
    let mut shortest_distances = vec![usize::MAX / 2; length];
    let mut previous: Vec<Option<usize>> = vec![None; length];
    let mut priority_queue: BinaryHeap<Priority<usize>> = BinaryHeap::new();
    shortest_distances[source] = 0;
    let mut results = Vec::with_capacity(amount as usize);

    // Push source variable
    priority_queue.push(Priority::new(source, 0, &map.edges[source]));

    while let Some(priority) = priority_queue.pop() {
        if let Some(poi) = map.points_of_interest.get(&(priority.number as u32)) {
            if poi.0 & category == category && !results.contains(&(priority.number as u32)) {
                results.push(priority.number as u32)
            }
        }
        if results.len() >= amount as usize {
            break;
        }
        for neighbour in priority.edges {
            let alt = shortest_distances[priority.number] + neighbour.drive_time as usize;
            if alt < shortest_distances[neighbour.to as usize] {
                shortest_distances[neighbour.to as usize] = alt;
                previous[neighbour.to as usize] = Some(priority.number);
                priority_queue.push(Priority::new(
                    neighbour.to as usize,
                    alt,
                    &map.edges[neighbour.to as usize],
                ));
            }
        }
    }
    results
}

/// Returns: (shotests distances, previous)
fn full_dijkstra(map: &Map, source: u32) -> (Vec<u32>, Vec<Option<usize>>) {
    // Init variables
    let length = map.edges.len();
    let mut shortest_distances = vec![u32::MAX / 2; length];
    let mut previous: Vec<Option<usize>> = vec![None; length];
    let mut priority_queue: BinaryHeap<Priority<u32>> = BinaryHeap::new();
    shortest_distances[source as usize] = 0;

    // Push source variable
    priority_queue.push(Priority::new(
        source as usize,
        0,
        &map.edges[source as usize],
    ));

    while let Some(priority) = priority_queue.pop() {
        for neighbour in priority.edges {
            let alt = shortest_distances[priority.number] + neighbour.drive_time;
            if alt < shortest_distances[neighbour.to as usize] {
                shortest_distances[neighbour.to as usize] = alt;
                previous[neighbour.to as usize] = Some(priority.number);
                priority_queue.push(Priority::new(
                    neighbour.to as usize,
                    alt,
                    &map.edges[neighbour.to as usize],
                ));
            }
        }
    }
    (shortest_distances, previous)
}

fn node_from_string(l: Vec<&str>) -> Node {
    Node::new(
        l[0].parse().expect("Could not parse to node"),
        l[1].parse().expect("Could not parse to node"),
        l[2].parse().expect("Could not parse to node"),
    )
}

fn edge_from_string(l: Vec<&str>) -> EdgeTo {
    EdgeTo::new(
        l[1].parse().expect("Could not parse to node"),
        l[2].parse().expect("Could not parse to node"),
        l[3].parse().expect("Could not parse to node"),
        l[4].parse().expect("Could not parse to node"),
    )
}

fn get_map_from_paths(node_path: &str, edge_path: &str, poi_path: &str) -> io::Result<Map> {
    let node_file = File::open(node_path)?;
    let edge_file = File::open(edge_path)?;
    let poi_file = File::open(poi_path)?;

    let mut node_reader = BufReader::new(node_file);
    let mut node_first_line = String::new();
    node_reader.read_line(&mut node_first_line)?;
    let node_lines: usize = node_first_line
        .trim()
        .parse()
        .expect("Could not parse First line");
    let nodes: Vec<Node> = node_reader
        .lines()
        .into_iter()
        .map(|line| {
            node_from_string(
                line.expect("Failed to read line")
                    .split_whitespace()
                    .collect(),
            )
        })
        .collect();

    assert_eq!(nodes.len(), node_lines);

    let mut edge_reader = BufReader::new(edge_file);
    let mut edge_first_line = String::new();
    edge_reader.read_line(&mut edge_first_line)?;
    let edges_len: usize = edge_first_line
        .trim()
        .parse()
        .expect("Could not parse First line");
    let mut edges = vec![Vec::new(); edges_len];

    for line in edge_reader.lines() {
        let line = line.expect("Could not parse line");
        let l: Vec<&str> = line.split_whitespace().collect();
        edges[l[0].parse::<usize>().unwrap()].push(edge_from_string(l));
    }

    assert_eq!(nodes.len(), node_lines);

    let mut poi_reader = BufReader::new(poi_file);
    let mut poi_first_line = String::new();
    poi_reader.read_line(&mut poi_first_line)?;
    let poi_len: usize = poi_first_line
        .trim()
        .parse()
        .expect("Could not parse First line");

    let mut poi: HashMap<u32, (u8, String)> = HashMap::with_capacity(poi_len);
    for line in poi_reader.lines() {
        let line = line.expect("Could not parse line");
        let l: Vec<&str> = line.split("\t").collect();
        poi.insert(
            l[0].parse().expect("Could not parse line"),
            (
                l[1].parse().expect("Could not parse line"),
                l[2][1..(l[2].len() - 1)].to_string(),
            ),
        );
    }
    Ok(Map::from_nodes_edges_and_poi(nodes, edges, poi))
}

fn travel_path_to_csv(travel_path: Vec<(f64, f64)>, file_path: &str) -> io::Result<()> {
    let mut file = File::create(file_path)?;
    file.write("Latitude,Longitude\n".as_bytes())?;
    for p in travel_path {
        file.write(format!("{},{}\n", p.0, p.1).as_bytes())?;
    }
    Ok(())
}

fn centi_seconds_to_time_format(centi_seconds: usize) -> String {
    let seconds = centi_seconds / 100;

    let mut str = format!(
        "{} second{}.",
        seconds % 60,
        if seconds != 1 { "s" } else { "" }
    );

    if seconds < 60 {
        return str;
    }

    let minutes = seconds / 60;
    str = format!(
        "{} minute{} and ",
        minutes % 60,
        if minutes != 1 { "s" } else { "" }
    )
        + &str;
    if minutes < 60 {
        return str;
    }

    let hours = minutes / 60;
    str = format!("{} hour{}, ", hours % 24, if hours != 1 { "s" } else { "" }) + &str;
    if hours < 24 {
        return str;
    }

    let days = hours / 24;
    str = format!("{} day{}, ", days, if days != 1 { "s" } else { "" }) + &str;

    str
}

fn get_file_as_bytes(path: &str) -> io::Result<Vec<u8>> {
    let mut buffer = Vec::new();
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    reader.read_to_end(&mut buffer)?;
    Ok(buffer)
}

fn write_file_as_bytes(path: &str, bytes: &[u8]) -> io::Result<usize> {
    let file = File::create(path).expect("File could not be created");
    let mut writer = BufWriter::new(file);
    writer.write(bytes)
}

fn get_u32_from_byte_array(bytes: &[u8]) -> u32 {
    assert_eq!(bytes.len(), 4);
    let mut num = 0;

    num |= bytes[3] as u32;
    num |= (bytes[2] as u32) << 8;
    num |= (bytes[1] as u32) << 16;
    num |= (bytes[0] as u32) << 24;

    num
}

fn get_byte_array_from_u32(input: u32) -> [u8; 4] {
    let b1: u8 = ((input >> 24) & 0xff) as u8;
    let b2: u8 = ((input >> 16) & 0xff) as u8;
    let b3: u8 = ((input >> 8) & 0xff) as u8;
    let b4: u8 = (input & 0xff) as u8;
    [b1, b2, b3, b4]
}

fn create_waypoints(map: &Map, sources: &Vec<u32>) {
    let mut handels = Vec::new();
    for source in sources {
        let map_copy = map.clone();
        let source_copy = source.clone();
        handels.push(thread::spawn(move || {
            let mut bytes: Vec<u8> = Vec::new();
            println!("Calculating for source: {}", source_copy);
            bytes.extend(get_byte_array_from_u32(source_copy.to_owned()));
    
            // To
            let (dijk_distances_to, _) = full_dijkstra(&map_copy, source_copy.to_owned());
            bytes.extend(get_byte_array_from_u32(dijk_distances_to.len() as u32));
    
            for dist in dijk_distances_to {
                bytes.extend(get_byte_array_from_u32(dist as u32))
            }
    
            // From
            let (dijk_distances_from, _) = full_dijkstra(&map_copy.get_reverse_copy(), source_copy.to_owned());
            bytes.extend(get_byte_array_from_u32(dijk_distances_from.len() as u32));
    
            for dist in dijk_distances_from {
                bytes.extend(get_byte_array_from_u32(dist as u32))
            }
            println!("Done creating waypoints");

            if write_file_as_bytes("waypoints.bin", &bytes).is_ok() {
                println!("Waypoints succsessfully written to file \"waypoints.bin\"");
            } else {
                println!("Waypoints could not be written to file \"waypoints.bin\"");
            }
        }));
    }

    for handle in handels {
        handle.join().expect("Couldn't join on the associated thread");
    }

}

fn get_waypoints_from_bytes(bytes: &Vec<u8>) -> Vec<Waypoint> {
    let mut pointer: usize = 0;
    let mut res: Vec<Waypoint> = Vec::new();
    loop {
        if pointer >= bytes.len() {
            break;
        }
        let mut distances_to = Vec::new();
        let mut distances_from = Vec::new();

        let source = get_u32_from_byte_array(&bytes[pointer..pointer + 4]);
        pointer += 4;
        let length_to = get_u32_from_byte_array(&bytes[pointer..pointer + 4]);
        pointer += 4;

        for _ in 0..length_to {
            distances_to.push(get_u32_from_byte_array(&bytes[pointer..pointer + 4]));
            pointer += 4
        }
        let length_from = get_u32_from_byte_array(&bytes[pointer..pointer + 4]);

        pointer += 4;
        for _ in 0..length_from {
            distances_from.push(get_u32_from_byte_array(&bytes[pointer..pointer + 4]));
            pointer += 4
        }

        res.push(Waypoint::new(source, distances_to, distances_from));
    }
    res
}

fn get_waypoints(map: &Map) -> Vec<Waypoint> {
    // 5697698	8	"Bergen Pizza"
    // 7283163	8	"Pizzabakeren Alta"
    // 1906903	8	"Kungsan Pizzeria"
    // 493001	8	"Pizzeria Roihu"

    let path = "waypoints.bin";
    let sources = vec![5697698, 7283163, 1906903, 493001];
    if let Ok(bytes) = get_file_as_bytes(path) {
        return get_waypoints_from_bytes(&bytes);
    } else {
        create_waypoints(map, &sources)
    }

    if let Ok(bytes) = get_file_as_bytes(path) {
        get_waypoints_from_bytes(&bytes)
    } else {
        panic!("File could not be read after creation");
    }
}

fn compare_alt_and_dijkstras(map: &Map, waypoints: &Vec<Waypoint>, from: u32, to: u32) {
    let from_name = map.get_name(from);
    let to_name = map.get_name(to);

    println!("\nTesting Dijkstras: From {}, To {}", from_name, to_name);
    let timer_dijkstras = Instant::now();
    let (time_distance, path, visited) = closest_dijkstra(map, from as usize, to as usize);
    let time_taken = timer_dijkstras.elapsed().as_millis();
    println!(
        "Djikstras took {} seconds, and visited {} nodes. Estimated travel time is: {}",
        time_taken as f64 / 1000.0,
        format_number(visited.len() as isize),
        centi_seconds_to_time_format(time_distance)
    );
    travel_path_to_csv(
        path.into_iter()
            .map(|n| map.get_coordinates_from_node(n as usize))
            .collect(),
        &format!("djikstra_path_{}_{}.csv", from_name, to_name),
    )
    .expect("Could not write result to file");

    println!("\nTesting ALT: From {}, To {}", from_name, to_name);
    let timer_alt = Instant::now();
    let (time_distance, path, visited) = alt(map, waypoints, from as usize, to as usize);
    let time_taken = timer_alt.elapsed().as_millis();
    println!(
        "Alt took {} seconds, and visited {} nodes. Estimated travel time is: {}",
        time_taken as f64 / 1000.0,
        format_number(visited.len() as isize),
        centi_seconds_to_time_format(time_distance)
    );
    travel_path_to_csv(
        path.into_iter()
            .map(|n| map.get_coordinates_from_node(n as usize))
            .collect(),
        &format!("alt_path_{}_{}.csv", from_name, to_name),
    )
    .expect("Could not write result to file");
}

fn format_number(number: isize) -> String {
    let mut str = Vec::new();
    let mut copy = Clone::clone(&number);

    while copy > 1000 {
        str.push((copy % 1000).to_string());
        copy /= 1000;
    }
    str.push((copy % 1000).to_string());

    str.into_iter().rev().collect::<Vec<String>>().join(" ")
}

fn find_closest_information(map: &Map) {
    const TRONDHEIM_LUFTHAVN: usize = 7172108;
    const TRONDHEIM_TORG: usize = 4546048;
    const HEMSEDAL: usize = 3509663;
    const AMOUNT_OF_RESULTS: u32 = 8;

    const CHARGING_STATION: u8 = 4;
    const PLACE_TO_EAT: u8 = 8;
    const PLACE_TO_DRINK: u8 = 16;

    println!(
        "\nFinding {} closest charging stations near Trondheim lufthavn:",
        AMOUNT_OF_RESULTS
    );
    let results =
        category_based_dijkstra(map, TRONDHEIM_LUFTHAVN, CHARGING_STATION, AMOUNT_OF_RESULTS);
    for result in &results {
        println!(
            "{} - {:?}",
            map.points_of_interest[&result].1,
            map.get_coordinates_from_node(result.to_owned() as usize)
        );
    }
    travel_path_to_csv(
        results.into_iter()
            .map(|n| map.get_coordinates_from_node(n as usize))
            .collect(),
        &format!("{}_closest_charging_to_{}.csv", AMOUNT_OF_RESULTS, map.get_name(TRONDHEIM_LUFTHAVN as u32)),
    )
    .expect("Could not write result to file");

    println!(
        "\nFinding {} closest places to drink near Trondheim torg:",
        AMOUNT_OF_RESULTS
    );
    let results = category_based_dijkstra(map, TRONDHEIM_TORG, PLACE_TO_DRINK, AMOUNT_OF_RESULTS);
    for result in &results {
        println!(
            "{} - {:?}",
            map.points_of_interest[&result].1,
            map.get_coordinates_from_node(result.to_owned() as usize)
        );
    }
    travel_path_to_csv(
        results.into_iter()
            .map(|n| map.get_coordinates_from_node(n as usize))
            .collect(),
        &format!("{}_closest_drinking_to_{}.csv", AMOUNT_OF_RESULTS, map.get_name(TRONDHEIM_TORG as u32)),
    )
    .expect("Could not write result to file");


    println!(
        "\nFinding {} closest places to eat in Hemsedal:",
        AMOUNT_OF_RESULTS
    );
    let results = category_based_dijkstra(map, HEMSEDAL, PLACE_TO_EAT, AMOUNT_OF_RESULTS);
    for result in &results {
        println!(
            "{} - {:?}",
            map.points_of_interest[&result].1,
            map.get_coordinates_from_node(result.to_owned() as usize)
        );
    }
    travel_path_to_csv(
        results.into_iter()
            .map(|n| map.get_coordinates_from_node(n as usize))
            .collect(),
        &format!("{}_closest_eating_to_{}.csv", AMOUNT_OF_RESULTS, map.get_name(HEMSEDAL as u32)),
    )
    .expect("Could not write result to file");

}

fn main() {
    println!("Loading map ...");
    let map = get_map_from_paths(
        "norden_noder.txt",
        "norden_kanter.txt",
        "norden_interessepkt.txt",
    )
    .expect("Could Not load map");
    println!("Done loading map.");

    println!("Loading waypoints ...");
    let waypoint_timer = Instant::now();
    let waypoints = get_waypoints(&map);

    println!("Done loading waypoints.");

    const KÅRVÅG: u32 = 3292784;
    const GJEMNES: u32 = 7352330;

    const TAMPERE: u32 = 232073;
    const ÅLESUND: u32 = 2518780;


    compare_alt_and_dijkstras(&map, &waypoints, KÅRVÅG, GJEMNES);
    compare_alt_and_dijkstras(&map, &waypoints, TAMPERE, ÅLESUND);
    find_closest_information(&map);
}



