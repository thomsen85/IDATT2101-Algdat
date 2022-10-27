use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{BufReader, Read, BufWriter, Write};
use std::env;
use std::fs::File;
use std::rc::Rc;

// Using 3 bytes as as indicator 3*8 = 24 bits 
static SEARCH_WINDOW_BITS: u8 = 11; // 11 for backref: 2^11 = 2048
static LOOK_AHEAD_BITS: u8 = 5; // 5 for looka ahead: 2^5 = 32
static DISTANCE_BITS: u8 = 8; // 8 for distance unitl next : 2^8 = 256

fn get_indicator_from_data(back_ref: u32, length: u32, distance_to_next: u32) -> [u8; 3] {
    let mut num: u32 = 0x00;
    
    assert!(distance_to_next < 2_u32.pow(DISTANCE_BITS as u32));
    assert!(length < 2_u32.pow(LOOK_AHEAD_BITS as u32));
    assert!(back_ref < 2_u32.pow(SEARCH_WINDOW_BITS as u32), "backref = {}",  back_ref);

    num |= distance_to_next; 
    num |= length << DISTANCE_BITS;   
    num |= back_ref << DISTANCE_BITS + LOOK_AHEAD_BITS;

    let b2: u8 = ((num >> 16) & 0xff) as u8;
    let b3: u8 = ((num >> 8) & 0xff) as u8;
    let b4: u8 = (num & 0xff) as u8;
    [b2, b3, b4]
}

fn get_data_from_indicator(indicator: &[u8; 3]) -> (usize , usize, usize){
    let mut num: usize= 0x00;
    num |= indicator[2] as usize;
    num |= (indicator[1] as usize) << 8;
    num |= (indicator[0] as usize) << 16;

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
        //println!("Adding raw to output buffer {:?}", &bytes[start_pointer..bytes_end_pointer]);

        // Fetching indicator
        let indicator: [u8; 3] = [bytes[bytes_end_pointer], bytes[bytes_end_pointer+1], bytes[bytes_end_pointer+2]];
        let (back_ref, length, distance_to_next) = get_data_from_indicator(&indicator);
        //println!("At {}, Indicator {:?}", bytes_end_pointer, (back_ref, length, distance_to_next));
        
        //println!("output_pointer =  {}",output_end_pointer);
        // Coping data
        output.extend_from_within((output_end_pointer - back_ref)..(output_end_pointer - back_ref + length));
        //println!("Adding ref to outbut buffer, output now {:?}", output);


        start_pointer = bytes_end_pointer + 3;
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

fn hc_encode(bytes: &Vec<u8>) {
    let mut frequency_list: Vec<u32> = vec![0_u32; u8::MAX as usize + 1];
    bytes.iter().for_each(|byte| {
        frequency_list[byte.to_owned() as usize] += 1;
    });

    let mut sorted_freq_list: Vec<(u8, u32)> = frequency_list.iter()
                .enumerate()
                .map(|(byte, freq)|(byte as u8, freq.to_owned()))
                .filter(|item| item.1 != 0)
                .collect();
    sorted_freq_list.sort_by(|a, b| b.1.cmp(&a.1));
    println!("{:?}", sorted_freq_list);


}

struct Tree {
    root_node: Node
}

impl Tree {
    fn new(root_node: Node) -> Self {
        Self { root_node }
    }
}

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
        Self {left: Some(Rc::new(RefCell::new(left))), right: Some(Rc::new(RefCell::new(right))), frequency: left.frequency + right.frequency, value: None }
    }

}

/// Frequency list must be sorted 
fn frequency_list_to_huffman_tree(frequency_list: &Vec<(u8, u32)>) -> Tree {
    loop {
        let (first_value, first_freq) = frequency_list.pop().unwrap();
        let first_node = Node::new_bottom_node(first_value, first_freq);

        if frequency_list.is_empty() {
            
            break
        }

        let (second_value, second_freq) = frequency_list.pop();
        if frequency_list.is_empty() {
            break
        }
    }
}

fn rec(frequency_list: &Vec<(u8, u32)>) -> Node {
    frequency_list.pop() 
    return Node::new_intermidiate_node(rec, rec)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Please (only) path as argument");
        std::process::exit(0)
    }
    let path = &args[1];

    println!("Opening File...");
    let bytes = get_file_as_bytes(path);
    
    hc_encode(&bytes)


    // println!("Encoding File with LZ...");
    // let encoded_bytes = lz_encode(&bytes);
    // //println!("encoded_bytes: {:?}", encoded_bytes);

    // println!("Writing to File...");
    // write_file_as_bytes("out.bin", &encoded_bytes);

    // println!("Decoding File...");
    // let decoded_bytes = lz_decode(&encoded_bytes);
    // //println!("decoded_bytes: {:?}", decoded_bytes);

    // println!("Checking Decoded File...");
    // assert_eq!(bytes.len(), decoded_bytes.len());
    // for i in 0..decoded_bytes.len() {
    //     //println!("{}: {}-{}", i, bytes[i], decoded_bytes[i]);
    //     assert_eq!(bytes[i], decoded_bytes[i], "Wrong at index {}", i);
    // }
    // println!("Successfully compressed with lz.");


}