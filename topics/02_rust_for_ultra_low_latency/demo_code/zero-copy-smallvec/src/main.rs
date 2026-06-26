//! Zero-Copy + SmallVec vs Naive Vec Cloning Demo
//!
//! This shows why the choice of data structures and ownership patterns
//! has a massive impact on hot-path latency for something that looks like
//! a Binance depth update.
//!
//! Run: cargo run --release

use bytes::Bytes;
use smallvec::SmallVec;
use std::time::Instant;

const LEVELS_PER_UPDATE: usize = 20;
const ITERATIONS: usize = 200_000;

/// A single price level (Copy type — extremely cheap to move around)
#[derive(Clone, Copy, Debug)]
struct Level {
    price: u64,
    qty: u64,
}

/// Simulated parsed depth update that borrows from the original network buffer.
/// (In reality you would parse the JSON or binary directly into such a view.)
struct ParsedDepth<'a> {
    update_id: u64,
    levels: &'a [Level],
}

/// The "book" side: we only keep the top N levels we care about.
/// Using SmallVec means we almost never allocate as long as we stay <= 16 levels.
type TopN = SmallVec<[Level; 16]>;

struct FastBook {
    top: TopN,
    last_update_id: u64,
}

impl FastBook {
    fn new() -> Self {
        Self {
            top: SmallVec::new(),
            last_update_id: 0,
        }
    }

    /// Apply update using zero-copy view. We copy only the few levels we need.
    #[inline]
    fn apply(&mut self, parsed: ParsedDepth) {
        if parsed.update_id <= self.last_update_id {
            return;
        }
        self.last_update_id = parsed.update_id;

        // Only keep the first N. SmallVec keeps this on the stack in the common case.
        self.top.clear();
        for &level in parsed.levels.iter().take(16) {
            self.top.push(level);
        }
    }
}

/// Naive version that clones the entire incoming vector every time.
struct NaiveBook {
    levels: Vec<Level>,
    last_update_id: u64,
}

impl NaiveBook {
    fn new() -> Self {
        Self {
            levels: Vec::with_capacity(LEVELS_PER_UPDATE),
            last_update_id: 0,
        }
    }

    fn apply(&mut self, parsed: ParsedDepth) {
        if parsed.update_id <= self.last_update_id {
            return;
        }
        self.last_update_id = parsed.update_id;
        // THE EXPENSIVE PART: full clone of the slice into a fresh Vec
        self.levels = parsed.levels.to_vec();
    }
}

fn main() {
    println!("=== Zero-Copy + SmallVec vs Naive Allocation Demo ===\n");

    // Simulate a realistic incoming frame (we reuse the same buffer to mimic
    // how a real system would receive bytes::Bytes from the network).
    let raw_levels: Vec<Level> = (0..LEVELS_PER_UPDATE)
        .map(|i| Level {
            price: 65_000_0000 - (i as u64) * 50,
            qty: 100 + (i as u64 % 5) * 10,
        })
        .collect();

    let frame: Bytes = {
        // In real life the frame would come from the socket.
        // Here we just create one Bytes buffer.
        let mut v = Vec::new();
        for l in &raw_levels {
            v.extend_from_slice(&l.price.to_le_bytes());
            v.extend_from_slice(&l.qty.to_le_bytes());
        }
        Bytes::from(v)
    };

    // Reinterpret the bytes as our levels (fake parsing, but same cost profile)
    // We take a slice view of the data we "received".
    let parsed_view: &[Level] = unsafe {
        // SAFETY: in a real parser you would validate + transmute carefully.
        // This is just to simulate the zero-copy view you get from bytes::Bytes.
        std::slice::from_raw_parts(
            frame.as_ptr() as *const Level,
            LEVELS_PER_UPDATE,
        )
    };

    // ------------------------------------------------------------------
    // FAST PATH
    // ------------------------------------------------------------------
    let mut fast = FastBook::new();
    let start = Instant::now();

    for i in 0..ITERATIONS {
        let parsed = ParsedDepth {
            update_id: i as u64,
            levels: parsed_view,
        };
        fast.apply(parsed);
    }

    let fast_time = start.elapsed();

    // ------------------------------------------------------------------
    // NAIVE PATH
    // ------------------------------------------------------------------
    let mut naive = NaiveBook::new();
    let start = Instant::now();

    for i in 0..ITERATIONS {
        let parsed = ParsedDepth {
            update_id: i as u64,
            levels: parsed_view,
        };
        naive.apply(parsed);
    }

    let naive_time = start.elapsed();

    println!("Iterations: {}", ITERATIONS);
    println!("Fast (zero-copy view + SmallVec): {:?}", fast_time);
    println!("Naive (full Vec clone every time): {:?}", naive_time);
    println!();
    println!("Speedup: {:.2}x", naive_time.as_nanos() as f64 / fast_time.as_nanos() as f64);
    println!();
    println!("In a real Binance feed handler the naive version would");
    println!("cause allocator pressure and jitter exactly when volatility spikes.");
    println!("The fast version stays almost entirely on the stack + cache.");
}
