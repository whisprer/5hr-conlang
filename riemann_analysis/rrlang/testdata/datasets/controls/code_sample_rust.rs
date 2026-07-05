fn main() {
    let values = [1, 2, 3, 5, 8, 13];
    for value in values {
        if value % 2 == 0 {
            println!("even: {}", value);
        } else {
            println!("odd: {}", value);
        }
    }
}
