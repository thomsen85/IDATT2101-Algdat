use std::time::Instant;
use std::fs::File;
use std::io::Read;
use std::io::BufReader;


#[derive(Debug)]
struct Investment {
    buy_day: usize,
    sell_day: usize
}

fn get_best_investment(diff_points: &[i32]) -> Option<Investment> {
    // In case
    if diff_points.len() < 2 {
        return None;
    }

    let mut last_zero_point_index = 0;
    let mut max_profit_value = 0;
    let mut max_profit_zero_point_index = 0;
    let mut max_profit_last_index = 1;
    let mut current_value = 0;

    for (i, value) in diff_points.iter().enumerate() {
        if &current_value + value <= 0 {
            current_value = 0;
            last_zero_point_index = i;
        } else {
            current_value += *value
        }
        
        if current_value > max_profit_value {
            max_profit_value = current_value;
            max_profit_last_index = i;
            max_profit_zero_point_index = last_zero_point_index;
        }
    }

    if max_profit_value == 0 {
        return None;
    }
    
    Some(Investment{buy_day: max_profit_zero_point_index, sell_day: max_profit_last_index})
}

fn get_diff_points() -> Vec<i32> {
    let mut bytes = vec![0u8; 10_000_000]; 
    match File::open("/dev/urandom") {
        Ok(file) => {
            let mut reader = BufReader::new(file);
            reader.read_exact(&mut bytes).unwrap();
        }
        Err(_err) => {
            panic!("Could not open /dev/urandom");
        }
    }

    bytes.into_iter().map(|b| (b as i32) - 120).collect()

}

fn main() {
    let test_points = [500_000, 1_000_000, 2_000_000, 4_000_000, 8_000_000];
    let diff_points = get_diff_points();
    for test_point in test_points {

        let diff_points_cut = &diff_points[0..test_point];
        let timer = Instant::now();

        let answer = get_best_investment(diff_points_cut);

        let time_elapsed = timer.elapsed();
        println!("{} n - took {} ms - {:?}", test_point, time_elapsed.as_micros() as f64 / 1000.0 , answer.unwrap());
    }
    // Output [optimized]
    // 500000 n - took 1.778 ms - Investment { buy_day: 12, sell_day: 499968 }
    // 1000000 n - took 2.847 ms - Investment { buy_day: 12, sell_day: 999986 }
    // 2000000 n - took 5.653 ms - Investment { buy_day: 12, sell_day: 1999915 }
    // 4000000 n - took 9.686 ms - Investment { buy_day: 12, sell_day: 3999832 }
    // 8000000 n - took 16.147 ms - Investment { buy_day: 12, sell_day: 7999969 }
}
