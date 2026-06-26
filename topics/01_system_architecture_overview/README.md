# 01 — System Architecture Overview

## Why This Matters in Ultra-Low-Latency Crypto Trading

In a Binance market maker you are paid to be the best bid or best offer most of the time while keeping inventory risk under control. The only way to do that profitably at scale is to react to every meaningful market data update faster and more reliably than your competitors.

The difference between a good system and a great one is almost never "we used a slightly faster CPU". It is almost always:
- Whether the hot path is completely isolated from everything slow
- Whether state is published in a way that never blocks the producer
- Whether a reconnect or a burst of updates can corrupt your view of the world
- Whether fills flow back into pricing fast enough that you don't over-hedge or leave toxic quotes up

A well-architected system makes the correct thing the easy thing. A poorly architected one forces you to fight the design every time volatility hits.

## What Problem Does It Solve?

Real trading systems must solve these simultaneously:

1. **Extreme speed on the critical path** (socket → parse → decision → wire)
2. **Predictable latency** (low p99 and p99.9, not just good averages)
3. **Correctness under partial failure and message loss**
4. **Multiple independent data sources** (public market data + private user stream + cross-exchange feeds)
5. **Feedback loops** that must not add jitter to the hot path (fills → inventory → quote adjustment)

Most "fast" codebases fail in production because they mix cold-path concerns (logging, config reload, metrics export, reconnection) into the same threads and data structures that must stay fast.

## How Does It Solve the Problem?

### Core Principle: Ruthless Separation of Hot and Cold Paths

The **hot path** is the only thing that must finish before the next market data frame arrives:
- Receive frame from kernel
- Parse with zero or one allocation
- Update local order book or top-of-book view
- Publish the new best bid/ask (or full snapshot view) in a way consumers can read without blocking the publisher
- Make a quoting decision or pass an intent to the next stage

Everything else is **cold** (or warm) and must live on different threads, different cores, or different timing:
- Reconnecting a dropped WebSocket
- Fetching a full order book snapshot
- Writing structured logs or metrics
- Reconciling positions with the exchange
- Responding to control-plane commands

### Threading and Runtime Model

Recommended starting model for a multi-symbol Binance market maker (10-30 symbols):

- 1 dedicated feed thread per major venue (or one very tight thread if symbols are few)
- 1 strategy/pricing thread (or small pool if symbols have very different logic)
- 1 OMS / order management thread that owns all order state machines and network writes for orders
- 1 background thread (or async runtime) for everything else

Why not one big async runtime on the hot path?
Because Tokio (or any executor) introduces task scheduling, waker overhead, and potential contention. For the absolute lowest and most predictable latency we often prefer:
- Thread-per-core with explicit pinning
- SPSC or MPSC lock-free queues (or atomic snapshot + sequence number for top-of-book)
- No locks on the publish path of market data

### Latency Budget Example (Realistic for Cloud Binance MM)

Target tick-to-trade (market data frame fully received → order on the wire) under load:

- Receive + parse depth frame: 2-8 µs
- Order book update + publish best bid/ask: 1-5 µs
- Strategy decision (simple quote model + inventory skew): 2-10 µs
- OMS: build + rate limit check + sign + send: 10-30 µs
- **Total hot path target**: < 60-80 µs p99 in quiet periods, < 200-300 µs p99 during extreme bursts

Anything that occasionally adds 1-2 ms (a lock convoy, an allocator pause, a page fault, a context switch onto a busy core) destroys the edge.

### The Critical Feedback Loop: Fills → Inventory → Strategy

```
Binance WS userDataStream
        │
        ▼
Feed handler publishes fill
        │
        ▼
Inventory atomically updates (or short lock)
        │
        ▼
Strategy reads latest inventory on next decision cycle
        │
        ▼
Quotes are skewed or pulled
```

This loop must be fast and safe. If the strategy reads a stale inventory value for too long you over-quote. If the inventory update blocks the feed handler you start dropping market data.

## Applicability in Real Trading Systems

### How to Apply It Effectively

1. **Draw the DFD and SDD on a whiteboard first.** Identify every place where data crosses a thread or process boundary.
2. **Choose your publication mechanism per data type**:
   - Top-of-book + sequence number: often best as a small atomic struct (or two atomics + seqlock-style).
   - Full order book deltas or complex decisions: SPSC queue with pre-allocated slots or a lock-free ring.
3. **Pin threads to isolated cores** (when you have them). Use `taskset`, `isolcpus`, or `pthread_setaffinity_np`.
4. **Make reconnect and snapshot logic live completely outside the hot thread**. The hot thread only consumes clean frames.
5. **Design your order ID and client order ID scheme for idempotency** from day one.
6. **Measure the real path**, not synthetic loops. Instrument from frame arrival timestamp (as close to NIC as you can) to send timestamp.

### Key Gotchas and Pitfalls (Especially Latency & Correctness)

- Putting logging or metrics inside the hot path "just for now".
- Using a `Mutex` or `RwLock` to publish best bid/ask — a single slow consumer can block the entire feed.
- Sharing a single allocator across hot and cold threads.
- Reusing the same WebSocket connection for both public data and private order placement without understanding back-pressure.
- "We'll just use async everywhere, it's fast enough." Async is great for cold path and connection management. On the extreme hot path it can hide sources of jitter.
- Not modeling the exact sequence numbers and gap detection rules for Binance depth streams → silent book corruption.
- Treating inventory as a simple `AtomicI64` without thinking about partial fills and multi-venue netting.

### When to Use vs When NOT to Use + Alternatives

**Use this strict hot/cold + dedicated thread model when**:
- You care about p99/p99.9 more than p50.
- You will run 10+ symbols with non-trivial quoting logic.
- You plan to add cross-venue hedging.

**Consider a simpler model (single async runtime + careful task priorities) when**:
- You are building a research or low-frequency system.
- You have < 5 symbols and very simple quoting.
- You are still validating the strategy logic and want faster iteration.

**Better alternatives for extreme cases**:
- Kernel bypass (DPDK + Solarflare/FPGA) when even 10-20 µs of kernel networking is too much (rare for most crypto shops).
- Multiple processes + shared memory rings when you need language or failure isolation.

## Visual Understanding: Data Flow & Structure Diagrams (ASCII)

### Simplified Hot Path DFD (single symbol)

```
[Binance TCP] ──▶ [recv buffer] ──▶ [parse depthUpdate]
                                           │
                                           ▼
                                 [OrderBook::apply_delta]
                                           │
                                           ▼
                              [publish_best(AtomicSnapshot)]
                                           │
                    ┌──────────────────────┼──────────────────────┐
                    ▼                      ▼                      ▼
             Strategy reads          OMS sees new book      Telemetry (cold)
             (non-blocking)          (via queue)            (sampling)
```

### Thread Ownership SDD (recommended for 10-30 symbols)

```
┌───────────────┐       ┌───────────────┐       ┌───────────────┐
│  Feed Thread  │──────▶│ Strategy Th.  │──────▶│   OMS Thread  │
│  (1 core)     │  SPSC │  (1 core)     │  SPSC │   (1 core)    │
│               │  or   │               │  or   │               │
│  - WS client  │ atomic│  - Pricing    │ atomic│  - State m/c  │
│  - Parser     │       │  - Inventory  │       │  - Signing    │
│  - Book       │       │    view       │       │  - Sending    │
└───────┬───────┘       └───────┬───────┘       └───────┬───────┘
        │                       │                       │
        │ (user data fills)     │                       │
        └───────────────────────┴───────────────────────┘
                                │
                                ▼
                        Background Thread(s)
                        - Reconnect
                        - Snapshot
                        - Metrics
                        - REST polling (if any)
```

## Data Structures / Primitives Comparison

| Primitive                  | Latency (publish) | Predictability | Contention Risk | Hot Path Fit | Trading Example                     | Better Alternative When |
|----------------------------|-------------------|----------------|-----------------|--------------|-------------------------------------|-------------------------|
| `Mutex<BestQuote>`         | High (occasional) | Poor           | High            | Bad          | Publishing top of book              | Atomic snapshot or seqlock |
| `AtomicU64` (price*1e8)    | ~10-30 ns         | Excellent      | None            | Excellent    | Best bid or last seqnum             | - |
| Seqlock + small struct     | ~20-50 ns         | Excellent      | Very low        | Excellent    | Full top-of-book + seq              | Lock-free queue for complex data |
| Crossbeam SPSC             | ~50-150 ns        | Very good      | None (if uncontended) | Good    | Intent queue feed→OMS              | Bounded ring for fixed size |
| Tokio channel (async)      | 200ns - several µs| Fair           | Medium          | Poor         | Background work                     | Dedicated thread + lock-free |
| `Arc<Mutex<Book>>`         | Very high         | Terrible       | High            | Never        | Full book sharing                   | Per-thread copy or lock-free snapshot |

## Demo Code & Examples

### 01 — Toy Tick-to-Trade Pipeline Skeleton

Location: `demo_code/toy-pipeline/`

This is a small self-contained Cargo project that demonstrates:
- Three threads (feed, strategy, oms) with explicit separation
- Lock-free publication of best bid/ask using atomics + sequence number
- A bounded intent queue from strategy to OMS
- Synthetic "market data" bursts to show behavior under load
- Very rough latency measurement between decision stages

Run it:

```bash
cd topics/01_system_architecture_overview/demo_code/toy-pipeline
cargo run --release
```

Experiment ideas:
- Increase burst size and watch how the queue behaves.
- Add a slow consumer (artificial sleep in strategy) and observe whether feed thread slows down (it shouldn't).
- Add a `println!` inside the feed hot loop and measure the damage to tail latency.

See `demo_code/toy-pipeline/README.md` for more details and suggested modifications.

## Further Reading & Resources (Topic-Specific)

- "Latency Numbers Every Programmer Should Know" (updated versions) — to internalize the cost of every abstraction.
- Binance Spot WebSocket API docs — depth stream format, how updateId and lastUpdateId work for snapshots vs deltas.
- "The Tail at Scale" (Google) — why tail latency dominates user experience (and trading P&L).
- LMAX Disruptor paper — classic on lock-free producer/consumer rings (many ideas still apply).
- "Designing Data-Intensive Applications" (Kleppmann) — chapters on streaming and consistency.

## Interview Focus for This Topic

1. **"Walk me through the full path a Binance depth update takes in your system from socket to order being sent."**  
   Strong answer shows clear thread ownership, publication mechanism, and exactly where allocations and locks are (or aren't).

2. **"How do you make sure a slow strategy consumer never stalls your market data feed?"**  
   Shows understanding of non-blocking publication and back-pressure strategy.

3. **"Design the reconnect + snapshot logic so you never have a corrupted book and never lose a fill."**  
   Must cover gap detection, snapshot sequencing, and fill correlation.

4. **"What is your latency budget and how do you enforce it?"**  
   Shows they think in distributions, not averages, and have measurement baked in.

5. **"Why might you deliberately choose not to use async/await on the hot path?"**  
   Reveals whether they understand the sources of jitter in executors.

6. **"How does inventory information flow back into quoting without adding jitter?"**  
   Tests the feedback loop understanding.

7. **"You see p99 tick-to-trade jump from 80µs to 1.2ms only during high volatility. What do you investigate first?"**  
   Practical debugging mindset.
