use std::cmp::{Ord, Ordering};
use std::collections::BinaryHeap;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::time::Instant;
use std::env;

#[derive(Debug)]
struct WGraph {
    // Neighbour list
    nodes: Vec<Vec<EdgeTo>>,
}

impl WGraph {
    fn with_nodes(amount_of_nodes: usize) -> Self {
        let mut nodes: Vec<Vec<EdgeTo>> = Vec::with_capacity(amount_of_nodes);
        for _ in 0..amount_of_nodes {
            nodes.push(Vec::new());
        }
        Self { nodes }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq)]
struct EdgeTo {
    to: usize,
    weight: usize,
}

impl EdgeTo {
    fn new(to: usize, weight: usize) -> Self {
        Self { to, weight }
    }
}

impl Ord for EdgeTo {
    fn cmp(&self, other: &Self) -> Ordering {
        self.weight.cmp(&other.weight).reverse()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Priority<'a> {
    number: usize,
    nodes: &'a Vec<EdgeTo>,
    cost: usize,
}

impl<'a> Priority<'a> {
    fn new(number: usize, cost: usize, nodes: &'a Vec<EdgeTo>) -> Self {
        Self {
            number,
            nodes,
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

fn main() {
    let args: Vec<String> = env::args().collect();
    let graph_path = &args[1];
    let source: usize = args[2].parse().expect("Second agrument must be a number");

    let graph = new_wgraph(graph_path)
        .expect(&format!("Could not load graph from file: {}", &graph_path));
    let length = graph.nodes.len();

    // Starting timer
    let time = Instant::now();
    // Running algoritm
    let (shortest_distances, previous) = djikstra(graph, source);
    let time_taken = time.elapsed();

    println!("Result For graph {}:", graph_path);
    if length < 100 {
        println!("+{0}+{0}+{0}+", "-".repeat(16));
        println!("|{: <15} | {: <15}| {: <15}|", "Node", "Prev", "Distance");
        println!("+{0}+{0}+{0}+", "-".repeat(16));
        for i in 0..length {
            if i == source {
                println!(
                    "|{: <15} | {: <15}| {: <15}|",
                    i, "Start", shortest_distances[i]
                )
            } else if let Some(prev) = previous[i] {
                println!(
                    "|{: <15} | {: <15}| {: <15}|",
                    i, prev, shortest_distances[i]
                )
            } else {
                println!(
                    "|{: <15} | {: <15}| {: <15}|",
                    i, "Not Reached", "Not Reached"
                )
            }
        }
        println!("+{0}+{0}+{0}+", "-".repeat(16));
    } else {
        println!("Took {} ms", time_taken.as_micros() as f64 / 1000.0);
    }

}

fn djikstra(graph: WGraph, source: usize) -> (Vec<usize>, Vec<Option<usize>>) {
    // Init variables
    let length = graph.nodes.len();
    let mut shortest_distances = vec![usize::MAX / 2; length];
    let mut previous: Vec<Option<usize>> = vec![None; length];
    let mut priority_queue: BinaryHeap<Priority> = BinaryHeap::new();
    shortest_distances[source] = 0;

    // Push source variable
    priority_queue.push(Priority::new(source, 0, &graph.nodes[source]));

    while let Some(priority) = priority_queue.pop() {
        for neighbour in priority.nodes {
            let alt = shortest_distances[priority.number] + neighbour.weight;
            if alt < shortest_distances[neighbour.to] {
                shortest_distances[neighbour.to] = alt;
                previous[neighbour.to] = Some(priority.number);
                priority_queue.push(Priority::new(neighbour.to, alt, &graph.nodes[neighbour.to]));
            }
        }
    }
    (shortest_distances, previous)
}

fn new_wgraph(path: &str) -> io::Result<WGraph> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let lines: Vec<String> = reader
        .lines()
        .into_iter()
        .map(|line| line.expect("Failed to read line"))
        .collect();

    let first_line: Vec<&str> = lines[0].split_whitespace().collect();
    let nodes: usize = first_line[0].parse().expect("File not formated correctly");
    let edges: usize = first_line[1].parse().expect("File not formated correctly");

    let mut graph = WGraph::with_nodes(nodes);
    for i in 1..=edges {
        let data: Vec<usize> = lines[i]
            .split_whitespace()
            .map(|c| c.parse().expect("File not formated correctly"))
            .collect();

        graph.nodes[data[0]].push(EdgeTo::new(data[1], data[2]));
    }
    Ok(graph)
}