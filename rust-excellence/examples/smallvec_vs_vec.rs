//! Compare SmallVec vs Vec clone cost in a hot loop.
//! cargo run --release

use smallvec::SmallVec;
use std::time::Instant;

#[derive(Clone, Copy)]
struct Level {
    price: u64,
    qty: u64,
}

const N: usize = 24;
const ITERS: usize = 200_000;

fn main() {
    let levels: Vec<Level> = (0..N).map(|i| Level { price: 1000 + i, qty: 10 }).collect();

    // Vec clone every time
    let start = Instant::now();
    let mut sum = 0u64;
    for _ in 0..ITERS {
        let v = levels.clone();
        sum += v[0].price;
    }
    println!("Vec clone: {:?}", start.elapsed());

    // SmallVec (stays on stack)
    let start = Instant::now();
    for _ in 0..ITERS {
        let mut sv: SmallVec<[Level; 32]> = SmallVec::new();
        sv.extend_from_slice(&levels);
        sum += sv[0].price;
    }
    println!("SmallVec extend: {:?}", start.elapsed());

    println!("Sum to prevent optimization: {}", sum);
}
