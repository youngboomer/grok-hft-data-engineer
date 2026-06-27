# 11 — Advanced Data Structures and Algorithms in Rust for HFT Data Management and Pipelines

## Why This Matters in HFT Data Trading Roles

In HFT (High-Frequency Trading) and market-making roles focused on **data management and data pipelines**, you are responsible for ingesting, normalizing, storing, replaying, and streaming massive volumes of tick data (market data updates, trades, order book deltas) with microsecond or nanosecond precision.

A single delayed or corrupted update can lead to:
- Stale order books → bad quotes → adverse selection (getting picked off).
- Lost messages during bursts → incorrect positions or risk calculations.
- High latency in the data plane → strategy cannot react in time.

Modern HFT data roles require building or optimizing **pipelines** that move data from exchange feeds → internal hot path → risk/analytics → storage/replay.

Rust is increasingly the language of choice for the performance-critical parts of these pipelines because it delivers C++-level speed with memory safety and excellent concurrency primitives.

This topic builds directly on the foundations from topics 03 (concurrency), 04 (memory/zero-copy), and 07 (order book basics). We go deeper into production-grade implementations that you can actually use or extend in real data systems.

## What Problem Does It Solve?

Traditional high-level approaches (pure Python with Pandas, or even naive Rust with std::collections) fail under HFT realities:

- **Bursts**: Binance or other venues can emit thousands of updates per second per symbol during volatility. Your structures must handle this without allocation jitter or lock contention.
- **Correctness under failure**: You must reconstruct accurate state from snapshots + deltas, handle gaps, duplicates, and out-of-order data.
- **Low tail latency + high throughput**: p99 latency matters. Every allocation, lock, or cache miss shows up in your numbers.
- **Data management needs**: You need efficient ways to:
  - Publish latest state to multiple consumers (strategy, risk, logging).
  - Maintain history for replay and backtesting.
  - Aggregate (imbalance, top-N, volume profiles) quickly.
  - Move data between hot path (Rust) and analytical layers (Python/Polars/Arrow).

The core challenge is designing **cache-friendly, lock-free or low-contention, allocation-free-on-hot-path** data structures that are simple enough to reason about but powerful enough for real production use.

## How Does It Solve the Problem?

We focus on a small set of battle-tested patterns that appear everywhere in real HFT data pipelines. We explain them with analogies, then show real Rust implementations (from scratch + references to crates).

### 1. Ring Buffers (Circular Buffers) for Streaming Data

**Analogy**: Imagine a circular train track with a fixed number of carriages. The producer (exchange feed) puts data into the next empty carriage. Consumers (strategy, risk engine) read from their carriage and move forward. No one waits for locks if the ring is sized correctly.

**Why important**:
- Core building block for lock-free inter-thread or inter-process communication.
- Used in feed handlers, between parser and book updater, between hot path and logging.

**Key concepts** (keep these terms in your mental map):
- **SPSC** = Single Producer, Single Consumer (simplest, highest performance).
- **SPMC / MPSC / MPMC** = multiple sides.
- **Sequence / Cursor / Head / Tail**: Atomic counters that track positions without locks.
- **Wrap-around**: When you reach the end, you go back to the beginning.
- **Padding / Cache-line alignment**: Prevent false sharing (two counters on same 64-byte cache line fighting).

**From-scratch Rust implementation sketch** (simplified SPSC for learning):

We will implement a basic version in the demo code. The key is using `AtomicUsize` for positions and a pre-allocated array.

Reference existing crates (use these in real projects, study their source):
- `rtrb` — realtime-safe SPSC ring buffer (excellent for audio/HFT-like use cases).
- `ringbuf` — flexible SPSC/MPMC with async support.
- `crossbeam::queue::ArrayQueue` or `SegQueue` — good starting points.
- `kanal` or `flume` — higher-level channels built on similar ideas.

Improved from-scratch version usually focuses on:
- Explicit cache-line padding (`#[repr(align(64))]`)
- No `unsafe` where possible (or minimal, well-documented unsafe)
- Support for zero-copy reads (return references into the buffer)
- Backpressure signaling (what happens when full?)

### 2. Lock-Free Order Book Representations (Beyond Basic)

From topic 07 we had a simple `BTreeMap` + `Vec`. For real pipelines we need better.

**Common production structures** (comparison table later):
- **Flat array / Vec indexed by price tick** (best when tick size is fixed and range is known — extremely cache friendly).
- **Sorted Vec of price levels** + binary search for updates.
- **HashMap<Price, Level> + separate sorted structure** for full depth.
- Per-level: `Vec<Order>` or intrusive linked list (for price-time priority matching).

**Important algorithms**:
- **Price-time priority matching**: Orders at same price matched in arrival order.
- **Incremental update**: Apply `bid/ask` deltas without rebuilding the whole book.
- **Snapshot + delta reconstruction**: The most common real-world pattern.

**Rust-specific wins**:
- Use `SmallVec<[Level; N]>` for top levels (stays on stack).
- `bytes::Bytes` or `&[u8]` for zero-copy when feeding from network.
- Atomics + seqlock for publishing the "latest view" of the book to many readers.

We will implement a clean, educational version that supports:
- Add / cancel / match
- Snapshot + incremental apply
- Lock-free publish of top-of-book + simple imbalance

Reference crates:
- `orderbook-rs`, `rust-order-book` on crates.io (study them).
- Many HFT shops roll their own for full control.

### 3. Seqlocks and Atomic Snapshots for State Publishing

**Analogy**: The librarian writes a new edition of the newspaper (the current best bid/ask or full top-N). Readers check the edition number before and after reading. If the number changed, they retry. The writer never waits.

This is the classic way to publish small-to-medium state (top of book, latest imbalance, last trade) from one writer thread to many reader threads with almost zero contention.

Implementation uses `Acquire`/`Release` ordering + a sequence counter.

### 4. Slab Allocators and Object Pools

**Analogy**: Instead of asking the town hall for a new piece of paper every time you need one (global allocator), you reserve a whole stack of paper at the start of the day and hand out sheets from your own pile. When done, you put them back in the pile for reuse.

Critical for avoiding allocator contention and jitter in hot paths.

Rust crates:
- `bumpalo` (bump allocator — great for temporary batches).
- `slab`, custom pre-allocated pools using `Vec` + free list.

We will build a simple object pool for `Order` structs.

### 5. Supporting Algorithms for Data Pipelines

- **Heavy hitters / top-K** at price levels (for imbalance signals).
- **Range sum queries** on price levels (Fenwick tree / Binary Indexed Tree or Segment Tree — useful for volume profiles).
- **Deduplication and gap detection** using sequence numbers.
- **Deterministic replay** — being able to replay a stream of events and get identical state.

## Applicability in Real Trading Systems

### How to Apply It Effectively
- Start every hot data structure with pre-allocation.
- Measure everything (use the tools from topic 09).
- Keep the "writer" side extremely simple (usually one thread or carefully coordinated).
- Use Rust's type system to make invalid states unrepresentable (e.g. separate types for `Snapshot` vs `Delta`).
- For Python integration: expose via PyO3/Maturin using Arrow or zero-copy buffers.

### Key Gotchas and Pitfalls
- False sharing on atomics (two hot counters on the same cache line).
- Assuming "it worked on my laptop" — test under burst load and on the target CPU.
- Forgetting about NUMA when you have multiple sockets.
- Using `Vec::push` in the hot path without capacity.
- Over-engineering: sometimes a well-sized `Vec` + binary search beats a fancy tree.

### When to Use vs When NOT to Use + Alternatives
- Use lock-free ring buffers when you have clear producer/consumer rates and can afford bounded loss or backpressure.
- Use Disruptor-style when you have one producer feeding many consumers with low latency.
- For very high cardinality symbol lookup → good hashmap (ahash, or fxhash).
- Alternative for some analytical parts: Polars (Rust) or DataFusion — they already use excellent columnar structures.

## Visual Understanding: Data Flow & Structure Diagrams (ASCII)

### Typical HFT Data Pipeline (Simplified)

```
Exchange Feed (WebSocket / Multicast / FIX)
        │
        ▼  (zero-copy parse)
[ Rust Parser Thread ] ──▶ Ring Buffer (lock-free)
                                │
          ┌─────────────────────┼─────────────────────┐
          ▼                     ▼                     ▼
   [Order Book Updater]   [Imbalance Calculator]   [Logger / Persister]
          │                     │                     │
          ▼                     ▼                     ▼
   Atomic Snapshot      Atomic Snapshot         File / DB (cold path)
   (seqlock)            (seqlock)
          │
          ▼
   Strategy / Risk Readers (many)
```

### Ring Buffer Layout (Conceptual)

```
[ Slot 0 | Slot 1 | Slot 2 | ... | Slot N-1 ]
   ^head (producer writes here)
         ^tail (consumer reads here)
When producer advances head, consumer can follow.
```

## Data Structures / Primitives Comparison

| Structure                  | Latency (publish) | Predictability | Contention | Allocation on hot path | Good for in HFT Data Pipelines                  | Rust Crate / From-scratch |
|----------------------------|-------------------|----------------|------------|------------------------|--------------------------------------------------|-----------------------------|
| `Mutex<Vec<T>>`            | High              | Poor           | High       | Possible               | Almost never on hot path                         | std::sync                  |
| Seqlock + small struct     | Very low (~20-50ns)| Excellent     | None       | No                     | Top-of-book, last trade, imbalance               | From scratch + atomics     |
| SPSC Ring Buffer           | Low               | Excellent      | None       | No (pre-alloc)         | Feed → book, book → strategy                     | rtrb, ringbuf, from scratch|
| Disruptor-style (multi)    | Low               | Excellent      | Very low   | No                     | One feed → many consumers (risk + strategy + log)| disruptor-rs + custom      |
| Flat price array (LOB)     | Very low          | Excellent      | Low        | No                     | Fixed-tick instruments (very cache friendly)     | From scratch               |
| Sorted Vec + binary search | Low               | Very good      | Low        | No                     | Full depth when sparsity is moderate             | SmallVec + binary_search   |
| Slab / Object Pool         | Very low          | Excellent      | Low        | No (after init)        | Reusing Order structs                            | bumpalo + custom pool      |

## Demo Code & Examples

We will provide:

1. A from-scratch SPSC ring buffer (with padding and zero-copy read support).
2. An improved LOB using flat array + hash for symbol lookup, supporting snapshot + delta.
3. A simple object pool.
4. Python side: using `polars` or Arrow to consume the same data for analytical pipelines (comparison).

All demos live in `demo_code/` subfolders and are runnable with `cargo run --release`.

See the demo folders for detailed READMEs and suggested experiments (measure under burst load, compare against referenced crates, add a second consumer, etc.).

## Further Reading & Resources

- LMAX Disruptor paper (classic, still highly relevant).
- "Mechanical Sympathy" blog posts by Martin Thompson.
- Crates: `rtrb`, `ringbuf`, `disruptor-rs`, `bumpalo`, `bytes`, `zerocopy`.
- "Rust for Rustaceans" (Jon Gjengset) — chapter on low-level concurrency.
- Real-world: Databento, Jump Trading, Jane Street tech talks on data pipelines.

## Interview Focus for This Topic

1. "Walk me through how you would design a lock-free way to publish the latest order book view from a single updater thread to multiple strategy and risk threads."
2. "Compare using a `BTreeMap` vs a flat array for a limit order book. When would each be appropriate?"
3. "What is false sharing and how would you prevent it in a high-throughput ring buffer?"
4. "Implement (or describe) a basic SPSC ring buffer. What happens when the buffer is full?"
5. "How would you reconstruct a correct order book from a snapshot + sequence of deltas while handling gaps?"
6. "Why do many HFT systems prefer pre-allocated slabs over the global allocator in the hot path?"
7. "You see high tail latency only when multiple consumers are reading. What structures or patterns might be causing this?"

## Potential Gaps & Nice-to-Have Topics (to be expanded in later topics)

This topic focuses on in-memory, single-machine, CPU-centric structures.

Gaps we will address in follow-up topics in this branch:
- Full end-to-end pipeline architecture (topic 10).
- Real tick data storage, compression, and replay engines (topic 12).
- Complete lock-free multi-producer / multi-consumer streaming (topic 13).
- Analytical pipelines using Arrow + Polars Rust (topic 14).
- Hardware acceleration concepts (FPGA, SmartNICs, DPDK) — explained at high level even without full implementation.
- Integration with risk/position data flows.
- Observability specific to data pipelines (nanosecond histograms, gap detection metrics).

Nice-to-haves for deeper expertise:
- Custom binary wire formats (SBE from scratch).
- NUMA-aware allocation and thread pinning.
- Backtesting data engines that can replay at accelerated speeds.
- Exactly-once semantics in low-latency contexts.
- Coalescing / conflation strategies for very high update rates.

This gives a newcomer a clear mental map while still going deep enough to be useful in real interviews and production work.
