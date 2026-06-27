# Common Anti-Patterns & Interview Red Flags

## Architecture Anti-Patterns

### The "Everything in One Runtime" Pattern
Putting market data ingestion, strategy, OMS, and reconnect logic all inside a single Tokio runtime without careful task priorities or isolation.

**Why it's dangerous**: The executor can introduce jitter, and a slow task can delay everything.

**Better**: Explicit thread separation for the true hot path.

### The "Shared Big Lock" Pattern
`Arc<RwLock<OrderBook>>` or `Arc<Mutex<Inventory>>` shared across feed and strategy.

**Problems**:
- Feed thread can be blocked by a slow reader.
- Readers can see inconsistent snapshots.
- High contention under load.

### The "Log Everything" Pattern
`info!` or `println!` or even structured logging inside the per-update hot loop.

Even structured logging usually allocates or does I/O eventually.

### The "We'll Optimize Later" Pattern
Starting with convenient abstractions (`HashMap<Price, Level>`, `Vec::new()` every time, `String` for everything) and hoping to fix performance later.

In this domain, the convenient path is often the path to tail latency.

## Data Structure Anti-Patterns

- Using `BTreeMap` or `HashMap` for price levels on the hot path.
- Not pre-sizing vectors.
- Cloning large structures "because it's only a few levels".
- Storing `f64` prices and doing floating point math in critical sections.
- Using `Arc` for tiny immutable data that could have been `Copy`.

## Concurrency Anti-Patterns

- Using `Relaxed` ordering for anything that is published to other threads.
- Assuming "it works on x86 so we're fine".
- Holding any kind of lock or guard while doing work that might allocate or syscall.
- Using async channels or `tokio::sync` primitives on the market data publication path.

## Observability Anti-Patterns

- Only having average latency numbers.
- No way to correlate a specific decision with the book state and inventory at that exact moment.
- Measuring only in synthetic quiet benchmarks.
- Adding heavy instrumentation that changes the behavior you're trying to measure.

## Interview Red Flags (Things That Make Interviewers Nervous)

- "We can just add more cores."
- "The p99 was bad so we increased the rate limit on the strategy thread."
- "We'll use a database for the order state machine."
- "Latency isn't that important as long as we don't lose money on average."
- No story for what happens during a multi-symbol burst + reconnect storm.
- Talking only about p50 or "it's fast enough in testing".

## How to Recover in an Interview If You've Said Something Weak

Acknowledge the limitation and show you understand the trade-off:

"That approach works for lower frequency systems, but in this domain it would introduce unacceptable tail latency / risk of stale state. A better approach would be X because..."

This shows maturity.
