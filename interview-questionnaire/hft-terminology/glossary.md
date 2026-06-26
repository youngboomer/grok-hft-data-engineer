# HFT Terminology Glossary

Every term below is explained with:
- Clear definition
- Why it matters in ultra-low-latency crypto trading
- System impact (latency, correctness, risk)
- Simple analogy (smart 10-year-old level)
- Binance-relevant example

---

## Core Concepts

### Tick-to-Trade Latency
**Definition**: The time from when a market data update (tick) fully arrives on the socket until an order based on that data is placed on the wire.

**Why it matters**: This is the primary performance metric for market making. Lower and more predictable tick-to-trade = better ability to quote at the right price before others react.

**System Impact**:
- Directly affects adverse selection and capture rate.
- p99 and p99.9 matter far more than average.
- Every allocation, lock, or syscall on this path increases tail latency.

**Analogy**: Like the time between seeing a ball coming at you and swinging the bat. If your reaction is slow or sometimes very slow, you miss good pitches and sometimes swing at bad ones.

**Binance Example**: Depth update for BTCUSDT arrives → you update book → decide new bid/ask → signed REST order leaves your machine.

### Hot Path vs Cold Path
**Definition**:
- **Hot path**: The absolute minimal work that must complete before you can react to new market data (socket → parse → decision → order on wire).
- **Cold path**: Everything else (reconnects, snapshots, logging, metrics, admin, reconciliation).

**Why it matters**: Mixing cold work into the hot path is the #1 cause of mysterious tail latency in production.

**System Impact**: Hot path must be allocation-free, lock-free (or very short), cache-friendly, and pinned to isolated cores when possible.

**Analogy**: Hot path is like the emergency lane on the highway. Only life-critical vehicles (market data → decision) are allowed. Everything else (logging, reconnect) must take the normal roads.

### Adverse Selection
**Definition**: When your quotes are hit by informed traders who know something you don't (or who are faster).

**System Impact**: If your local book is even 1-2ms stale, you will be picked off on the wrong side. Fast, correct market data handling directly reduces adverse selection.

### Inventory / Position Skew
**Definition**: Adjusting your quotes based on your current net exposure so you don't become too long or too short.

**Why it matters**: Uncontrolled inventory is how market makers blow up. Good skew logic turns risk management into a pricing signal.

**Impact**: Must be visible to the quoting logic with very low latency after a fill arrives.

---

## Networking & Exchange Terms

### TCP_NODELAY
**Definition**: Disables Nagle's algorithm. Small packets are sent immediately instead of being buffered.

**Why it matters**: In HFT, waiting even 1-40ms for more data to batch is unacceptable.

**Analogy**: Instead of waiting until the delivery truck is full, you send the package the moment it's ready.

### Snapshot + Delta (Order Book)
**Definition**:
- Snapshot = full current state of the book at a point in time.
- Delta = incremental changes (bids/asks added, updated, or removed).

**Binance Rule**: You must apply deltas only after the snapshot's `lastUpdateId`, and you must discard deltas whose `lastUpdateId` is older than the snapshot.

**Impact of getting it wrong**: Corrupted book → quoting on stale or wrong prices → instant losses.

### ListenKey / User Data Stream
**Definition**: Authenticated WebSocket stream from Binance that delivers your private execution reports, order updates, and account updates.

**Critical Point**: This is a completely separate connection from public market data. You must correlate fills back to your local order state reliably.

**Impact**: If you lose this stream or mishandle reconnects, you can have incorrect inventory or place duplicate orders.

### Sequence Number / Update ID Gap
**Definition**: Detecting that you missed one or more messages (common during volatility or reconnects).

**Handling**:
- Buffer deltas during reconnect.
- Fetch fresh snapshot.
- Apply only deltas newer than the snapshot.
- Discard duplicates.

**Impact**: Missing a gap → stale book. Applying the same delta twice → double counting levels.

### Idempotency (Client Order ID)
**Definition**: Using a unique identifier so that sending the same order request multiple times has the same effect as sending it once.

**Why critical on reconnects**: You may not know if your previous order placement actually reached the exchange.

---

## Concurrency & Memory Terms

### Memory Ordering (Acquire / Release / Relaxed / SeqCst)
**Definition**: Rules that control when writes by one thread become visible to reads in another thread.

**Acquire/Release** is the most common useful pair for publishing data:
- Writer uses `Release` when publishing.
- Reader uses `Acquire` when consuming.

**Relaxed** = fastest but can produce torn or stale views.

**Analogy**: Release is like putting a new edition of a newspaper on the stand and ringing a bell. Acquire is waiting to hear the bell before you trust what you read.

### Seqlock (Sequence Lock)
**Definition**: A lock-free technique for publishing small structs. Writer bumps a sequence number after updating data. Readers read data then re-check the sequence. If it changed, they retry.

**Common Use**: Publishing best bid/ask + sequence atomically without a lock.

**Advantage over Mutex**: Readers never block writers. Writers never block readers.

### False Sharing
**Definition**: Two different pieces of data that happen to live on the same CPU cache line. When one core writes its data, it invalidates the cache line for the other core, causing expensive cache coherency traffic.

**Impact in HFT**: Two hot atomics (e.g. best_bid and some metric counter) on the same cache line can destroy performance.

**Mitigation**: Pad structures so hot fields are on separate cache lines (`#[repr(align(64))]` or manual padding).

### Lock-Free vs Wait-Free
**Lock-Free**: At least one thread makes progress in a finite number of steps (no deadlock), but a thread might retry many times (bad tail latency).

**Wait-Free**: Every thread finishes in a bounded number of steps (stronger guarantee, harder to achieve).

Most practical HFT structures aim for lock-free + low contention.

---

## Data Structure Terms

### Price-Time Priority
**Definition**: At the same price level, orders are matched in the order they arrived (time priority).

**Relevance**: When you model your own order book or simulate matching, you must respect this if you want realistic fill probabilities.

### Top-of-Book (TOB) vs Full Depth
**Definition**:
- TOB = only best bid and best ask.
- Full depth = all available levels.

Many strategies only need fast TOB + imbalance. Maintaining full depth correctly is more expensive.

### Ring Buffer / Circular Buffer
**Definition**: Fixed-size buffer that wraps around. Excellent for bounded queues between threads when you want to avoid allocations.

**Common in HFT**: SPSC intent queues, recent trade buffers, latency logging rings.

### Bump Allocator / Arena
**Definition**: Allocate a large block of memory once. Then "bump" a pointer forward for each new allocation. Free everything by resetting the pointer.

**Benefit**: Extremely fast allocation + no fragmentation within the arena. Perfect when objects have clear batch lifetimes.

---

## Risk & Trading Terms

### Hedge
**Definition**: Taking an offsetting position on another venue (or instrument) to reduce net exposure.

**Impact on System**: Requires fast feedback from fills on the primary venue to the hedging engine. Cross-venue latency differences create risk.

### Rate Limit
**Definition**: Exchange-imposed limits on number of requests (REST) or orders per second.

**Binance Reality**: 1200 requests per minute on many endpoints. Violating this gets you banned temporarily or your IP restricted.

**System Impact**: Your OMS must never send orders faster than allowed, even during reconnect storms.

### Maker vs Taker
**Definition**:
- Maker = your order rests on the book and provides liquidity.
- Taker = your order crosses the spread and takes liquidity (pays higher fees).

Market makers want to be makers almost all the time.

### Last Look (less common in spot crypto, more in FX)
**Definition**: The practice where a liquidity provider can reject a trade after seeing the request.

---

## Quick Mental Models

| Term                    | One-Sentence Mental Model                                      | Latency Risk if Done Wrong          |
|-------------------------|----------------------------------------------------------------|-------------------------------------|
| Tick-to-Trade           | End-to-end reaction time                                       | Direct P&L impact                   |
| Hot Path                | The sacred emergency lane                                      | Tail latency explosions             |
| Seqlock / Atomic Pub    | Putting a new newspaper out without blocking readers           | Readers see garbage or old data     |
| Snapshot + Delta        | Rebuilding a Lego set from instructions + picture              | Corrupted book = guaranteed losses  |
| Client Order ID         | "If you already did this, please ignore the duplicate"         | Duplicate orders or missed fills    |
| False Sharing           | Two people fighting over the same library desk drawer          | Hidden cache line ping-pong         |
| Bump Allocator          | Taking a big sheet of paper and just drawing boxes on it       | Allocator jitter during bursts      |

---

Use this glossary as your foundation. Every time you see a term in a question, come back here first.
