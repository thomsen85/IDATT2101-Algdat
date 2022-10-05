use std::fs::File;
use std::io::Read;
use std::collections::LinkedList;
use std::cell::RefCell;

fn main() {
    let names = get_names_from_file("navn.txt");
    let capacity = 157; 
    
    let mut table = StringKeyHashTable::new(capacity);
    let mut collisions = 0;
    for name in &names {
        if table.push(name) {
            collisions += 1;
        }     
    }

    println!("{} from file, {} in hash table", &names.len(), table.len());
    println!("Load factor {:.2}", table.load_factor());

    println!("{:?}", table.get("Vetle Ålesve Nordang"));
    println!("{:?}", table.get("Thomas Svendal"));

    println!("Number of collisions: {}", collisions);
    println!("Collisions per person: {:.2}", collisions as f64/names.len() as f64);
}

fn get_names_from_file(path: &str) -> Vec<String> {
    let mut names_file = File::open(path).expect("Couldn't find file");
    let mut buffer = String::new();
    names_file.read_to_string(&mut buffer).expect("Cant read from file");
    let mut names: Vec<String> = buffer.split("\n").into_iter().map(|w| w.to_string()).collect();
    names.pop(); //Bause of split makes an extra at the end
    names
}

#[inline]
fn djb2_hash(string: &str) -> usize {
    let mut hash: usize = 5381;
    for c in string.chars() {
        hash = ((hash << 5).wrapping_add(hash)) + c as usize;
    }
    hash
}

struct StringKeyHashTable {
    arr: Vec<Option<RefCell<LinkedList<String>>>>,
    capacity: usize
}

impl StringKeyHashTable {
    fn new(capacity: usize ) -> Self {
        let mut arr = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            arr.push(None);
        }
        Self { arr, capacity}
    }

    fn push(&mut self, name: &str) -> bool{
        let hash = djb2_hash(name) % self.capacity;
        if let Some(list) = &self.arr[hash] {
            list.borrow_mut().push_back(name.to_string());
            println!("Collision with {} at hash {}", name, hash);
            return true;
        } else {
            let mut linked_list = LinkedList::new();
            linked_list.push_back(name.to_string());
            self.arr[hash] = Some(RefCell::new(linked_list)); 
            return false;
        }
    }

    fn get(&self, name: &str) -> Option<String> {
        let hash = djb2_hash(name) % self.capacity;
        if let Some(list) = &self.arr[hash] {
            for list_name in list.borrow().iter() {
                if name == list_name {
                    return Some(list_name.clone());
                }
            }
        }
    None
    }

    fn len(&self) -> usize {
        let mut num = 0;
        for n in &self.arr {
            if let Some(list) = n {
                num += list.borrow().len();
            }
        }
        num
    }

    // With or without linked lists
    fn load_factor(&self) -> f64 {
        self.len() as f64 / self.capacity as f64 
    }
}

