
use std::cmp::{Ord, Ordering, Reverse};
use std::collections::{BinaryHeap, binary_heap};
use std::fs::File;
use std::io;
use std::io::{BufReader, BufRead};
use std::time::Instant;

struct PrioirtyQueue {

}

#[derive(Debug)]
struct WGraph {
   // Neighbour list
   nodes: Vec<Vec<EdgeTo>>
}

impl WGraph {
    fn with_nodes(amount_of_nodes: usize) -> Self {
        let mut nodes: Vec<Vec<EdgeTo>> = Vec::with_capacity(amount_of_nodes);
        for _ in 0..amount_of_nodes {
            nodes.push(Vec::new());
        }
        Self {nodes}
    }
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq)]
struct EdgeTo {
    to: usize,
    weight: usize,
}

impl EdgeTo {
    fn new(to: usize, weight: usize) -> Self {
        Self {to, weight}
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
        Self {number, nodes, cost}
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


fn main(){
    const paths: [&str; 6] = ["vg1", "vg2", "vg3", "vg4",  "vg5", "vgSkandinavia"];
    // Load graph
    for graph_path in paths {
        let graph = new_wgraph(graph_path).expect("Could not load graph from file");
        let length = graph.nodes.len();
        let source: usize = 1;

        // Init variables
        let mut shortest_distances = vec![usize::MAX/2; length]; 
        let mut previous: Vec<Option<usize>> = vec![None; length];
        let mut priority_queue: BinaryHeap<Priority> = BinaryHeap::with_capacity(length);
        shortest_distances[source] = 0;

        // Push source variable
        priority_queue.push(Priority::new(source, 0, &graph.nodes[source]));
        let time = Instant::now();

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
        let time_taken = time.elapsed();
        println!("Result For graph {}:", graph_path);
        if length < 100 {
            println!("+----------------+----------------+----------------+");
            println!("|{: <15} | {: <15}| {: <15}|","Node", "Prev", "Distance");
            for i in 0..length {
                if i == source {
                    println!("|{: <15} | {: <15}| {: <15}|", i, "Start", shortest_distances[i])
                } else if let Some(prev) = previous[i]  {
                    println!("|{: <15} | {: <15}| {: <15}|", i, prev, shortest_distances[i])
                }  else {
                    println!("|{: <15} | {: <15}| {: <15}|", i, "Not Reached", "Not Reached")
                }
            }
            println!("+----------------+----------------+----------------+");
        } else {
            println!("Took {} ms", time_taken.as_micros() as f64 / 1000.0);
        }
    }


    /*
    let starting_node = 1;
    for node in &graph.nodes[starting_node] {
        priority_queue.push(node)
    }

    println!("{:?}", priority_queue);
    while priority_queue.len() > 0 {
        let node = priority_queue.pop().unwrap();
        length 
        if shortest_distances[node.to] > node.weight {
            shortest_distances[node.to] = node.weight
        }
        
    }
    */
   
}

fn new_wgraph(path: &str) -> io::Result<WGraph>{
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let lines: Vec<String> = reader.lines().into_iter()
                .map(|line| 
                    line.expect("Failed to read line"))
                .collect();
            
    let first_line: Vec<&str> = lines[0].split_whitespace().collect();
    let nodes: usize = first_line[0].parse().expect("File not formated correctly");
    let edges: usize = first_line[1].parse().expect("File not formated correctly");

    let mut graph = WGraph::with_nodes(nodes);
    for i in 1..=edges {
        let data: Vec<usize> = lines[i].split_whitespace()
                    .map(|c| c.parse().expect("File not formated correctly"))
                    .collect();

        graph.nodes[data[0]].push(EdgeTo::new(data[1], data[2]));
    }
    Ok(graph)
}
