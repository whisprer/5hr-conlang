fn main() {
    let mut total: u64 = 0;
    for i in 0..10_000 {
        if is_prime(i) {
            total += i;
        }
    }
    println!("total={}", total);
}

fn is_prime(n: u64) -> bool {
    if n < 2 { return false; }
    let mut d = 2;
    while d * d <= n {
        if n % d == 0 { return false; }
        d += 1;
    }
    true
}
