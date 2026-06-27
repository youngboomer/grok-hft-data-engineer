//! Educational multi-consumer lock-free pipeline inspired by LMAX Disruptor.
//! Uses crossbeam for the ring buffer foundation.
//!
//! In real HFT this pattern is used to publish normalized market data
//! to strategy, risk, and logging consumers without the producer blocking.

use crossbeam::channel;
use std::thread;
use std::time::Duration;

#[derive(Clone, Debug)]
struct MarketEvent {
    seq: u64,
    symbol: String,
    price: u64,
    qty: u64,
}

fn main() {
    let (tx, rx) = channel::bounded::<MarketEvent>(1024);

    // Producer (simulated feed)
    let producer = thread::spawn(move || {
        for i in 0..10000u64 {
            let ev = MarketEvent {
                seq: i,
                symbol: "BTCUSDT".to_string(),
                price: 65000_0000 + (i % 100),
                qty: 10,
            };
            if tx.send(ev).is_err() {
                break;
            }
            if i % 1000 == 0 {
                thread::sleep(Duration::from_micros(10));
            }
        }
    });

    // Multiple consumers (strategy, risk, etc.)
    let mut consumers = vec![];
    for id in 0..3 {
        let r = rx.clone();
        let h = thread::spawn(move || {
            let mut count = 0u64;
            while let Ok(ev) = r.recv() {
                count += 1;
                if count % 2000 == 0 {
                    println!("Consumer {} processed up to seq {}", id, ev.seq);
                }
            }
            println!("Consumer {} done, total {}", id, count);
        });
        consumers.push(h);
    }

    drop(rx);
    producer.join().unwrap();
    for c in consumers {
        let _ = c.join();
    }

    println!("Disruptor-style pipeline demo complete. In real impl use dedicated ring buffer + wait strategies.");
}
