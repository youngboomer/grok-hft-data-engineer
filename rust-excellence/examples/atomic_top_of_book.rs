//! Self-contained example: Atomic top-of-book publication
//! Add to Cargo.toml:
//! [dependencies]
//! crossbeam-utils = "0.8"

use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

struct TopOfBook {
    bid: AtomicU64,
    ask: AtomicU64,
    seq: AtomicU64,
}

fn main() {
    let tob = TopOfBook {
        bid: AtomicU64::new(0),
        ask: AtomicU64::new(0),
        seq: AtomicU64::new(0),
    };

    // Writer thread (simulates feed)
    let writer_tob = &tob;
    let w = thread::spawn(move || {
        for i in 0..100_000u64 {
            let bid = 65000_0000 + (i % 1000);
            let ask = bid + 100;
            let new_seq = writer_tob.seq.load(Ordering::Relaxed) + 1;
            writer_tob.bid.store(bid, Ordering::Release);
            writer_tob.ask.store(ask, Ordering::Release);
            writer_tob.seq.store(new_seq, Ordering::Release);
            if i % 10000 == 0 {
                thread::sleep(Duration::from_micros(10));
            }
        }
    });

    // Reader threads (simulate strategies)
    let mut readers = vec![];
    for id in 0..3 {
        let r_tob = &tob;
        let r = thread::spawn(move || {
            let mut last = 0u64;
            let mut seen = 0u64;
            for _ in 0..200_000 {
                let s1 = r_tob.seq.load(Ordering::Acquire);
                let bid = r_tob.bid.load(Ordering::Acquire);
                let ask = r_tob.ask.load(Ordering::Acquire);
                let s2 = r_tob.seq.load(Ordering::Acquire);
                if s1 == s2 && s2 > last {
                    last = s2;
                    seen += 1;
                    if seen % 5000 == 0 {
                        println!("reader {} saw bid={} ask={}", id, bid, ask);
                    }
                }
            }
            println!("reader {} finished, saw {} updates", id, seen);
        });
        readers.push(r);
    }

    w.join().unwrap();
    for r in readers {
        r.join().unwrap();
    }
    println!("Done. Try changing Release/Acquire to Relaxed and see if you get weird values.");
}
