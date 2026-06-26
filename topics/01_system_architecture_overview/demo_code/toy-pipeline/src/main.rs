//! Toy Tick-to-Trade Pipeline Skeleton
//!
//! This demo shows the core architectural pattern used in real ultra-low-latency
//! Binance market makers:
//!
//! 1. Dedicated FEED thread (hot path): receives synthetic "market data",
//!    updates a tiny "order book view", and publishes best bid/ask using ONLY
//!    atomics + a sequence number. Zero locks, zero allocations on the publish path.
//!
//! 2. STRATEGY thread: reads the latest published quote non-blockingly,
//!    makes a fake quoting decision that incorporates current inventory,
//!    and sends an "intent" to the OMS via a bounded channel.
//!
//! 3. OMS thread: receives intents, simulates rate limiting + order "sending",
//!    and occasionally generates fills that flow back to inventory.
//!
//! The key lesson: the feed thread can NEVER be slowed down by a slow strategy
//! or a slow OMS. The publication mechanism (atomics) makes this mechanically true.
//!
//! Run with: cargo run --release
//!
//! Try these experiments:
//!   - Increase BURST_SIZE to 5000+
//!   - Add a sleep(1ms) inside strategy and observe feed stats do not degrade
//!   - Change the atomic ordering and see what breaks (relaxed vs acquire/release)
//!
//! Teaching note for the 10-year-old brain:
//! Think of the "best bid/ask" as a library book that the librarian (feed thread)
//! updates on a special shelf. Readers (strategy) can look at the shelf any time.
//! They don't have to wait for the librarian, and the librarian never waits for readers.
//! The sequence number is like the "edition number" stamped on the book so readers
//! can tell if they got a fresh copy or an old one.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use crossbeam_channel::{bounded, Receiver, Sender};

// -----------------------------------------------------------------------------
// Shared "Best Quote" published lock-free from the feed thread
// Using a tiny struct packed into two atomics for simplicity.
// In production you might use a seqlock or a 128-bit atomic on x86_64.
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Default)]
struct BestQuote {
    bid: u64,      // price * 1e8
    ask: u64,
    seq: u64,      // monotonically increasing
}

struct AtomicBestQuote {
    bid: AtomicU64,
    ask: AtomicU64,
    seq: AtomicU64,
}

impl AtomicBestQuote {
    fn new() -> Self {
        Self {
            bid: AtomicU64::new(0),
            ask: AtomicU64::new(0),
            seq: AtomicU64::new(0),
        }
    }

    /// Hot path publish. Called only from the dedicated feed thread.
    /// This is the "write the new edition number on the book and put it on the shelf".
    #[inline]
    fn publish(&self, bid: u64, ask: u64) {
        // Important ordering: we write data first, then bump seq.
        // Readers do seq, then data, then check seq again (seqlock style).
        // Here we do a simplified version that is still very useful.
        let new_seq = self.seq.load(Ordering::Relaxed) + 1;
        self.bid.store(bid, Ordering::Release);
        self.ask.store(ask, Ordering::Release);
        self.seq.store(new_seq, Ordering::Release);
    }

    /// Non-blocking read for consumers.
    /// Returns the quote + the seq at the time of read.
    #[inline]
    fn read(&self) -> (BestQuote, u64) {
        let seq1 = self.seq.load(Ordering::Acquire);
        let bid = self.bid.load(Ordering::Acquire);
        let ask = self.ask.load(Ordering::Acquire);
        let seq2 = self.seq.load(Ordering::Acquire);

        let quote = BestQuote { bid, ask, seq: seq2 };

        // If seq changed between reads, the value may be torn.
        // In real seqlock you would spin until you get a clean read.
        // For this demo we just return it + the seq so caller can decide.
        (quote, seq2.max(seq1))
    }
}

// -----------------------------------------------------------------------------
// Inventory (very simplified). In reality this would be much more careful.
// We use atomics here because the OMS thread is the only writer and strategy
// only needs approximate latest value for quoting decisions.
// -----------------------------------------------------------------------------

struct Inventory {
    // net position in base asset (positive = long)
    position: AtomicU64, // stored as u64 with bias for simplicity (real code would use i64 + careful logic)
}

impl Inventory {
    fn new() -> Self {
        Self {
            position: AtomicU64::new(1_000_000), // start with 1.0 BTC expressed as 1e6 units
        }
    }

    #[inline]
    fn get(&self) -> u64 {
        self.position.load(Ordering::Relaxed)
    }

    // OMS thread calls this when a fill arrives
    fn apply_fill(&self, qty: u64) {
        // In real life: signed arithmetic, consider quote side, fees, etc.
        // Here we just pretend every fill reduces our long by qty.
        let current = self.position.load(Ordering::Relaxed);
        let new = current.saturating_sub(qty);
        self.position.store(new, Ordering::Relaxed);
    }
}

// -----------------------------------------------------------------------------
// Simple "intent" that strategy sends to OMS
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
struct QuoteIntent {
    bid_price: u64,
    ask_price: u64,
    bid_qty: u64,
    ask_qty: u64,
    decision_seq: u64, // links back to the market data snapshot that caused it
}

// -----------------------------------------------------------------------------
// Synthetic market data generator (stands in for Binance WS feed)
// -----------------------------------------------------------------------------

struct SyntheticFeed {
    next_bid: u64,
    next_ask: u64,
    update_count: u64,
}

impl SyntheticFeed {
    fn new() -> Self {
        Self {
            next_bid: 65_000_0000, // 65000.00
            next_ask: 65_000_0100,
            update_count: 0,
        }
    }

    /// Simulate a burst of depth updates. Returns (bid, ask) for the "best".
    fn next_update(&mut self) -> (u64, u64) {
        self.update_count += 1;

        // Simulate small price walks + occasional bigger moves (volatility)
        if self.update_count % 17 == 0 {
            self.next_bid = self.next_bid.saturating_sub(50);
            self.next_ask = self.next_ask.saturating_add(30);
        } else if self.update_count % 31 == 0 {
            self.next_bid = self.next_bid.saturating_add(80);
            self.next_ask = self.next_ask.saturating_sub(40);
        } else {
            self.next_bid = self.next_bid.wrapping_add(1);
            self.next_ask = self.next_ask.wrapping_add(1);
        }

        (self.next_bid, self.next_ask)
    }
}

// -----------------------------------------------------------------------------
// Main
// -----------------------------------------------------------------------------

const BURST_SIZE: usize = 2000;
const NUM_BURSTS: usize = 8;

fn main() {
    println!("=== Toy Tick-to-Trade Pipeline Demo ===");
    println!("Feed thread publishes best quotes with atomics (no locks).");
    println!("Strategy reads non-blocking and produces intents.");
    println!("OMS receives intents and occasionally produces fills.");
    println!();

    let atomic_quote = Arc::new(AtomicBestQuote::new());
    let inventory = Arc::new(Inventory::new());

    // Bounded channel between strategy and OMS.
    // In real life this would be sized to absorb bursts without dropping
    // critical intents, but still bounded to avoid unbounded memory growth.
    let (intent_tx, intent_rx): (Sender<QuoteIntent>, Receiver<QuoteIntent>) = bounded(256);

    // Channels for simple "fill" feedback (OMS -> inventory simulation)
    let (fill_tx, fill_rx): (Sender<u64>, Receiver<u64>) = bounded(64);

    // Shared counters so we can print stats without locks in hot path
    let feed_updates = Arc::new(AtomicU64::new(0));
    let strategy_decisions = Arc::new(AtomicU64::new(0));
    let oms_orders = Arc::new(AtomicU64::new(0));

    // ---------------- FEED THREAD (HOT PATH) ----------------
    let feed_quote = Arc::clone(&atomic_quote);
    let feed_count = Arc::clone(&feed_updates);
    let feed_handle = thread::Builder::new()
        .name("feed-hot".into())
        .spawn(move || {
            let mut feed = SyntheticFeed::new();
            let mut last_publish = Instant::now();

            for burst in 0..NUM_BURSTS {
                for _ in 0..BURST_SIZE {
                    let (bid, ask) = feed.next_update();

                    // === THIS IS THE HOT PATH ===
                    // One publish = two stores + one seq bump.
                    // No allocation. No lock. No syscall.
                    feed_quote.publish(bid, ask);

                    let c = feed_count.fetch_add(1, Ordering::Relaxed) + 1;

                    // Very occasional status (cold)
                    if c % 5000 == 0 {
                        let now = Instant::now();
                        let dt = now.duration_since(last_publish).as_micros();
                        println!(
                            "[FEED] {} updates | last burst dt ~{}µs | bid={} ask={}",
                            c, dt, bid, ask
                        );
                        last_publish = now;
                    }
                }

                // Simulate Binance burst gap then quiet period
                if burst < NUM_BURSTS - 1 {
                    thread::sleep(Duration::from_micros(300));
                }
            }
            println!("[FEED] thread finished. Total updates: {}", feed_count.load(Ordering::Relaxed));
        })
        .unwrap();

    // ---------------- STRATEGY THREAD ----------------
    let strat_quote = Arc::clone(&atomic_quote);
    let strat_inv = Arc::clone(&inventory);
    let strat_count = Arc::clone(&strategy_decisions);
    let strat_tx = intent_tx.clone();
    let strategy_handle = thread::Builder::new()
        .name("strategy".into())
        .spawn(move || {
            let mut last_seq = 0u64;

            for _ in 0..(NUM_BURSTS * BURST_SIZE) {
                let (quote, seq) = strat_quote.read();

                // Only act on genuinely new data (avoid busy-spinning on same quote)
                if seq <= last_seq {
                    // In real systems you might spin with a pause or use a notification
                    // mechanism. Here we just yield a tiny bit.
                    thread::yield_now();
                    continue;
                }
                last_seq = seq;

                let position = strat_inv.get();

                // Extremely simplified quoting logic:
                // - skew our quotes based on inventory (if long, bid a bit less aggressively)
                // - add tiny "edge"
                let skew = if position > 1_200_000 { 30 } else { 0 };

                let intent = QuoteIntent {
                    bid_price: quote.bid.saturating_sub(50 + skew),
                    ask_price: quote.ask.saturating_add(50 + skew),
                    bid_qty: 1000,
                    ask_qty: 1000,
                    decision_seq: seq,
                };

                // Send to OMS. If the channel is full we drop the intent (backpressure).
                // In production you would have better backpressure signaling.
                let _ = strat_tx.try_send(intent);

                strat_count.fetch_add(1, Ordering::Relaxed);

                // Simulate "thinking" time (very small). In real code this is the pricing model.
                if strat_count.load(Ordering::Relaxed) % 2000 == 0 {
                    thread::sleep(Duration::from_micros(10));
                }
            }

            println!("[STRATEGY] decisions: {}", strat_count.load(Ordering::Relaxed));
        })
        .unwrap();

    // ---------------- OMS THREAD ----------------
    let oms_rx = intent_rx;
    let oms_count = Arc::clone(&oms_orders);
    let oms_inv = Arc::clone(&inventory);
    let oms_fill_tx = fill_tx;
    let oms_handle = thread::Builder::new()
        .name("oms".into())
        .spawn(move || {
            let mut local_seq = 0u64;
            let mut fills_sent = 0u64;

            while let Ok(intent) = oms_rx.recv() {
                local_seq += 1;

                // Pretend rate limit check + HMAC signing + TCP send.
                // All of this would be carefully measured in a real OMS.
                if local_seq % 7 == 0 {
                    // Simulate a fill coming back from Binance
                    let fill_qty = 120;
                    let _ = oms_fill_tx.try_send(fill_qty);
                    fills_sent += 1;

                    // Update inventory (this is the feedback loop)
                    oms_inv.apply_fill(fill_qty);
                }

                oms_count.fetch_add(1, Ordering::Relaxed);

                if oms_count.load(Ordering::Relaxed) % 3000 == 0 {
                    println!(
                        "[OMS] {} orders processed | last intent seq={} | fills generated so far={}",
                        oms_count.load(Ordering::Relaxed),
                        intent.decision_seq,
                        fills_sent
                    );
                }
            }

            println!("[OMS] done. Total orders: {}", oms_count.load(Ordering::Relaxed));
        })
        .unwrap();

    // ---------------- BACKGROUND "fill applier" simulation ----------------
    // In reality the feed handler would receive fills on the private userDataStream
    // and publish them into inventory. We simulate the effect here.
    let bg_inv = Arc::clone(&inventory);
    let bg_handle = thread::Builder::new()
        .name("background-fills".into())
        .spawn(move || {
            while let Ok(_qty) = fill_rx.recv() {
                // Already applied by OMS in this toy example.
                // Here we could do reconciliation, logging, or cross-venue hedge calc.
                let pos = bg_inv.get();
                if pos < 200_000 {
                    println!("[BG] WARNING: inventory getting low: {}", pos);
                }
            }
        })
        .unwrap();

    // Wait for feed to finish (it drives the demo)
    let _ = feed_handle.join();
    // Give the other threads a moment to drain
    thread::sleep(Duration::from_millis(50));

    // Close intent channel so strategy/oms can exit cleanly
    drop(intent_tx);

    let _ = strategy_handle.join();
    let _ = oms_handle.join();
    // fill_tx was moved into the OMS thread; channel closes when OMS drops its sender
    let _ = bg_handle.join();

    println!();
    println!("=== Summary ===");
    println!("Feed updates     : {}", feed_updates.load(Ordering::Relaxed));
    println!("Strategy decisions: {}", strategy_decisions.load(Ordering::Relaxed));
    println!("OMS orders        : {}", oms_orders.load(Ordering::Relaxed));
    println!("Final inventory   : {}", inventory.get());
    println!();
    println!("Key takeaway: the feed thread never waited for strategy or OMS.");
    println!("The atomic publication mechanism + bounded channel enforces separation.");
}
