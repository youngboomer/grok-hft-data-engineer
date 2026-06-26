//! Example: Pre-allocated sorted levels for top of book (allocation-free update path).
//! Demonstrates SmallVec + binary search for updates.

use smallvec::SmallVec;
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Level {
    price: u64,
    qty: u64,
}

const MAX_LEVELS: usize = 32;

fn apply_delta(levels: &mut SmallVec<[Level; MAX_LEVELS]>, price: u64, qty: u64) {
    // Simple linear scan + insert for illustration. Real version would be more optimized.
    if let Some(pos) = levels.iter().position(|l| l.price == price) {
        if qty == 0 {
            levels.remove(pos);
        } else {
            levels[pos].qty = qty;
        }
        return;
    }
    if qty == 0 {
        return;
    }
    // Insert in sorted order (very naive for demo)
    let pos = levels.iter().position(|l| l.price < price).unwrap_or(levels.len());
    levels.insert(pos, Level { price, qty });
    if levels.len() > MAX_LEVELS {
        levels.pop();
    }
}

fn main() {
    let mut book: SmallVec<[Level; MAX_LEVELS]> = SmallVec::new();

    let start = Instant::now();
    for i in 0..100_000u64 {
        let price = 65000_0000 - (i % 50) * 100;
        apply_delta(&mut book, price, 1000 + (i % 7));
    }
    let dt = start.elapsed();

    println!("Applied 100k updates in {:?}", dt);
    println!("Final top levels: {:?}", &book[..book.len().min(5)]);
    println!("(In real code you would also publish atomically after each meaningful change)");
}
