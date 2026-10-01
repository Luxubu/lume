use std::time::Instant;
struct Rng { s: i64 }
impl Iterator for Rng {
    type Item = i64;
    fn next(&mut self) -> Option<i64> {
        self.s = (self.s * 1103515245 + 12345) % 2147483648;
        Some(self.s)
    }
}
fn main() {
    // ten rounds, each from its own seed, as in the Lume program
    let t0 = Instant::now();
    let mut total = 0i64;
    for round in 0..10i64 {
        total += Rng { s: round + 1 }.filter(|x| x % 3 == 0).map(|x| x % 1000).take(2000000).sum::<i64>();
    }
    let ms = t0.elapsed().as_millis();
    println!("{}", total);
    println!("{} ms", ms);
}
