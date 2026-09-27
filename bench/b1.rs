use std::time::Instant;
fn map_all<T, U, F: FnMut(&T) -> U>(xs: &Vec<T>, mut f: F) -> Vec<U> {
    let mut out = Vec::new();
    for x in xs { out.push(f(x)); }
    out
}
fn keep<T: Clone, F: FnMut(&T) -> bool>(xs: &Vec<T>, mut ok: F) -> Vec<T> {
    let mut out = Vec::new();
    for x in xs { if ok(x) { out.push(x.clone()); } }
    out
}
fn fold_all<T, A, F: FnMut(A, &T) -> A>(xs: &Vec<T>, start: A, mut f: F) -> A {
    let mut acc = start;
    for x in xs { acc = f(acc, x); }
    acc
}
fn main() {
    let mut xs: Vec<i64> = Vec::new();
    for i in 0..2000000i64 { xs.push(i); }
    // ten rounds, each with its own factor, as in the Lume program
    let t0 = Instant::now();
    let mut total = 0i64;
    for round in 0..10i64 {
        let k = round + 2;
        let scaled = map_all(&xs, |n| n * k);
        let kept = keep(&scaled, |n| n % 3 == 0);
        total += fold_all(&kept, 0i64, |a, n| a + n);
    }
    let ms = t0.elapsed().as_millis();
    println!("{}", total);
    println!("{} ms", ms);
}
