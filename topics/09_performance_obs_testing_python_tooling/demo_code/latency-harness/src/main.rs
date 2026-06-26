//! Simple atomic timing harness that records tick-to-"decision" latencies
//! into a file that the accompanying Python script can analyze.
//!
//! Run: cargo run --release
//! Then run the Python analyzer on latency.log

use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

const SAMPLES: usize = 100_000;

fn main() {
    let mut latencies_ns: Vec<u64> = Vec::with_capacity(SAMPLES);

    let mut last = Instant::now();

    for i in 0..SAMPLES {
        // Simulate "market data arrival"
        let arrival = Instant::now();

        // Simulate tiny amount of work (strategy decision)
        let mut x = i as u64;
        for _ in 0..8 {
            x = x.wrapping_mul(0x123456789abcdef);
        }

        let decision = Instant::now();
        let dt = decision.duration_since(arrival).as_nanos() as u64;
        latencies_ns.push(dt);

        // Occasionally simulate a burst
        if i % 3000 == 0 {
            std::thread::sleep(Duration::from_micros(50));
        }
        last = decision;
    }

    // Write log
    let f = File::create("latency.log").unwrap();
    let mut w = BufWriter::new(f);
    for &v in &latencies_ns {
        writeln!(w, "{}", v).unwrap();
    }
    println!("Wrote {} samples to latency.log", latencies_ns.len());
    println!("Run the Python analyzer next.");
}
