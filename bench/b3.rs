// Not the benchmark: this is the same program with borrowed keys, kept as the
// floor. Lume's `{Str: Int}` owns its keys, so `b3b.rs` is the fair comparison.
use std::collections::HashMap;
use std::time::Instant;
fn main() {
    let text = std::fs::read_to_string("bench/words.txt").unwrap();
    let t0 = Instant::now();
    let mut counts: HashMap<&str, i64> = HashMap::new();
    for line in text.lines() {
        for w in line.split(' ') {
            if !w.is_empty() { *counts.entry(w).or_insert(0) += 1; }
        }
    }
    let mut v: Vec<(&&str, &i64)> = counts.iter().collect();
    v.sort_by_key(|p| -*p.1);
    let ms = t0.elapsed().as_millis();
    for p in v.iter().take(3) { println!("{} {}", p.0, p.1); }
    println!("{} distinct", counts.len());
    println!("{} ms", ms);
}
