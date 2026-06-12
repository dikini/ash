pub fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

fn main() {
    let n = 20;
    let result = fibonacci(n);
    println!("fibonacci({}) = {}", n, result);
}