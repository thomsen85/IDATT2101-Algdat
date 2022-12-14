use std::fs::File;
use std::io::{BufReader, Read};
use std::io;
use std::time::Instant;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let capacity = 13_000_027;
    let random_numbers = 10_000_000;

    let random_vec = get_random_vec(random_numbers).expect("Couldn't get random numbers");
    let mut hash_table = HashTable::new(capacity);
    
    let mut hash_set = HashSet::new();
    let clock_1 = Instant::now();
    for n in &random_vec {
        hash_set.insert(n);
    }
    let time_taken = clock_1.elapsed().as_millis();

    let threads = 10;
    let clock_2 = Instant::now();
    let handle = thread::spawn(move || {
        for i in 0..threads {
            for n in i*(random_numbers/threads)..(i+1)*(random_numbers/threads) {
                hash_table.insert(random_vec[n]);
            }
        }

    });

    handle.join().unwrap();
    let time_taken_2 = clock_2.elapsed().as_millis();

    println!("Build-in time: {} ms", time_taken);
    println!("Self-buildt time: {} ms", time_taken_2)
}

#[inline]
fn fasthash(key: u32, capacity: usize) -> usize {
    let decimal = key as f64 * 0.61803339887;
    let whole_number = decimal as u32;
    let fraction = decimal - whole_number as f64;
    (fraction * capacity as f64) as usize
}

struct HashTable {
    arr: Arc<Mutex<Vec<Option<u32>>>>,
    capacity: usize
}

impl HashTable {
    fn new(capacity: usize) -> Self {
        let mut arr = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            arr.push(None);
        }
        HashTable {arr: Arc::new(Mutex::new(arr)), capacity}
    }

    fn insert(&mut self, value: u32) {
        let mut hash = fasthash(value, self.capacity);
        if let Ok(mut lock) = self.arr.clone().lock() {
            if lock[hash].is_some() {
                let jump_distance = fasthash(value, 64) + 1;
                hash = (hash + jump_distance) % self.capacity;

                while lock[hash].is_some() {
                    hash = (hash + jump_distance) % self.capacity
                }

                lock[hash] = Some(value);
            } else {
                lock[hash] = Some(value);
            }
        }
    }


    fn len(&self) -> usize {
        let mut len = 0;
        if let Ok(lock) = self.arr.clone().lock() {
            for i in 0..self.capacity {
                if lock[i].is_some() {
                    len += 1;
                }
            }
        }
        len
    }
}

fn get_random_vec(len: usize) -> io::Result<Vec<u32>> {
    let file = File::open("/dev/urandom")?;
    let mut reader = BufReader::new(file);
    let mut bytes: Vec<u8> = vec![0u8; len*4];
    let mut vec: Vec<u32> = Vec::with_capacity(len);
    reader.read_exact(&mut bytes)?;

    for i in 0..len{
        vec.push(
        ((bytes[i*4] as u32) <<  0) +
        ((bytes[i*4 + 1] as u32) <<  8) +
        ((bytes[i*4 + 2] as u32) << 16) +
        ((bytes[i*4 + 3] as u32) << 24)
        )
    }
    Ok(vec)
}
