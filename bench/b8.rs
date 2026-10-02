use std::collections::HashMap;
use std::time::Instant;
struct Entry { at: i64, path: String, status: i64, ms: i64 }
// seconds since 1970 for a date (Howard Hinnant's days_from_civil)
fn days(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}
fn stamp(s: &str) -> Option<i64> {
    let n = |a: usize, b: usize| s.get(a..b)?.parse::<i64>().ok();
    if s.len() != 20 { return None; }
    Some(days(n(0, 4)?, n(5, 7)?, n(8, 10)?) * 86400 + n(11, 13)? * 3600 + n(14, 16)? * 60 + n(17, 19)?)
}
fn parse(line: &str) -> Option<Entry> {
    let p: Vec<&str> = line.split(' ').collect();
    if p.len() != 6 { return None; }
    let ms = p[5].strip_suffix("ms")?.parse().ok()?;
    Some(Entry { at: stamp(p[0])?, path: p[3].to_string(), status: p[4].parse().ok()?, ms })
}
fn main() {
    let text = std::fs::read_to_string("bench/access.log").unwrap();
    let t0 = Instant::now();
    let all: Vec<Entry> = text.lines().filter_map(parse).collect();
    let mut times: Vec<i64> = all.iter().map(|e| e.ms).collect();
    times.sort();
    let failed = all.iter().filter(|e| e.status >= 500).count();
    let p95 = times[(95 * times.len() + 99) / 100 - 1];
    let mut by_path: HashMap<&str, usize> = HashMap::new();
    for e in &all { *by_path.entry(&e.path).or_default() += 1; }
    let top = by_path.iter().max_by_key(|(_, n)| **n).map(|(p, _)| *p).unwrap_or("-");
    let span = all.last().map(|e| e.at).unwrap_or(0) - all.first().map(|e| e.at).unwrap_or(0);
    let ms = t0.elapsed().as_millis();
    println!("{} {} {} {} {}", all.len(), failed, p95, top, span);
    println!("{} ms", ms);
}
