use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::io::{BufReader, Read, BufWriter, Write};
use std::env;
use std::fs::File;
use std::rc::Rc;

const INDICATOR_BYTE: usize = 4;
static SEARCH_WINDOW_BITS: u8 = 15; // 11 for backref: 2^15 = 32768
static LOOK_AHEAD_BITS: u8 = 9; // 5 for looka ahead: 2^9 = 512
static DISTANCE_BITS: u8 = 8; // 8 for distance unitl next : 2^8 = 256

fn get_indicator_from_data(back_ref: u32, length: u32, distance_to_next: u32) -> [u8; INDICATOR_BYTE] {
    let mut num: u32 = 0x00;
    
    assert!(distance_to_next < 2_u32.pow(DISTANCE_BITS as u32));
    assert!(length < 2_u32.pow(LOOK_AHEAD_BITS as u32));
    assert!(back_ref < 2_u32.pow(SEARCH_WINDOW_BITS as u32), "backref = {}",  back_ref);

    num |= distance_to_next; 
    num |= length << DISTANCE_BITS;   
    num |= back_ref << DISTANCE_BITS + LOOK_AHEAD_BITS;

    let b1: u8 = ((num >> 24) & 0xff) as u8;
    let b2: u8 = ((num >> 16) & 0xff) as u8;
    let b3: u8 = ((num >> 8) & 0xff) as u8;
    let b4: u8 = (num & 0xff) as u8;
    [b1, b2, b3, b4]
}

fn get_data_from_indicator(indicator: &[u8; INDICATOR_BYTE]) -> (usize , usize, usize){
    let mut num: usize = 0;
    num |= indicator[3] as usize;
    num |= (indicator[2] as usize) << 8;
    num |= (indicator[1] as usize) << 16;
    num |= (indicator[0] as usize) << 24;


    let distance_to_next = num & (2_usize.pow(DISTANCE_BITS as u32) - 1);
    let length   = (num >> DISTANCE_BITS) & (2_usize.pow(LOOK_AHEAD_BITS as u32) - 1);
    let back_ref = (num >> DISTANCE_BITS + LOOK_AHEAD_BITS) & (2_usize.pow((SEARCH_WINDOW_BITS) as u32) - 1);
    (back_ref, length, distance_to_next)
}

fn get_byte_array_from_u16(input: u16) -> [u8; 2]{
    let b1: u8 = ((input >> 8) & 0xff) as u8;
    let b2: u8 = (input & 0xff) as u8;
    [b1, b2]
}

fn get_u16_from_byte_array(input: &[u8; 2]) -> u16 {
    let mut num = 0;

    num |= input[1] as u16;
    num |= (input[0] as u16) << 8 ;
    num
}


fn lz_encode(bytes: &Vec<u8>) -> Vec<u8> {
    let mut split = 0;
    let mut distance = 0; 
    let mut buffer_start: usize= 0;
    let mut buffer_end: usize= 0;
    let mut output = Vec::new();
    let mut first = true; 

    let mut prev_pointer: usize = 0;
    let mut prev_length = 0; 

    while split < bytes.len() {
        let (back_ref, length) = find_match_in_window(bytes, split, SEARCH_WINDOW_BITS, LOOK_AHEAD_BITS);
        if length > 3 {
            //println!("Match found at: {}, back: {}, length:{}", split, back_ref, length);
            if first {
                output.extend(get_byte_array_from_u16(split as u16));
                first = false; 
            } else {
                let indicator = get_indicator_from_data(prev_pointer as u32, prev_length, distance);
                //println!("Creating referance at {}, to: {}, length: {}", split, prev_pointer,  prev_length);
                output.extend(indicator);
            }
            output.extend(&bytes[buffer_start..buffer_end]);
            prev_pointer = back_ref;
            prev_length = length;
            split += length as usize;

            buffer_start = split;
            distance = 0;
        }
        split += 1;
        distance += 1;
        buffer_end = split;
    }

    let indicator = get_indicator_from_data(prev_pointer as u32, prev_length, distance);
    //println!("Creating referance at {}, to: {}, length: {}", split, prev_pointer,  prev_length);
    output.extend(indicator);
        
    output.extend(&bytes[buffer_start..buffer_end]); 

    output
}

fn lz_decode(bytes: &Vec<u8>) ->Vec<u8> {
    let first_start = get_u16_from_byte_array(&[bytes[0], bytes[1]]) as usize ;
    //println!("First at {}", first_start);
    let mut output = Vec::new();
    let mut start_pointer  = 2;
    let mut bytes_end_pointer = first_start + start_pointer;

    while bytes_end_pointer < bytes.len() {
        // Adding raw bytes
        output.extend(&bytes[start_pointer..bytes_end_pointer]);
        let output_end_pointer = output.len(); 

        // Fetching indicator
        let indicator: [u8;  INDICATOR_BYTE] = [bytes[bytes_end_pointer], bytes[bytes_end_pointer+1], bytes[bytes_end_pointer+2], bytes[bytes_end_pointer+3]];
        let (back_ref, length, distance_to_next) = get_data_from_indicator(&indicator);
        
        //println!("output_pointer =  {}",output_end_pointer);
        // Coping data
        println!("{} - {}", output_end_pointer, back_ref);
        output.extend_from_within((output_end_pointer - back_ref)..(output_end_pointer + length - back_ref ));
        //println!("Adding ref to outbut buffer, output now {:?}", output);


        start_pointer = bytes_end_pointer + 4;
        bytes_end_pointer = start_pointer + distance_to_next;
    }
    output.extend(&bytes[start_pointer..bytes_end_pointer]);
    output
}

/// Finds matches in search window 
/// 
/// Note:  split is the index of the first look ahead byte 
fn find_match_in_window(bytes:&Vec<u8>, split: usize, search_window_bits: u8, look_ahead_bits: u8) -> (usize, u32) {
    if split < 1 {
        return (0,0);
    }
    let mut search_pointer = split - 1;     
    let search_pointer_end: usize = 
        if 2_usize.pow(search_window_bits as u32) - 1 >= split {
            0_usize
        } else {
            split - (2_usize.pow(search_window_bits as u32) - 1)
        };

    let mut look_ahead_pointer = split;
    let mut longest_match_len: u32 = 0;
    let mut longest_match_pointer = split;
    
    let mut temp_longest_match_len: u32 = 0;
    let mut temp_longest_match_pointer = split;

    loop {
        let old_search_pointer = search_pointer; 
        while bytes[search_pointer] == bytes[look_ahead_pointer] {
            temp_longest_match_len += 1;
            if temp_longest_match_len == 1 {
                temp_longest_match_pointer = search_pointer;
            }

            search_pointer += 1;
            if search_pointer >= split {
                break
            }
            look_ahead_pointer += 1;
            if temp_longest_match_len >= 2_u32.pow(look_ahead_bits as u32) - 1{
                break
            }
            if look_ahead_pointer >= bytes.len() {
                break;
            }
        }

        if temp_longest_match_len > longest_match_len {
            longest_match_len = temp_longest_match_len;
            longest_match_pointer = temp_longest_match_pointer
        }

        search_pointer = old_search_pointer;
        look_ahead_pointer = split;
        temp_longest_match_len = 0;
        temp_longest_match_pointer = split;

        if search_pointer <= search_pointer_end {
            break
        }
        search_pointer -= 1;
    }

    (split - longest_match_pointer, longest_match_len)

}

fn get_file_as_bytes(path: &str) -> Vec<u8> {
    let mut buffer = Vec::new();
    let file = File::open(path).expect("File could not be opened");
    let mut reader = BufReader::new(file);
    reader.read_to_end(&mut buffer).expect("File could not be read");
    buffer
}

fn write_file_as_bytes(path: &str, bytes: &Vec<u8>) {
    let file = File::create(path).expect("File could not be created");
    let mut writer = BufWriter::new(file);
    writer.write(bytes).expect("File could not be written");
}

fn hc_encode(bytes: &Vec<u8>) -> Vec<u8>{
    let mut frequency_list: Vec<u32> = vec![0_u32; u8::MAX as usize + 1];
    bytes.iter().for_each(|byte| {
        frequency_list[byte.to_owned() as usize] += 1;
    });

    let freq_list: Vec<(u8, u32)> = frequency_list.iter()
                .enumerate()
                .map(|(byte, freq)|(byte as u8, freq.to_owned()))
                .filter(|item| item.1 != 0)
                .collect();


    let tree = frequency_list_to_huffman_tree(&freq_list);

    let mut bit_builder = BitBuilder::new();
    bit_builder.add_byte((freq_list.len() - 1) as u8);
    for freq_item in freq_list {
        bit_builder.add_byte(freq_item.0);
        bit_builder.add_u32(freq_item.1)
    }

    for byte in bytes {
        bit_builder.extend(tree.get_encoded_bit_array_from_byte(byte.to_owned()))
    }

    bit_builder.collect()
}

fn get_u32_from_byte_array(bytes: &[u8 ;4]) -> u32 {
    let mut num = 0;

    num |= bytes[3] as u32;
    num |= (bytes[2] as u32) << 8;
    num |= (bytes[1] as u32) << 16;
    num |= (bytes[0] as u32) << 24;

    num
}

fn hc_decode(bytes: &Vec<u8>) -> Vec<u8> {
    let length = bytes[0] as usize + 1;

    let mut freq_list: Vec<(u8, u32)> = Vec::with_capacity(length.into());
    for i in 0..length {
        let base = 1 + (i*5) as usize;
        let val = bytes[base];
        let freq = get_u32_from_byte_array(&[bytes[base+1], bytes[base+2], bytes[base+3], bytes[base+4]]);
        freq_list.push((val, freq));
    }

    let tree = frequency_list_to_huffman_tree(&freq_list);
    
    let res = tree.get_decode_bytes(&bytes, (1 + 5*length)*8);
    res

}
struct BitBuilder {
    bytes: Vec<u8>, 
    buffer_byte: u8,
    bit_pos: u8,
}

impl BitBuilder {
    fn new() -> Self {
        Self { bytes:Vec::new(), buffer_byte: 0, bit_pos: 0}
    }

    fn extend(&mut self, bits: &Vec<bool>) {
        for bit in bits {
            if self.bit_pos >= 8 {
                self.bit_pos = 0;
                self.bytes.push(self.buffer_byte);
                self.buffer_byte = 0; 
            } 
            
            if *bit {
                self.buffer_byte |= 1 << (7 - self.bit_pos)
            }

            self.bit_pos += 1
        }
    }

    fn add_byte(&mut self, byte: u8) {
        self.bytes.push(byte);
    }

    fn add_u32(&mut self, input: u32) {
        let b1: u8 = ((input >> 24) & 0xff) as u8;
        let b2: u8 = ((input >> 16) & 0xff) as u8;
        let b3: u8 = ((input >> 8) & 0xff) as u8;
        let b4: u8 = (input & 0xff) as u8;

        self.add_byte(b1);
        self.add_byte(b2);
        self.add_byte(b3);
        self.add_byte(b4);
    }

    fn collect(mut self) -> Vec<u8> {
        let rest = self.buffer_byte;
        self.bytes.push(rest);
        self.bytes.push(self.bit_pos as u8);
        self.bytes
    }
}

#[derive(Debug)]
struct Tree {
    root_node: Node,
    lookup_table: Vec<Vec<bool>>,
}

impl Tree {
    fn new(root_node: Node) -> Self {
        let mut lookup_table = vec![vec![]; 256];
        Tree::fill_lookup_table(&root_node, &mut lookup_table, Vec::new());
        Self { root_node, lookup_table }
    }

    fn fill_lookup_table(node: &Node, lookup_table: &mut Vec<Vec<bool>>, progress: Vec<bool>) {
        if let Some(val) = node.value {
            lookup_table[val as usize] = progress.clone();
        } else {
            if let Some(l_node) = &node.left {
                let mut l_progress = progress.clone();
                l_progress.push(false);
                Tree::fill_lookup_table(&Rc::clone(l_node).as_ref().borrow(), lookup_table, l_progress);
            }
            if let Some(r_node) = &node.right {
                let mut r_progress = progress.clone();
                r_progress.push(true);
                Tree::fill_lookup_table(&Rc::clone(r_node).as_ref().borrow(), lookup_table, r_progress)
            }
        }     
    }

    fn get_encoded_bit_array_from_byte(&self, index: u8) -> &Vec<bool> {
        &self.lookup_table[index as usize]
    }


    fn get_decode_bytes(&self, encoded_bytes: &Vec<u8>, start: usize) -> Vec<u8> {
        let mut buff = Vec::new();
        let mut res = Vec::new();
        let last_active_bits = encoded_bytes.last().unwrap();
        let stop =  ((encoded_bytes.len() - 1) * 8) - (7  - last_active_bits.to_owned() as usize);
        
        for i in start..stop {
            buff.push(get_bit(encoded_bytes, i));

            if self.lookup_table.contains(&buff) {
                res.push(self.lookup_table.iter().position(|o|  o == &buff).unwrap() as u8);
                buff.clear();
            }
        }
        res
    }

}

fn get_bit(bytes: &Vec<u8>, index: usize) -> bool {
    let byte = bytes[index / 8];
    let index = index % 8;
    let result = byte >> (7 - index) & 1 != 0; 

    result
}

#[derive(Debug, PartialEq, Eq)]
struct Node {
    left: Option<Rc<RefCell<Node>>>,
    right: Option<Rc<RefCell<Node>>>,
    frequency: u32,
    value: Option<u8>, 
}

impl Node {
    fn new_bottom_node(frequency: u32, value: Option<u8>) -> Self{
        Self { left: None, right: None, frequency, value }
    }

    fn new_intermidiate_node(left: Node, right: Node) -> Self {
        let freq = left.frequency + right.frequency;
        Self {left: Some(Rc::new(RefCell::new(left))), right: Some(Rc::new(RefCell::new(right))), frequency: freq, value: None }
    }

}

impl Ord for Node{
    fn cmp(&self, other: &Self) -> Ordering {
        self.frequency.cmp(&other.frequency).reverse()
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Frequency list must be sorted 
fn frequency_list_to_huffman_tree(frequency_list: &Vec<(u8, u32)>) -> Tree {
    let mut heap = BinaryHeap::new();
    frequency_list.iter().for_each(|o| {
        heap.push(Node::new_bottom_node(o.1, Some(o.0)));
    }); 

    while heap.len() > 1 {
        let left = heap.pop().unwrap();
        let right = heap.pop().unwrap();
        
        heap.push(Node::new_intermidiate_node(left, right));
    }
    Tree::new(heap.pop().unwrap())
}


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Please (only) path as argument");
        std::process::exit(0)
    }
    let path = &args[1];

    println!("Opening file...");
    let bytes = get_file_as_bytes(path);
    
    println!("Encoding bytes...");
    let lz_encoded = lz_encode(&bytes);
    let hc_encoded = hc_encode(&lz_encoded);

    println!("Writing bytes to file...");
    write_file_as_bytes("out.bin", &hc_encoded);

    println!("Decoding bytes...");
    let hc_decoded_bytes = hc_decode(&hc_encoded);
    let lz_decoded_bytes = lz_decode(&hc_decoded_bytes);


    println!("Checking decoded bytes...");
    assert_eq!(bytes.len(), lz_decoded_bytes.len());
    for i in 0..lz_decoded_bytes.len() {
        //println!("{}: {}-{}", i, bytes[i], lz_decoded_bytes[i]);
        assert_eq!(bytes[i], lz_decoded_bytes[i], "Wrong at index {}", i);
    }
    println!("Successfully compressed and decompressed.");
}