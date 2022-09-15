use std::io;
use std::io::{ Read, Write };
use std::io::BufReader;
use std::fs;
use std::time::Instant;

fn single_pivot_quicksort(arr: &mut Vec<i32>, left: usize, right: usize) {
    if right as i32 - left as i32 > 2 {
        let mid = split(arr, left, right);
        single_pivot_quicksort(arr, left, mid-1);
        single_pivot_quicksort(arr, mid + 1, right);
    } else {
        median3sort(arr, left, right);
    }
}

fn median3sort(arr: &mut Vec<i32>, left: usize, right: usize) -> usize {
    let mid = (left + right) / 2;
    if arr[left] > arr[mid] {
        arr.swap(left, mid);
    }
    if arr[mid] > arr[right] {
        arr.swap(mid, right);
        if arr[left] > arr[mid] {
            arr.swap(left, mid);
        }
    }
    mid
}

fn split(arr: &mut Vec<i32>, left: usize, right: usize) -> usize {
    let mut left_index = left;
    let mut right_index = right-1;
    let mid = median3sort(arr, left, right );
    let pivot = arr[mid];
    
    arr.swap(mid, right -1);
    loop {
        loop {
            left_index += 1;
            if arr[left_index] >= pivot {
                break
            }
        }
        loop {
            right_index -= 1;
            if arr[right_index] <= pivot {
                break
            }
        }
        if left_index >= right_index {
            break
        }

        arr.swap(left_index, right_index);
    }
    arr.swap(left_index, right - 1);
    left_index
}

fn dual_pivot_quicksort(arr: &mut Vec<i32>, left: isize, right: isize) {
    if left < right {
        let (left_pivot, right_pivot) = partition(arr, left as usize, right as usize);
        let left_pivot = left_pivot as isize;
        let right_pivot = right_pivot as isize;

        dual_pivot_quicksort(arr, left, left_pivot-1);
        dual_pivot_quicksort(arr, left_pivot + 1, right_pivot - 1);
        dual_pivot_quicksort(arr, right_pivot + 1, right);
    }
}


fn partition(arr: &mut Vec<i32>, left: usize, right: usize) -> (usize, usize){
    let a_third2 = right - (right -left) / 3;

    arr.swap(left, left + (right-left) / 3);
    arr.swap(right, right - (right -left) / 3);
    
    if arr[left] > arr[right] {
        arr.swap(left, right);
    }

    let mut j = left + 1;
    let mut k = left + 1;
    let mut g = right - 1;
    let left_pivot = arr[left];
    let right_pivot = arr[right];

    while k <= g {
        if arr[k] < left_pivot {
            arr.swap(k, j);
            j += 1
        } else if arr[k] >= right_pivot {
            while arr[g] > right_pivot && k < g {
                g -= 1;
            }
            
            arr.swap(k, g);
            g -= 1; 

            if arr[k] < left_pivot {
                arr.swap(k, j);
                j += 1;
            }
        }
        
        k += 1
    }
    j -= 1;
    g += 1;

    arr.swap(left, j);
    arr.swap(right, g);

    (j, g)

}

fn main() {
    let multiplier = 10_000_000;
    let run_point: Vec<usize> = vec![1_usize; 10].iter().enumerate().map(|(p, i)| (p+1) * i * multiplier).collect();
    let mut s_random_result_times: Vec<(usize, u128)> = Vec::with_capacity(run_point.len());
    let mut s_sorted_result_times: Vec<(usize, u128)> = Vec::with_capacity(run_point.len());
    let mut s_almost_same_result_times: Vec<(usize, u128)> = Vec::with_capacity(run_point.len());
    let mut d_random_result_times: Vec<(usize, u128)> = Vec::with_capacity(run_point.len());
    let mut d_sorted_result_times: Vec<(usize, u128)> = Vec::with_capacity(run_point.len());
    let mut d_almost_same_result_times: Vec<(usize, u128)> = Vec::with_capacity(run_point.len());

    for p in run_point {
        // Random single
        if let Ok(mut vec) = get_random_vec(p) {
            s_random_result_times.push((p, test_vec_single(&mut vec)));
        } else {
            println!("Something went wrong with file reading")
        }
        // Sorted single 
        s_sorted_result_times.push((p, test_vec_single(&mut get_sorted_vec(p))));
        // Almost same single
        s_almost_same_result_times.push((p, test_vec_single(&mut get_many_same_vec(p))));

        // Random dual
        if let Ok(mut vec) = get_random_vec(p) {
            d_random_result_times.push((p, test_vec_dual(&mut vec)));
        } else {
            println!("Something went wrong with file reading")
        }
        // Sorted dual 
        d_sorted_result_times.push((p, test_vec_dual(&mut get_sorted_vec(p))));
        // Almost same dual
        d_almost_same_result_times.push((p, test_vec_dual(&mut get_many_same_vec(p))));
    }

    print_result_to_file(s_random_result_times, "s_random_result.csv"); 
    print_result_to_file(s_sorted_result_times, "s_sorted_result.csv"); 
    print_result_to_file(s_almost_same_result_times, "s_many_same_result.csv"); 
    print_result_to_file(d_random_result_times, "d_random_result.csv"); 
    print_result_to_file(d_sorted_result_times, "d_sorted_result.csv"); 
    print_result_to_file(d_almost_same_result_times, "d_many_same_result.csv"); 
}

fn test_vec_dual(vec: &mut Vec<i32>) -> u128 {
    let len = vec.len();
    let pre_sum= sum(&vec);
    let timer = Instant::now();
    dual_pivot_quicksort(vec, 0, (len-1) as isize);
    let time_taken = timer.elapsed().as_millis();
    let post_sum = sum(vec);
    println!("List sorted correctly:      {}", check_if_sorted(&vec));
    println!("Sum before and after match: {}", pre_sum == post_sum);
    time_taken
}

fn test_vec_single(vec: &mut Vec<i32>) -> u128 {
    let len = vec.len();
    let pre_sum= sum(&vec);
    let timer = Instant::now();
    single_pivot_quicksort(vec, 0, len-1);
    let time_taken = timer.elapsed().as_millis();
    let post_sum = sum(vec);
    println!("List sorted correctly:      {}", check_if_sorted(&vec));
    println!("Sum before and after match: {}", pre_sum == post_sum);
    time_taken
}

fn sum(vec: &Vec<i32>) -> u128 {
    let mut sum: u128 = 0; 
    for num in vec {
        sum += *num as u128;
    }
    sum
}

fn print_result_to_file(result: Vec<(usize, u128)>, file_name: &str) -> std::io::Result<()> {
    let mut file = fs::File::create(file_name)?;
    for v in result {
        file.write(format!("{}, {}\n", v.0, v.1).as_bytes())?;
    }
    Ok(())
    
}
fn check_if_sorted(vec: &Vec<i32>) -> bool {
    for i in 1..vec.len() {
        if vec[i-1] > vec[i] {
            return false;
        }
    }
    return true;
 }

fn get_many_same_vec(len: usize) -> Vec<i32> {
    let mut vec: Vec<i32> = Vec::with_capacity(len);
    let mut same = true;
    for i in 0..(len as i32) {
        if same {
            vec.push(5);
            same = false;
        } else {
            vec.push(i);
            same = true;
        }
    }
    vec
}



fn get_sorted_vec(len: usize) -> Vec<i32>{
    let mut vec: Vec<i32> = Vec::with_capacity(len);
    for i in 0..len{
        vec.push((i) as i32);
    }
    vec
}

fn get_random_vec(len: usize) -> io::Result<Vec<i32>> {
    let file = fs::File::open("/dev/urandom")?;
    let mut reader = BufReader::new(file);
    let mut bytes: Vec<u8> = vec![0u8; len*2];
    let mut vec: Vec<i32> = Vec::with_capacity(len);
    reader.read_exact(&mut bytes)?;

    for i in 0..len{
        vec.push(
        ((bytes[i*2] as i32) <<  0) +
        ((bytes[i*2 + 1] as i32) <<  8)
        )
    }
    

    return Ok(vec);
}
