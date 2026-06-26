//! Memory Ordering Basics for Market Data Publication
//!
//! Dependencies (add to Cargo.toml):
//! [dependencies]
//! crossbeam-utils = "0.8"
//!
//! This file demonstrates a minimal correct publish pattern using
//! Acquire/Release and shows (via comments) what goes wrong with Relaxed.
//!
//! The pattern is exactly what you use to publish best bid/ask from a
//! feed handler thread to a strategy thread with no locks.

use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

struct AtomicTopOfBook {
    bid: AtomicU64,
    ask: AtomicU64,
    seq: AtomicU64,
}

impl AtomicTopOfBook {
    fn publish(&self, bid: u64, ask: u64) {
        // Write data with Release, then bump seq with Release.
        // Readers that see the new seq with Acquire are guaranteed
        // to see the data written before.
        let new_seq = self.seq.load(Ordering::Relaxed) + 1;
        self.bid.store(bid, Ordering::Release);
        self.ask.store(ask, Ordering::Release);
        self.seq.store(new_seq, Ordering::Release);
    }

    fn read(&self) -> (u64, u64, u64) {
        // Classic seqlock-style double check (simplified)
        let s1 = self.seq.load(Ordering::Acquire);
        let bid = self.bid.load(Ordering::Acquire);
        let ask = self.ask.load(Ordering::Acquire);
        let s2 = self.seq.load(Ordering::Acquire);

        if s1 == s2 {
            (bid, ask, s2)
        } else {
            // In production you would loop a few times or have a fallback.
            // Here we just return what we have (may be slightly torn).
            (bid, ask, s2)
        }
    }
}

fn main() {
    let tob = AtomicTopOfBook {
        bid: AtomicU64::new(0),
        ask: AtomicU64::new(0),
        seq: AtomicU64::new(0),
    };

    let tob = std::sync::Arc::new(tob);

    let writer = {
        let tob = tob.clone();
        thread::spawn(move || {
            for i in 1..=50 {
                let bid = 65_000_0000 + i;
                let ask = 65_000_0100 + i;
                tob.publish(bid, ask);
                thread::sleep(Duration::from_micros(50));
            }
        })
    };

    let reader = {
        let tob = tob.clone();
        thread::spawn(move || {
            let mut last = 0;
            for _ in 0..100 {
                let (b, a, s) = tob.read();
                if s != last {
                    println!("Reader saw seq={} bid={} ask={}", s, b, a);
                    last = s;
                }
                thread::sleep(Duration::from_micros(20));
            }
        })
    };

    writer.join().unwrap();
    reader.join().unwrap();
    println!("Done. If you change Release/Acquire to Relaxed you may see stale values.");
}
