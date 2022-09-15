use std::time::Instant;


pub fn main() {
    let test_points = [(1.01,50), (1.01, 500), (1.01, 1000), (1.01, 2000)];
    let test_amount = 1_000_000;
    // Pow 1
    for p in test_points {
        let pow_clock = Instant::now();
        let mut pow_result = 0.0;

        for _ in 0..test_amount {
            pow_result = pow(p.0, p.1);
        }
        println!("{}^{} = {}", p.0, p.1, pow_result);
        let pow_result_time = pow_clock.elapsed().as_millis();
        println!("Pow took: {:>15} ms\n", pow_result_time);
    }


    // Pow 2
    for p in test_points {
        let pow2_clock = Instant::now();
        let mut pow2_result = 0.0;

        for _ in 0..test_amount {
            pow2_result = pow2(p.0, p.1);
        }
        println!("{}^{} = {}", p.0, p.1, pow2_result);

        let pow2_result_time = pow2_clock.elapsed().as_millis();
        println!("Pow2 took: {:>14} ms\n", pow2_result_time);
    }

    // Build in pow
    for p in test_points {
        let build_in_pow_clock = Instant::now();
        let mut build_in_pow_result = 0.0;

        for _ in 0..test_amount {
            build_in_pow_result = p.0.powi(p.1);
        }
        println!("{}^{} = {}", p.0, p.1, build_in_pow_result);

        let build_in_pow_result_time = build_in_pow_clock.elapsed().as_millis();
        println!("Built in pow took: {:>6} ms\n", build_in_pow_result_time);
    }
    /*
    
    if pow_result == pow2_result && pow_result == build_in_pow_result {
        println!("OK");
    } else {
        println!("Failed:");
        println!("Pow: {}",  pow_result);
        println!("Pow2: {}",  pow2_result);
        println!("x.pow: {}",  build_in_pow_result);
    }
    */
}

fn pow(x: f64, n: i32) -> f64 {
    if n == 0 {
        return 1.0;
    }

    x * pow(x, n-1)
}

fn pow2(x: f64, n: i32) -> f64 {
    if n == 0 {
        return 1.0;
    }

    if n % 2 == 1 {
        return x * pow2(x*x, (n-1)/2);
    } else {
        return pow2(x*x, n/2);
    }
}
