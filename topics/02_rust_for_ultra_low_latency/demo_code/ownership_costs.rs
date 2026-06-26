//! Demo: Moves, Borrows, Clones, and Why They Matter for Latency
//!
//! Add to your Cargo.toml:
//! [dependencies]
//! smallvec = { version = "1", features = ["const_generics"] }
//!
//! How to run:
//!   cargo run --release --example ownership_costs   (if placed in examples/)
//!   or just compile this file together with a minimal main that calls the functions.
//!
//! Teaching goal:
//! Show that moving is free, borrowing is free, cloning large things is expensive,
//! and that the compiler forces you to be explicit — which is exactly what you want
//! when every microsecond of jitter can cost money.

use std::time::Instant;

#[derive(Clone, Copy, Debug)]
struct PriceLevel {
    price: u64, // scaled * 1e8
    qty: u64,
}

// A "depth update" that borrows the raw data (simplified).
// In reality you would have a proper zero-copy parser.
struct BorrowedDepth<'a> {
    symbol: &'a str,
    levels: &'a [PriceLevel],
}

// An owned version you would create if you need to keep the data longer than the buffer.
#[derive(Clone)]
struct OwnedDepth {
    symbol: String,
    levels: Vec<PriceLevel>,
}

fn process_borrowed(depth: BorrowedDepth) -> u64 {
    // Pure read-only work. No ownership transferred.
    depth.levels.iter().map(|l| l.qty).sum()
}

fn process_owned(depth: OwnedDepth) -> u64 {
    depth.levels.iter().map(|l| l.qty).sum()
}

fn main() {
    println!("=== Rust Ownership & Borrowing Cost Demo ===\n");

    // Simulate a realistic top-of-book snapshot size (Binance depth can be much larger)
    let levels: Vec<PriceLevel> = (0..100)
        .map(|i| PriceLevel {
            price: 65_000_0000 - i * 100,
            qty: 10 + (i as u64 % 7),
        })
        .collect();

    // ------------------------------------------------------------------
    // 1. BORROW (the good hot-path pattern)
    // ------------------------------------------------------------------
    let start = Instant::now();
    let mut total: u64 = 0;
    for _ in 0..100_000 {
        let borrowed = BorrowedDepth {
            symbol: "BTCUSDT",
            levels: &levels,
        };
        total += process_borrowed(borrowed);
    }
    let borrow_time = start.elapsed();

    // ------------------------------------------------------------------
    // 2. MOVE (also free, but you lose the original)
    // ------------------------------------------------------------------
    let mut moved_levels = levels.clone(); // just setup
    let start = Instant::now();
    for _ in 0..100_000 {
        // move the whole vec into the function (cheap move of the pointer + length)
        let owned = OwnedDepth {
            symbol: "BTCUSDT".to_string(),
            levels: std::mem::take(&mut moved_levels),
        };
        total += process_owned(owned);
        // put something back so we can loop (artificial)
        moved_levels = vec![PriceLevel { price: 0, qty: 0 }; 100];
    }
    let move_time = start.elapsed();

    // ------------------------------------------------------------------
    // 3. CLONE (the silent killer on hot paths)
    // ------------------------------------------------------------------
    let start = Instant::now();
    for _ in 0..10_000 {
        // Every iteration allocates a brand new Vec and copies 100 elements.
        let owned = OwnedDepth {
            symbol: "BTCUSDT".to_string(),
            levels: levels.clone(),
        };
        total += process_owned(owned);
    }
    let clone_time = start.elapsed();

    println!("Total (just to prevent optimization away): {}", total);
    println!();
    println!("100k iterations using BORROW (hot path style): {:?}", borrow_time);
    println!("100k iterations using MOVE  (also fine):         {:?}", move_time);
    println!(" 10k iterations using CLONE (disaster if in hot path): {:?}", clone_time);
    println!();
    println!("Lesson: a single accidental .clone() of a 100-element Vec");
    println!("        inside the per-update loop is often more expensive");
    println!("        than all the actual business logic combined.");
}
