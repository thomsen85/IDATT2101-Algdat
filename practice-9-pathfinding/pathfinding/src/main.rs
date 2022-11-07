use std::cmp::{Ord, Ordering};
use std::collections::{BinaryHeap, HashMap};
use std::fs::File;
use std::io;
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
    points_of_interest: HashMap<u32, (u8, String)> 

}

impl Map {
    fn new() -> Self {
        Self { nodes: Vec::new(), edges: Vec::new(), points_of_interest: HashMap::new()}
    }

    fn from_nodes_edges_and_poi(nodes: Vec<Node>, edges: Vec<Vec<EdgeTo>>, points_of_interest: HashMap<u32, (u8, String)> ) -> Self {
        Self { nodes, edges, points_of_interest}
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Priority<'a> {
    number: usize,
    edges: &'a Vec<EdgeTo>,
    cost: usize,
}

impl<'a> Priority<'a> {
    fn new(number: usize, cost: usize, edges: &'a Vec<EdgeTo>) -> Self {
        Self {
            number,
            edges,
            cost,
        }
    }
}

impl Ord for Priority<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cost.cmp(&other.cost).reverse()
    }
}

impl PartialOrd for Priority<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}


fn djikstra(map: &Map, source: usize) -> (Vec<usize>, Vec<Option<usize>>) {
    // Init variables
    let length = map.edges.len();
    let mut shortest_distances = vec![usize::MAX / 2; length];
    let mut previous: Vec<Option<usize>> = vec![None; length];
    let mut priority_queue: BinaryHeap<Priority> = BinaryHeap::new();
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
    for p in travel_path {
        file.write(format!("{},{}\n", p.0, p.1).as_bytes())?;
    }

    Ok(())

}

fn main() {
    println!("Loading Files ...");
    let map = get_map_from_paths("island_noder.txt", "island_kanter.txt", "island_interessepkt.txt").expect("Could Not load map");
    println!("Done loading files.");

    // (100591, (32, "Tower Suites Reykjavík")) 
    // (99741, (32, "The Potato Storage"))

    let (res_dist, res_prev) = djikstra(&map, 100591);
    let shortest_path = res_dist[99741];
    let mut path: Vec<(f64, f64)> = Vec::new();
    
    println!("Djikstra done, shortest path = {:?}", shortest_path);

    let mut prev = 99741;
    loop {
        if let Some(p) = res_prev[prev] {
            prev = p
        } else {
            break
        }
        if prev == 99741 {
            break;
        }
        path.push((map.nodes[prev].latitude, map.nodes[prev].longitude));
    }

    println!("Writing to csv");
    
    let _ = travel_path_to_csv(path, "path.csv");

    loop {
        let mut input_string = String::new();
        print!("Search: ");
        let _ = std::io::stdout().flush();
        std::io::stdin().read_line(&mut input_string)
            .ok()
            .expect("Failed to read line");

        if input_string == "q" {
            break
        }

        if let Ok(num) = input_string.trim().parse::<u8>() {
            println!("Searching for {} in list", num);
            let res: Vec<(&u32, &(u8, String))> = map.points_of_interest.iter()
                .filter(|(_, v)| v.0 == num)
                .collect();

            println!("{:?}", res);
        }
    }
}