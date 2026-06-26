//! Minimal educational order book that handles synthetic snapshot + deltas.
//! Focus: correct sequencing + fast top-of-book query.
//! Not a full production book.

use std::collections::BTreeMap;
use std::time::Instant;

type Price = u64;
type Qty = u64;

#[derive(Default)]
struct Side {
    levels: BTreeMap<Price, Qty>,
}

impl Side {
    fn apply(&mut self, price: Price, qty: Qty) {
        if qty == 0 {
            self.levels.remove(&price);
        } else {
            self.levels.insert(price, qty);
        }
    }

    fn best(&self) -> Option<(Price, Qty)> {
        self.levels.iter().next().map(|(&p, &q)| (p, q))
    }
}

struct OrderBook {
    bids: Side,
    asks: Side,
    last_update_id: u64,
}

impl OrderBook {
    fn new() -> Self {
        Self {
            bids: Side::default(),
            asks: Side::default(),
            last_update_id: 0,
        }
    }

    fn apply_snapshot(&mut self, update_id: u64, bids: &[(Price, Qty)], asks: &[(Price, Qty)]) {
        self.last_update_id = update_id;
        self.bids.levels.clear();
        self.asks.levels.clear();
        for &(p, q) in bids {
            self.bids.levels.insert(p, q);
        }
        for &(p, q) in asks {
            self.asks.levels.insert(p, q);
        }
    }

    /// Apply delta. Must be called with strictly increasing update ids (or handle gaps).
    fn apply_delta(&mut self, update_id: u64, bids: &[(Price, Qty)], asks: &[(Price, Qty)]) {
        if update_id <= self.last_update_id {
            return; // stale or duplicate
        }
        self.last_update_id = update_id;
        for &(p, q) in bids {
            self.bids.apply(p, q);
        }
        for &(p, q) in asks {
            self.asks.apply(p, q);
        }
    }

    fn best_bid(&self) -> Option<(Price, Qty)> {
        self.bids.best()
    }
    fn best_ask(&self) -> Option<(Price, Qty)> {
        self.asks.best()
    }
}

fn main() {
    let mut book = OrderBook::new();

    // Snapshot
    book.apply_snapshot(
        1000,
        &[(6500000000, 1200), (6499900000, 500)],
        &[(6500100000, 900), (6500200000, 300)],
    );

    println!("After snapshot best: bid={:?} ask={:?}", book.best_bid(), book.best_ask());

    // Deltas
    let start = Instant::now();
    for i in 0..50_000u64 {
        book.apply_delta(
            1001 + i,
            &[(6500000000 - i % 100, 1000)],
            &[(6500100000 + i % 50, 800)],
        );
    }
    let dt = start.elapsed();
    println!("Applied 50k deltas in {:?}", dt);
    println!("Final best: bid={:?} ask={:?}", book.best_bid(), book.best_ask());
    println!("last_update_id = {}", book.last_update_id);
}
