use std::time::Instant;
fn main() {
    let t0 = Instant::now();
    let mut checksum: i64 = 0;
    for i in 1..=200000i64 {
        println!("row {} of 200000, hits {}", i, i % 97);
        checksum += i % 97;
    }
    let ms = t0.elapsed().as_millis();
    eprintln!("{} checksum", checksum);
    eprintln!("{} ms", ms);
}
