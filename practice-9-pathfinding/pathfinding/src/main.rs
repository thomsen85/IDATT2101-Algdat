#![allow(dead_code)]
use std::cmp::{Ord, Ordering};
use std::collections::{BinaryHeap, HashMap};
use std::fs::File;
use std::{io, result};
use std::io::{BufRead, BufReader, Write};
use std::string::ParseError;
use std::time::Instant;
use std::env;


#[derive(Debug, Clone, PartialEq, Eq)]
struct EdgeTo {
    to: u32,
    drive_time: u32,
    length: u32,
    speed_limit: u16,
}

impl EdgeTo {
    fn new(to: u32, drive_time: u32, length: u32, speed_limit: u16) -> Self {
        Self { to, drive_time, length, speed_limit }
    }
}

struct Node {
    id: u32,
    latitude: f64,
    longitude: f64,
} 

impl Node {
    fn new(id: u32, latitude: f64, longitude: f64) -> Self {
        Self { id, latitude, longitude }
    }
}

struct Map {
    nodes: Vec<Node>,
    edges: Vec<Vec<EdgeTo>>,
    points_of_interest: HashMap<u32, (u8, String)>,

}

impl Map {
    fn new() -> Self {
        Self { nodes: Vec::new(), edges: Vec::new(), points_of_interest: HashMap::new()}
    }

    fn from_nodes_edges_and_poi(nodes: Vec<Node>, edges: Vec<Vec<EdgeTo>>, points_of_interest: HashMap<u32, (u8, String)> ) -> Self {
        Self { nodes, edges, points_of_interest}
    }

    fn get_distance_between_nodes_in_meters(&self, from: usize, to: usize) -> usize {
        let a = self.get_coordinates_from_node(from);
        let b = self.get_coordinates_from_node(to);

        (((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt() * 111_139.0) as usize
    }

    fn get_coordinates_from_node(&self, node: usize) -> (f64, f64) {
        let n = &self.nodes[node];
        (n.latitude, n.longitude)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Priority<'a, T>{
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

fn a_star(map: &Map, from: usize, to: usize) -> (usize, Vec<u32>, Vec<u32>) {
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
                let cost = map.get_distance_between_nodes_in_meters(neighbour.to as usize, to) + alt / 2;
                priority_queue.push(Priority::new(neighbour.to as usize, cost, &map.edges[neighbour.to as usize]));
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
        } if prev == from {
            break
        }
    }
    path.reverse();
    
    (shortest_distances[to], path, visited)
}

fn closest_dijkstra(map: &Map, from: usize, to: usize) -> (usize, Vec<u32>) {
    // Init variables
    let length = map.edges.len();
    let mut shortest_distances = vec![usize::MAX / 2; length];
    let mut previous: Vec<Option<usize>> = vec![None; length];
    let mut priority_queue: BinaryHeap<Priority<usize>> = BinaryHeap::new();
    shortest_distances[from] = 0;

    // Push source variable
    priority_queue.push(Priority::new(from, 0, &map.edges[from]));

    while let Some(priority) = priority_queue.pop() {
        if priority.number == to {
            break; 
        }
        for neighbour in priority.edges {
            let alt = shortest_distances[priority.number] + neighbour.drive_time as usize;
            if alt < shortest_distances[neighbour.to as usize] {
                shortest_distances[neighbour.to as usize] = alt;
                previous[neighbour.to as usize] = Some(priority.number);
                priority_queue.push(Priority::new(neighbour.to as usize, alt, &map.edges[neighbour.to as usize]));
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
            break
        }
    }
    path.reverse();
    
    (shortest_distances[to], path)
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

    while let Some(priority) = priority_queue.pop(){
        if let Some(poi) =  map.points_of_interest.get(&(priority.number as u32)) {
            if poi.0 & category == category {
                results.push(priority.number as u32)
            }
        }
        if results.len() >= amount as usize {
            break
        }
        for neighbour in priority.edges {
            let alt = shortest_distances[priority.number] + neighbour.drive_time as usize;
            if alt < shortest_distances[neighbour.to as usize] {
                shortest_distances[neighbour.to as usize] = alt;
                previous[neighbour.to as usize] = Some(priority.number);
                priority_queue.push(Priority::new(neighbour.to as usize, alt, &map.edges[neighbour.to as usize]));
            }
        }
    }
    results
}

fn full_dijkstra(map: &Map, source: usize) -> (Vec<usize>, Vec<Option<usize>>) {
    // Init variables
    let length = map.edges.len();
    let mut shortest_distances = vec![usize::MAX / 2; length];
    let mut previous: Vec<Option<usize>> = vec![None; length];
    let mut priority_queue: BinaryHeap<Priority<usize>> = BinaryHeap::new();
    shortest_distances[source] = 0;

    // Push source variable
    priority_queue.push(Priority::new(source, 0, &map.edges[source]));

    while let Some(priority) = priority_queue.pop() {
        for neighbour in priority.edges {
            let alt = shortest_distances[priority.number] + neighbour.drive_time as usize;
            if alt < shortest_distances[neighbour.to as usize] {
                shortest_distances[neighbour.to as usize] = alt;
                previous[neighbour.to as usize] = Some(priority.number);
                priority_queue.push(Priority::new(neighbour.to as usize, alt, &map.edges[neighbour.to as usize]));
            }
        }
    }
    (shortest_distances, previous)
}

fn node_from_string(l: Vec<&str>) -> Node {
    Node::new(l[0].parse().expect("Could not parse to node"), 
        l[1].parse().expect("Could not parse to node"), 
        l[2].parse().expect("Could not parse to node"))
}

fn edge_from_string(l: Vec<&str>) -> EdgeTo {
    EdgeTo::new(l[1].parse().expect("Could not parse to node"), 
    l[2].parse().expect("Could not parse to node"), 
    l[3].parse().expect("Could not parse to node"), 
    l[4].parse().expect("Could not parse to node"))
}

fn get_map_from_paths(node_path: &str, edge_path: &str, poi_path: &str) -> io::Result<Map> {
    let node_file = File::open(node_path)?;
    let edge_file = File::open(edge_path)?;
    let poi_file = File::open(poi_path)?;

    let mut node_reader = BufReader::new(node_file);
    let mut node_first_line = String::new();
    node_reader.read_line(&mut node_first_line)?;
    let node_lines: usize = node_first_line.trim().parse().expect("Could not parse First line");
    let nodes: Vec<Node> = node_reader
        .lines()
        .into_iter()
        .map(|line| node_from_string(line.expect("Failed to read line").split_whitespace().collect()))
        .collect();
    
    assert_eq!(nodes.len(), node_lines);
    
    let mut edge_reader = BufReader::new(edge_file);
    let mut edge_first_line = String::new();
    edge_reader.read_line(&mut edge_first_line)?;
    let edges_len : usize = edge_first_line.trim().parse().expect("Could not parse First line");
    let mut edges = vec![Vec::new(); edges_len];

    for line in edge_reader.lines() {
        let line= line.expect("Could not parse line");
        let l: Vec<&str> = line.split_whitespace().collect();
        edges[l[0].parse::<usize>().unwrap()].push(edge_from_string(l));
    }

    assert_eq!(nodes.len(), node_lines);

    let mut poi_reader = BufReader::new(poi_file);
    let mut poi_first_line = String::new();
    poi_reader.read_line(&mut poi_first_line)?;
    let poi_len : usize = poi_first_line.trim().parse().expect("Could not parse First line");

    let mut poi: HashMap<u32, (u8, String)> = HashMap::with_capacity(poi_len);
    for line in poi_reader.lines() {
        let line= line.expect("Could not parse line");
        let l: Vec<&str> = line.split("\t").collect();
        poi.insert(l[0].parse().expect("Could not parse line"), 
        (l[1].parse().expect("Could not parse line"),
        l[2][1..(l[2].len()-1)].to_string()));
    }
    Ok(Map::from_nodes_edges_and_poi(nodes, edges, poi))
}


fn travel_path_to_csv(travel_path: Vec<(f64, f64)>, file_path: &str) -> io::Result<()>{
    let mut file = File::create(file_path)?;
    file.write("Latitude,Longitude\n".as_bytes())?;
    for p in travel_path {
        file.write(format!("{},{}\n", p.0, p.1).as_bytes())?;
    }

    Ok(())
}

fn centi_seconds_to_time_format(centi_seconds: usize) -> String {
    let seconds = centi_seconds / 100;

    let mut str = format!("{} second{}.", seconds % 60, if seconds != 1 {"s"} else {""}).to_string();
    if seconds < 60 {
        return str;
    }

    let minutes = seconds / 60;
    str = format!("{} minute{} and ", minutes % 60, if minutes != 1 {"s"} else {""}).to_string() + &str;
    if minutes < 60 {
        return str;
    }

    let hours = minutes / 60;
    str = format!("{} hour{}, ", hours % 24, if hours != 1 {"s"} else {""}).to_string() + &str;
    if hours < 24 {
        return str;
    }

    let days = hours / 24;
    str = format!("{} day{}, ", days, if days != 1 {"s"} else {""}).to_string() + &str;

    str

}

fn test_djikstra_shortest_path(map: Map) {
    // (100591, (32, "Tower Suites Reykjavík")) 
    // (99741, (32, "The Potato Storage"))
    let (shortest_time, path) = closest_dijkstra(&map, 100591, 99741);
    println!("Djikstra done, shortest path = {:?}", centi_seconds_to_time_format(shortest_time));
    println!("Writing to csv");
    let coordinates = path.iter()
        .map(|p| {
            let p = &map.nodes[p.clone() as usize];
            (p.latitude, p.longitude)
        })
        .collect();
    let _ = travel_path_to_csv(coordinates, "path.csv");
}

fn test_category_find(map: &Map) {
    // Hemsedal 3509663
    // 1 Stedsnavn
    // 2 Bensinstasjon
    // 4 Ladestasjon 
    // 8 Spisested
    // 16 Drikkested
    // 32 Overnattingssted 

    let result = category_based_dijkstra(map, 3509663, 1, 8);
    let refined_result: Vec<String> = result.iter()
        .map(|id| map.points_of_interest.get(id).unwrap().1.clone())
        .collect();
    println!("Finding 8 closest foodplaces to hemsedal: \n{:?}", refined_result);
}

fn test_a_star(map: &Map) {
    // Hemsedal 3509663
    //2826143	1	"Mjølkeråen"
    //2977813	1	"Gol"
    //4467988	1	"Hønefoss"
    //2587604	1	"Stjørdal"
    //2507642	1	"Steinkjer"



    let (distance, path, visited) = a_star(map, 3509663, 2826143); 
    travel_path_to_csv(visited.iter().map(|l| map.get_coordinates_from_node(l.clone() as usize)).collect(), "visited.csv").expect("Coud not write to visited");
    travel_path_to_csv(path.iter().map(|l| map.get_coordinates_from_node(l.clone() as usize)).collect(), "path.csv").expect("Coud not write to visited");
    println!("Time: {}", centi_seconds_to_time_format(distance));

}

fn main() {
    println!("Loading Files ...");
    //let map = get_map_from_paths("island_noder.txt", "island_kanter.txt", "island_interessepkt.txt").expect("Could Not load map");
    let map = get_map_from_paths("norden_noder.txt", "norden_kanter.txt", "norden_interessepkt.txt").expect("Could Not load map");
    println!("Done loading files.");

    
    //test_category_find(&map);
    test_a_star(&map); 

    // loop {
    //     let mut input_string = String::new();
    //     print!("Search: ");
    //     let _ = std::io::stdout().flush();
    //     std::io::stdin().read_line(&mut input_string)
    //         .ok()
    //         .expect("Failed to read line");

    //     if input_string == "q" {
    //         break
    //     }

    //     if let Ok(num) = input_string.trim().parse::<u8>() {
    //         println!("Searching for {} in list", num);
    //         let res: Vec<(&u32, &(u8, String))> = map.points_of_interest.iter()
    //             .filter(|(_, v)| v.0 == num)
    //             .collect();

    //         println!("{:?}", res);
    //     }
    // }
}

