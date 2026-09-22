use std::collections::HashMap;
use std::time::Instant;
fn main() {
    let text = std::fs::read_to_string("bench/words.txt").unwrap();
    let t0 = Instant::now();
    // owned keys, as Lume's {Str: Int} has
    let mut counts: HashMap<String, i64> = HashMap::new();
    for line in text.lines() {
        for w in line.split(' ') {
            if !w.is_empty() {
                match counts.get_mut(w) {
                    Some(v) => *v += 1,
                    None => { counts.insert(w.to_string(), 1); }
                }
            }
        }
    }
    let mut v: Vec<(&String, &i64)> = counts.iter().collect();
    v.sort_by_key(|p| -*p.1);
    let ms = t0.elapsed().as_millis();
    for p in v.iter().take(3) { println!("{} {}", p.0, p.1); }
    println!("{} distinct", counts.len());
    println!("{} ms", ms);
}
