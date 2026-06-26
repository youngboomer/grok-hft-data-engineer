//! Very small demo contrasting repeated allocation vs pre-allocated buffer reuse.
//! This is the spirit of arena thinking even without a full arena crate.

use bytes::Bytes;
use std::time::Instant;

const N: usize = 100_000;

#[derive(Clone, Copy)]
struct Level { price: u64, qty: u64 }

fn main() {
    println!("=== Pre-allocated buffer reuse vs repeated allocation ===\n");

    let template: Vec<Level> = (0..64).map(|i| Level { price: 1000 + i, qty: 10 }).collect();

    // Naive: new Vec every time
    let start = Instant::now();
    let mut sum = 0u64;
    for i in 0..N {
        let mut v = Vec::with_capacity(64);
        v.extend_from_slice(&template);
        if i % 2 == 0 { sum += v[0].price; }
    }
    println!("Naive Vec each time: {:?}", start.elapsed());

    // Good: one reusable buffer (arena-like thinking)
    let start = Instant::now();
    let mut reusable: Vec<Level> = Vec::with_capacity(64);
    for i in 0..N {
        reusable.clear();
        reusable.extend_from_slice(&template);
        if i % 2 == 0 { sum += reusable[0].price; }
    }
    println!("Reusable buffer   : {:?}", start.elapsed());

    println!("\nSum to keep optimizer honest: {}", sum);
    println!("The reusable version avoids allocator traffic on every update.");
}
