//! Lock-free publication of top-of-book from one feed thread
//! to multiple consumer threads using only atomics for the hot path.
//!
//! The feed thread is the only writer. Consumers never block it.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use crossbeam_channel::bounded;

struct AtomicQuote {
    bid: AtomicU64,
    ask: AtomicU64,
    seq: AtomicU64,
}

impl AtomicQuote {
    fn new() -> Self {
        Self {
            bid: AtomicU64::new(0),
            ask: AtomicU64::new(0),
            seq: AtomicU64::new(0),
        }
    }

    #[inline(always)]
    fn publish(&self, bid: u64, ask: u64) {
        let new_seq = self.seq.load(Ordering::Relaxed) + 1;
        self.bid.store(bid, Ordering::Release);
        self.ask.store(ask, Ordering::Release);
        self.seq.store(new_seq, Ordering::Release);
    }

    #[inline(always)]
    fn read(&self) -> (u64, u64, u64) {
        loop {
            let s1 = self.seq.load(Ordering::Acquire);
            let bid = self.bid.load(Ordering::Acquire);
            let ask = self.ask.load(Ordering::Acquire);
            let s2 = self.seq.load(Ordering::Acquire);
            if s1 == s2 {
                return (bid, ask, s2);
            }
            // rare contention - spin a little
            std::hint::spin_loop();
        }
    }
}

fn main() {
    let quote = Arc::new(AtomicQuote::new());
    let (tx, rx) = bounded::<(u64, u64, u64)>(128);

    // Feed thread (hot)
    let q1 = quote.clone();
    let feed = thread::Builder::new().name("feed".into()).spawn(move || {
        for i in 0..50_000u64 {
            q1.publish(65_000_0000 + i % 1000, 65_000_0100 + i % 1000);
            if i % 10_000 == 0 {
                thread::sleep(Duration::from_micros(10));
            }
        }
    }).unwrap();

    // Two strategy-like consumers
    let mut consumers = vec![];
    for id in 0..2 {
        let q = quote.clone();
        let tx = tx.clone();
        let h = thread::Builder::new().name(format!("consumer-{}", id)).spawn(move || {
            let mut last = 0;
            let start = Instant::now();
            let mut reads = 0u64;
            while start.elapsed() < Duration::from_millis(300) {
                let (b, a, s) = q.read();
                if s != last {
                    last = s;
                    reads += 1;
                    let _ = tx.try_send((b, a, s));
                }
            }
            println!("consumer-{} did {} fresh reads", id, reads);
        }).unwrap();
        consumers.push(h);
    }

    drop(tx); // close channel for main

    let _ = feed.join();
    for c in consumers {
        let _ = c.join();
    }

    // Drain a few values
    let mut count = 0;
    while rx.try_recv().is_ok() {
        count += 1;
    }
    println!("Main observed {} published quotes via channel", count);
    println!("Demo complete. Feed thread was never blocked by readers.");
}
