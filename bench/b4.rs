use std::time::Instant;
struct Row { name: String, hits: i64 }
fn main() {
    let mut rows: Vec<Row> = Vec::new();
    for i in 0..200000i64 { rows.push(Row { name: format!("row{}", i % 1000), hits: i % 97 }); }
    let t0 = Instant::now();
    let mut parts: Vec<String> = Vec::new();
    for r in rows.iter() { parts.push(format!("{}: {}", r.name, r.hits)); }
    let body = parts.join("\n");
    let ms = t0.elapsed().as_millis();
    println!("{} chars", body.chars().count());
    println!("{} ms", ms);
}
