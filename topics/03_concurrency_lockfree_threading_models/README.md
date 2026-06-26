# 03 — Concurrency, Lock-Free, and Threading Models

## Why This Matters in Ultra-Low-Latency Crypto Trading

A market maker must consume market data and produce orders while other parts of the system react to fills, manage risk, and reconnect streams. If any of these activities can block or slow down the critical path, you lose the edge and can also lose money through bad state.

The central challenge is **safe sharing of data between threads without introducing locks on the hot path** and without data races or torn reads that corrupt your view of the book or your inventory.

## What Problem Does It Solve?

- Multiple threads need to see the "latest" best bid/ask or order book snapshot.
- The producer (feed handler) must never be blocked by consumers (strategy, OMS, telemetry).
- You need to reason about **when** a write becomes visible to other threads (memory ordering).
- Traditional mutexes introduce unpredictable latency under contention and are a common source of tail latency.

## How Does It Solve the Problem?

### The Traffic Light Analogy for Memory Ordering

Imagine a busy intersection with no traffic lights. Cars from different directions can enter at the same time and crash.

Memory ordering is the set of traffic lights and signs the CPU and compiler must obey:

- `Relaxed`: "No rules — go whenever you want." Fast, but you can get surprising reorderings.
- `Acquire` / `Release`: "If you see the green light (Release), everything before the light on the other thread happened." This is the most common safe pattern for publishing data.
- `SeqCst`: "Everyone agrees on a total order." Strongest but slowest. Rarely needed.

For publishing best bid/ask from a feed thread:

```rust
// Writer (feed)
best_bid.store(new_bid, Ordering::Release);
seq.store(new_seq, Ordering::Release);

// Reader (strategy)
let s = seq.load(Ordering::Acquire);
let price = best_bid.load(Ordering::Acquire);
```

If the reader sees the new seq, it is guaranteed to also see the new price (on architectures that matter for us).

### Lock-Free Publication Patterns Useful in Trading

1. **Single atomic values** — best bid, best ask, last seq.
2. **Seqlock** (reader spins on sequence) — great for small structs (top-of-book snapshot).
3. **SPSC / MPMC ring buffers** (crossbeam, rtrb, or custom) — for intents, fills, commands.
4. **Hazard pointers / epoch reclamation** when you need to safely reclaim memory that readers may still be looking at (advanced).

### Threading Model Recommendation for a Binance MM

- **Feed thread(s)**: pinned to isolated core(s). Only job = ingest + parse + publish.
- **Strategy thread(s)**: separate core. Reads published data, computes desired quotes.
- **OMS thread**: owns order state machines, rate limiting, signing, and sending.
- **Background pool**: reconnects, snapshots, metrics, admin, Python bridge, etc.

The feed thread uses the most lock-free publication possible. Strategy and OMS communicate via bounded lock-free queues.

Never put a general-purpose async runtime on the true hot path unless you have measured that the jitter is acceptable for your latency budget.

### Why Many Systems Avoid Full Async Runtimes on the Critical Path

Tokio, async-std etc. are fantastic for I/O-bound services. They introduce:
- Task scheduling overhead
- Waker allocations in some paths
- Potential shared state inside the runtime

For sub-100µs tick-to-trade you usually want explicit threads + explicit queues + explicit pinning.

## Applicability in Real Trading Systems

### How to Apply It Effectively

- Start with atomic primitives for anything that fits in a few words (prices, sequence numbers, flags).
- Use a proven channel crate (`crossbeam-channel`, `flume`, or a carefully reviewed custom ring) for anything more complex.
- Pin hot threads and set scheduling policy when possible (`SCHED_FIFO` or `SCHED_RR` on Linux).
- Measure the effect of every synchronization choice with real load.

### Key Gotchas and Pitfalls

- Using `Relaxed` when you actually needed `Acquire/Release` → torn or stale views of the book.
- Holding a lock while doing anything that might allocate or syscall.
- Assuming that "lock free" automatically means "wait free". Lock-free can still have retry loops that create tail latency.
- Sharing a single `Arc<Mutex<Book>>` between feed and strategy.
- Not accounting for false sharing (two atomics on the same cache line being written by different cores).

### When to Use vs Not

Use lock-free + dedicated threads when p99 latency and correctness under burst matter.

Use a well-tuned async runtime when developer velocity and moderate latency are the priority, or when the work is I/O heavy rather than CPU hot-path.

## Visual Understanding

### Recommended Threading Model SDD (ASCII)

```
┌────────────────────┐         atomic / seqlock          ┌────────────────────┐
│  Feed Thread(s)    │ ─────────────────────────────────▶ │  Strategy Thread   │
│  (pinned core)     │         best bid/ask + seq        │  (pinned core)     │
│                    │                                    │                    │
│  parse + OB update │                                    │  pricing + risk    │
└─────────┬──────────┘                                    └─────────┬──────────┘
          │ (fills via user stream)                                 │
          │                                                         │ intent
          ▼                                                         ▼
┌──────────────────────────────────────────────────────┐   ┌────────────────────┐
│  Fill → Inventory (atomic or very short critical sec) │   │  OMS Thread        │
└──────────────────────────────────────────────────────┘   │  (pinned)          │
                                                           │  state machine     │
                                                           │  send orders       │
                                                           └────────────────────┘
```

## Demo Code

### 03a — Memory Ordering Basics (single file)

`demo_code/memory_ordering_basics.rs`

Shows a classic publish pattern and what can go wrong with wrong orderings.

### 03b — Simple Multi-Threaded Demo with Atomics + Channel

`demo_code/lockfree-pub/` — a small project demonstrating feed publishing top-of-book to two consumer threads with no locks on the publish side.

Run the project with `cargo run --release`.

## Interview Focus

1. Explain Acquire/Release with a trading example.
2. Design a lock-free way to publish best bid/ask + a sequence number so readers never block writers.
3. What are the risks of using a general async runtime on your market data thread?
4. How do you prevent false sharing between hot atomics written by different cores?
5. Walk through a reconnect scenario — which parts must be lock-free vs which can use normal synchronization.
