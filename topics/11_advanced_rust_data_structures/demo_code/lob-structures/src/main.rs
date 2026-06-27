//! Educational Limit Order Book structures for HFT data pipelines.
//!
//! This demonstrates practical data structures used in real market data
//! normalization and internal state management pipelines.
//!
//! Goals:
//! - Show cache-friendly representations (flat arrays when possible).
//! - Demonstrate snapshot + delta reconstruction (the real-world pattern).
//! - Use lock-free publication of top-of-book (seqlock style).
//! - Keep it simple enough for a dedicated newcomer while using real terminology.
//!
//! Important terms (mental map):
//! - Price level: a specific price with aggregated quantity.
//! - Price-time priority: orders at the same price are matched in arrival order.
//! - Snapshot + Delta: full book image + incremental updates.
//! - LastUpdateId / sequence: used to detect gaps and apply updates correctly.
//!
//! Reference crates:
//! - orderbook-rs, rust-order-book (study for production ideas)
//! - We build an improved educational version from scratch here.

use ahash::AHashMap;
use smallvec::SmallVec;
use std::sync::atomic::{AtomicU64, Ordering};

/// A single price level.
/// In real systems this often holds more (order ids for cancellation, etc.).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PriceLevel {
    pub price: u64,   // scaled integer (e.g. price * 1e8)
    pub qty: u64,
}

/// The main order book.
/// For fixed-tick instruments (very common), we can use a flat array.
/// Here we support both modes for learning:
/// 1. HashMap-based (flexible, good for sparse books)
/// 2. Flat array (extremely cache friendly when tick size is known)
pub struct OrderBook {
    /// Symbol → (bids, asks)
    /// Using AHashMap for fast symbol lookup.
    books: AHashMap<String, (BookSide, BookSide)>,

    /// Last applied sequence (for gap detection in pipelines)
    last_seq: u64,
}

/// One side of the book (bids or asks).
/// We store levels in a SmallVec for top-N (very common) and fall back to HashMap for full depth.
pub struct BookSide {
    /// Top levels kept in a sorted SmallVec for fast top-of-book and small depth.
    /// Sorted descending for bids, ascending for asks.
    top_levels: SmallVec<[PriceLevel; 32]>,

    /// Full depth (when needed). In production you might use a BTreeMap or a more sophisticated structure.
    full_depth: AHashMap<u64, u64>,

    /// Whether we are using the flat-array optimization for this symbol
    use_flat: bool,
    flat_min_price: u64,
    flat_tick: u64,
    flat_levels: Vec<PriceLevel>, // pre-sized when use_flat == true
}

impl OrderBook {
    pub fn new() -> Self {
        Self {
            books: AHashMap::new(),
            last_seq: 0,
        }
    }

    /// Register a symbol. If you know the tick size and reasonable price range,
    /// pass them to enable the flat array optimization (highly recommended for performance).
    pub fn register_symbol(&mut self, symbol: &str, use_flat: bool, min_price: u64, tick: u64, max_levels: usize) {
        let side = BookSide {
            top_levels: SmallVec::new(),
            full_depth: AHashMap::new(),
            use_flat,
            flat_min_price: min_price,
            flat_tick: tick,
            flat_levels: if use_flat {
                vec![PriceLevel { price: 0, qty: 0 }; max_levels]
            } else {
                vec![]
            },
        };

        self.books.insert(symbol.to_string(), (side.clone(), side));
    }

    /// Apply a snapshot (full book image). This resets state for the symbol.
    pub fn apply_snapshot(&mut self, symbol: &str, bids: &[PriceLevel], asks: &[PriceLevel], seq: u64) {
        if let Some((bid_side, ask_side)) = self.books.get_mut(symbol) {
            bid_side.clear();
            ask_side.clear();

            for &level in bids {
                bid_side.insert_level(level);
            }
            for &level in asks {
                ask_side.insert_level(level);
            }
            self.last_seq = seq;
        }
    }

    /// Apply a delta (incremental update). Returns false if we detect a gap.
    pub fn apply_delta(&mut self, symbol: &str, bids: &[PriceLevel], asks: &[PriceLevel], seq: u64) -> bool {
        if seq <= self.last_seq {
            // duplicate or old
            return true;
        }
        if seq > self.last_seq + 1 {
            // gap detected — in real pipeline you would trigger snapshot recovery here
            return false;
        }

        if let Some((bid_side, ask_side)) = self.books.get_mut(symbol) {
            for &level in bids {
                bid_side.insert_level(level);
            }
            for &level in asks {
                ask_side.insert_level(level);
            }
            self.last_seq = seq;
        }
        true
    }

    /// Publish top of book using a simple atomic snapshot (seqlock style).
    /// In a real pipeline this would be called after every meaningful update.
    pub fn publish_top(&self, symbol: &str, snapshot: &AtomicTopOfBook) {
        if let Some((bids, asks)) = self.books.get(symbol) {
            let best_bid = bids.best();
            let best_ask = asks.best();

            snapshot.publish(
                best_bid.map(|l| l.price).unwrap_or(0),
                best_bid.map(|l| l.qty).unwrap_or(0),
                best_ask.map(|l| l.price).unwrap_or(0),
                best_ask.map(|l| l.qty).unwrap_or(0),
            );
        }
    }
}

impl BookSide {
    fn clear(&mut self) {
        self.top_levels.clear();
        self.full_depth.clear();
        if self.use_flat {
            for l in &mut self.flat_levels {
                *l = PriceLevel { price: 0, qty: 0 };
            }
        }
    }

    fn insert_level(&mut self, level: PriceLevel) {
        if self.use_flat {
            self.insert_flat(level);
        } else {
            self.insert_sparse(level);
        }
    }

    fn insert_flat(&mut self, level: PriceLevel) {
        if level.qty == 0 {
            // remove
            if let Some(idx) = self.flat_index(level.price) {
                self.flat_levels[idx] = PriceLevel { price: 0, qty: 0 };
            }
            return;
        }

        if let Some(idx) = self.flat_index(level.price) {
            self.flat_levels[idx] = level;

            // Keep top_levels roughly in sync for fast top-of-book
            self.refresh_top_from_flat();
        }
    }

    fn flat_index(&self, price: u64) -> Option<usize> {
        if price < self.flat_min_price {
            return None;
        }
        let idx = ((price - self.flat_min_price) / self.flat_tick) as usize;
        if idx < self.flat_levels.len() {
            Some(idx)
        } else {
            None
        }
    }

    fn refresh_top_from_flat(&mut self) {
        // Very naive — in real code you would keep a small sorted window
        self.top_levels.clear();
        for l in &self.flat_levels {
            if l.qty > 0 {
                self.top_levels.push(*l);
            }
        }
        // Sort bids descending (simplified)
        self.top_levels.sort_by(|a, b| b.price.cmp(&a.price));
        self.top_levels.truncate(32);
    }

    fn insert_sparse(&mut self, level: PriceLevel) {
        if level.qty == 0 {
            self.full_depth.remove(&level.price);
            self.top_levels.retain(|l| l.price != level.price);
            return;
        }

        self.full_depth.insert(level.price, level.qty);

        // Maintain small top_levels
        if let Some(pos) = self.top_levels.iter().position(|l| l.price == level.price) {
            self.top_levels[pos] = level;
        } else {
            self.top_levels.push(level);
        }

        // Sort and trim (bids descending)
        self.top_levels.sort_by(|a, b| b.price.cmp(&a.price));
        self.top_levels.truncate(32);
    }

    pub fn best(&self) -> Option<PriceLevel> {
        self.top_levels.first().copied()
    }
}

/// Simple atomic snapshot for top-of-book (seqlock style).
/// Used to publish latest state to readers without locks.
pub struct AtomicTopOfBook {
    bid_price: AtomicU64,
    bid_qty: AtomicU64,
    ask_price: AtomicU64,
    ask_qty: AtomicU64,
    seq: AtomicU64,
}

impl AtomicTopOfBook {
    pub fn new() -> Self {
        Self {
            bid_price: AtomicU64::new(0),
            bid_qty: AtomicU64::new(0),
            ask_price: AtomicU64::new(0),
            ask_qty: AtomicU64::new(0),
            seq: AtomicU64::new(0),
        }
    }

    pub fn publish(&self, bp: u64, bq: u64, ap: u64, aq: u64) {
        let new_seq = self.seq.load(Ordering::Relaxed) + 1;
        self.bid_price.store(bp, Ordering::Release);
        self.bid_qty.store(bq, Ordering::Release);
        self.ask_price.store(ap, Ordering::Release);
        self.ask_qty.store(aq, Ordering::Release);
        self.seq.store(new_seq, Ordering::Release);
    }

    pub fn read(&self) -> (u64, u64, u64, u64, u64) {
        loop {
            let s1 = self.seq.load(Ordering::Acquire);
            let bp = self.bid_price.load(Ordering::Acquire);
            let bq = self.bid_qty.load(Ordering::Acquire);
            let ap = self.ask_price.load(Ordering::Acquire);
            let aq = self.ask_qty.load(Ordering::Acquire);
            let s2 = self.seq.load(Ordering::Acquire);

            if s1 == s2 {
                return (bp, bq, ap, aq, s2);
            }
        }
    }
}

fn main() {
    println!("=== Educational HFT Limit Order Book Structures ===\n");

    let mut book = OrderBook::new();

    // Register with flat array optimization (recommended for performance when tick is known)
    book.register_symbol("BTCUSDT", true, 60_000_0000, 100, 1000); // min 60000.00, tick 0.01

    // Simulate snapshot
    let snapshot_bids = vec![
        PriceLevel { price: 65_000_0000, qty: 1200 },
        PriceLevel { price: 64_999_0000, qty: 800 },
    ];
    let snapshot_asks = vec![
        PriceLevel { price: 65_001_0000, qty: 1500 },
        PriceLevel { price: 65_002_0000, qty: 900 },
    ];

    book.apply_snapshot("BTCUSDT", &snapshot_bids, &snapshot_asks, 1000);

    let top = AtomicTopOfBook::new();
    book.publish_top("BTCUSDT", &top);

    let (bp, bq, ap, aq, seq) = top.read();
    println!("After snapshot: bid={}/{} ask={}/{} seq={}", bp, bq, ap, aq, seq);

    // Apply a delta
    let delta_bids = vec![PriceLevel { price: 65_000_0000, qty: 1100 }];
    book.apply_delta("BTCUSDT", &delta_bids, &[], 1001);

    book.publish_top("BTCUSDT", &top);
    let (bp, bq, _, _, _) = top.read();
    println!("After delta: best bid qty now {}", bq);

    println!("\nKey takeaway: flat arrays + SmallVec for top levels + atomic snapshots are extremely effective in real HFT data pipelines.");
}
