# grok-hft-data-engineer

Ultra-low-latency data engineering and systems for high-frequency crypto trading in Rust.

**Target Job Description Focus**: Design, build, and operate ultra-low-latency automated trading systems for crypto market making and cross-exchange hedging. The core objective is **tick-to-trade optimization**: the time from a market data frame arriving on the socket to an order leaving the wire.

This repository is a complete, production-minded study and reference guide. It teaches the engineering discipline required to build systems that are:
- Extremely fast on the hot path
- Predictable (low tail latency / jitter)
- Correct under burst load and failure modes
- Maintainable and testable

Everything is grounded in real Binance usage (WebSocket depth/aggTrade/user data streams, snapshot + delta order books, HMAC-signed REST orders, heartbeats, reconnect semantics).

## The Tick-to-Trade Reality

In crypto market making:
- Binance can emit hundreds of depth updates per second per symbol during normal times, and thousands during volatility spikes.
- A single missed or delayed update can leave your local book stale → bad quotes → adverse selection or inventory blow-up.
- A single duplicated order (bad reconnect logic) can cause massive losses.
- Every allocation, lock, syscall, or branch on the critical path adds jitter that shows up in p99/p99.9 tail latency — the exact numbers that decide whether your MM is profitable or not.

The only reliable way to win is ruthless separation of the **hot path** from everything else, zero-cost abstractions, pre-allocation, careful memory ordering, and obsessive measurement.

## High-Level End-to-End Data Flow Diagram (DFD)

```
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│                                  MARKET DATA (Binance)                                       │
│  ws://stream.binance.com:9443/ws/btcusdt@depth@100ms   +   aggTrade   +   userDataStream     │
└─────────────────────────────────────────────────────────────────────────────────────────────┘
                                            │
                                            ▼
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│  FEED HANDLER THREAD (dedicated core, pinned)                                                 │
│  ┌────────────────────┐   ┌──────────────────────┐   ┌───────────────────────────────┐     │
│  │ TCP Socket / TLS   │──▶│ Zero-copy frame      │──▶│ Parse (depth/aggTrade)        │     │
│  │ (TCP_NODELAY)      │   │ buffer (bytes::Bytes)│   │ No alloc on hot path          │     │
│  └────────────────────┘   └──────────────────────┘   └───────────────┬───────────────┘     │
│                                                                      │                       │
│  Reconnect + gap detection + dedup (cold path)                       ▼                       │
│                                                           ┌──────────────────────┐          │
│                                                           │ Update OrderBook     │          │
│                                                           │ (lock-free publish   │          │
│                                                           │  best bid/ask)       │          │
│                                                           └───────────┬──────────┘          │
└───────────────────────────────────────────────────────────────────────┼─────────────────────┘
                                                                        │ (atomic snapshot or SPSC)
                                                                        ▼
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│  STRATEGY THREAD(S) (separate core(s))                                                        │
│  ┌──────────────────────────────┐   ┌───────────────────────────────┐                       │
│  │ Read latest top-of-book      │──▶│ Compute quotes / imbalance    │                       │
│  │ (relaxed or acquire)         │   │ + inventory skew + signal     │                       │
│  └──────────────────────────────┘   └───────────────┬───────────────┘                       │
│                                                     │                                       │
│                                          (decision) ▼                                       │
│                                        ┌────────────────────────┐                           │
│                                        │ Build order intent     │                           │
│                                        │ (price, qty, side)     │                           │
│                                        └────────────┬───────────┘                           │
└─────────────────────────────────────────────────────┼───────────────────────────────────────┘
                                                      │ (bounded SPSC or lock-free queue)
                                                      ▼
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│  OMS / EXECUTION THREAD (or same as strategy for minimal symbols)                             │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────────┐  ┌─────────────────┐ │
│  │ Rate limiter     │─▶│ Order lifecycle  │─▶│ Sign (HMAC) + send   │─▶│ TCP to REST /   │ │
│  │ (token bucket)   │  │ state machine    │  │ via signed REST      │  │ WS order entry  │ │
│  └──────────────────┘  └──────────────────┘  └──────────────────────┘  └─────────────────┘ │
│                                                                                        ▲    │
│  Fills come back via userDataStream  ──────────────────────────────────────────────────┘    │
│  (separate WS connection, feed handler publishes to inventory atomically)                   │
└─────────────────────────────────────────────────────────────────────────────────────────────┘
                                                      │
                                                      ▼
                                        ┌──────────────────────────┐
                                        │ Inventory Manager        │
                                        │ (per-symbol + total USD) │
                                        │ + Hedge decision queue   │
                                        └──────────────────────────┘
```

**Key boundaries**:
- Market data never blocks on strategy.
- Strategy never blocks on order submission.
- Fills flow back asynchronously into shared (carefully synchronized) inventory state.
- Reconnects, snapshots, logging, metrics export happen off the hot path.

## System Structure Design Diagram (SDD)

```
┌────────────────────────────────────────────────────────────────────────────┐
│                              PROCESS / CONTAINER                            │
│                                                                             │
│  ┌───────────────────────┐     ┌─────────────────────────────┐            │
│  │  Feed Handler Thread  │     │  Strategy + Pricing Thread  │            │
│  │  (core 2, SCHED_FIFO) │     │  (core 3)                   │            │
│  │                       │     │                             │            │
│  │  • Binance WS client  │────▶│  • Latest OB view (atomic)  │            │
│  │  • Parser (zero-copy) │     │  • Quote model              │            │
│  │  • OrderBook          │     │  • Risk checks              │            │
│  │    (lock-free pub)    │     │  • Intent generation        │            │
│  └───────────────────────┘     └──────────────┬──────────────┘            │
│                                               │ (SPSC queue)              │
│                                               ▼                           │
│  ┌────────────────────────────┐  ┌────────────────────────────┐         │
│  │  OMS / Networking Thread   │  │  Background / Cold Path    │         │
│  │  (core 4)                  │  │  (normal priority)         │         │
│  │                            │  │                            │         │
│  │  • Order state machine     │  │  • Reconnect manager       │         │
│  │  • REST signer             │  │  • Snapshot fetcher        │         │
│  │  • Rate limiter            │  │  • Metrics / telemetry     │         │
│  │  • Fill → inventory update │  │  • Position reconciliation │         │
│  └────────────────────────────┘  └────────────────────────────┘         │
│                                                                             │
│  Shared (careful):                                                          │
│    • Atomic best bid/ask + seqnum per symbol                                │
│    • Inventory (atomic or lock-protected with very short critical sections) │
│    • Order ID → state map (for dedup + lifecycle)                           │
└────────────────────────────────────────────────────────────────────────────┘
```

Hot path threads avoid the global allocator, locks, and complex runtime work.

## Mapping This Repository to Real Job Requirements

| Topic | What You Will Master | Direct JD Relevance |
|-------|----------------------|---------------------|
| 01 System Architecture Overview | End-to-end pipeline, hot/cold separation, latency budgets, feedback loops | System design for MM + hedging engines |
| 02 Rust for Ultra Low Latency | Ownership, zero-cost abstractions, panic=abort, release tuning | Writing the actual low-jitter code |
| 03 Concurrency & Lock-Free Models | Atomics, memory ordering, SPSC, thread pinning, async vs sync on hot path | Safe concurrent market data + strategy |
| 04 Memory & Zero-Copy Structures | Arenas, pre-allocation, `bytes`, custom layouts | No jitter from allocations under load |
| 05 Linux Networking & Sockets | TCP_NODELAY, recvmsg, epoll tuning, userspace | Talking to Binance with minimal latency |
| 06 Exchange Protocols (WS/FIX/SBE) | Binance WS client, auth, heartbeats, binary protocols | Real exchange connectivity |
| 07 Market Data & Order Book | Snapshot + delta, consistent book, fast top-N | The heart of any market maker |
| 08 OMS, Strategy & Hedging | State machines, inventory, cross-venue logic | Risk and execution correctness |
| 09 Perf, Obs & Tooling | Profiling, histograms, Python analysis, golden tests | Proving your system is fast and correct |

## Recommended Study Order

1. Read **this** README completely.
2. Go through topics in order: 01 → 09.
3. For each topic:
   - Read the full `README.md` (Why → What → How → Applicability + diagrams + tables).
   - Study and run the demo(s).
   - Try the suggested experiments.
4. Work through `interview-prep/` in parallel or after core topics.
5. Use `resources/curated_reading.md` for deep dives.

## Additional Branches for Deeper Preparation

This repository has two specialized branches that go beyond the core 9 topics:

### `interview-questionnaire` branch
A dedicated deep-dive for interview readiness and advanced knowledge:

- **HFT Terminology Glossary** — 30+ terms with definitions, system impact, and analogies
- **Data Structures** — Common + HFT-specific (order books, seqlocks, ring buffers, etc.) with trade-off tables
- **Core Questions** — In-depth answers for concurrency, market data, system design
- **Tricky Questions** — Advanced scenarios that separate strong candidates
- **Practice Scenarios** — Whiteboard-style exercises
- **Quick Reference Cheat Sheet**

Check it out with:
```bash
git checkout interview-questionnaire
```

### `rust-excellence` branch
Focused on becoming exceptional at writing high-performance, correct Rust for trading systems:

- Ownership & lifetimes in real trading contexts
- Atomics and memory ordering mastery
- Allocation-free hot path techniques
- Zero-copy patterns with `bytes`
- Performance tooling (perf, asm, histograms, etc.)
- Runnable focused examples

```bash
git checkout rust-excellence
```

These branches follow the same teaching style as the main content.


## How to Use the Demos

Most simple demos are single `.rs` files. Add the listed dependencies to a Cargo.toml and run.

Complex demos are self-contained mini Cargo projects:

```bash
cd topics/07_market_data_orderbook_management/demo_code/binance-depth-book
cargo run --release
```

Always prefer `--release`. Many examples pin cores or use specific allocators that only behave correctly in optimized builds.

## Crypto Realities You Must Internalize

- **Bursts are normal**: Depth update rates can jump 10-50x in seconds. Your hot path must not degrade.
- **Snapshots + deltas**: You will miss messages. You must be able to rebuild a correct book from snapshot + replay without double-applying.
- **User data stream**: Fills, partial fills, cancellations, and order status arrive on a completely separate authenticated WS connection. You must correlate them reliably to your local order state.
- **Reconnect without duplicates**: Both WS and order placement need idempotency keys + sequence numbers.
- **Rate limits**: 1200 requests/minute on many endpoints. Your OMS must never blow this up, even on reconnect storms.
- **Cross-venue hedging**: Inventory on Binance affects what you quote on Deribit or Bybit. The feedback loop is the product.
- **Cloud vs colo**: In cloud you fight noisy neighbors + virtualization jitter. The techniques here still apply and are even more necessary.

## Engineering Philosophy of This Repo

- **Hot path is sacred**. Everything else can be slower or use more resources.
- **Correctness under load > micro-benchmark numbers**. A system that is fast in a quiet test but corrupts state or explodes p99 at 3am is worse than useless.
- **Measure what matters**: tick-to-trade latency distributions (not just average), update-to-decision, decision-to-wire, gap detection time, etc.
- **Make the dangerous things impossible** via Rust's type system where practical (e.g., order states that cannot be in two places at once).
- **Document the "why"** for every performance decision.

## Repository Layout

```
grok-hft-data-engineer/
├── README.md                          # This file
├── topics/
│   ├── 01_system_architecture_overview/
│   ├── 02_rust_for_ultra_low_latency/
│   ├── ...
│   └── 09_performance_obs_testing_python_tooling/
├── interview-prep/
│   ├── questions_by_topic.md
│   ├── rigorous_practice_exercises.md
│   └── mock_scenarios.md
└── resources/
    └── curated_reading.md
```

Each topic directory contains a detailed `README.md` and a `demo_code/` folder with runnable examples.

## Getting Started Right Now

```bash
cd grok-hft-data-engineer
# Read the first topic
cat topics/01_system_architecture_overview/README.md

# Example: run a simple Rust demo (once you have Rust)
# cd topics/02_rust_for_ultra_low_latency/demo_code/...
# cargo run --release
```

Welcome to the craft of building systems where microseconds matter and correctness is non-negotiable.

Let's begin.

---

## New Focus Area on This Branch: HFT Data Management & Data Pipelines

This branch (`hft-data-pipelines`) extends the repository with deep material on **data management and data pipelines** for HFT trading roles.

We cover:
- High-performance data structures fully (or near-fully) implemented in Rust (with references to production crates)
- Lock-free and zero-copy patterns for real pipelines
- Tick data reconstruction, storage, and replay
- Analytical layers using Arrow/Polars
- Brief explanations of important non-Rust topics (FPGA, DPDK, Aeron, etc.)

All content follows the same philosophy as the rest of the repo:
- Clear "Why it matters", "How it works", analogies for newcomers
- Tables, ASCII diagrams
- Runnable Rust (and Python where helpful) examples
- Keep technical terms for mental mapping while explaining them simply

Start with `topics/11_advanced_rust_data_structures/README.md` and its `demo_code/`.

See also `topics/11_advanced_rust_data_structures/gaps_and_nice_to_haves.md` for the bigger picture.
