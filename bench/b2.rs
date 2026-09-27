use std::time::Instant;
trait Shape { fn area(&self) -> i64; }
struct Sq { side: i64 }
struct Rect { w: i64, h: i64 }
impl Shape for Sq { fn area(&self) -> i64 { self.side * self.side } }
impl Shape for Rect { fn area(&self) -> i64 { self.w * self.h } }
fn main() {
    let mut shapes: Vec<Box<dyn Shape>> = Vec::new();
    for i in 0..4000000i64 {
        if i % 2 == 0 { shapes.push(Box::new(Sq { side: i % 100 })); }
        else { shapes.push(Box::new(Rect { w: i % 50, h: 3 })); }
    }
    // ten rounds, each adding its own number, as in the Lume program
    let t0 = Instant::now();
    let mut total = 0i64;
    for round in 0..10i64 {
        for s in &shapes { total += s.area() + round; }
    }
    let ms = t0.elapsed().as_millis();
    println!("{}", total);
    println!("{} ms", ms);
}
