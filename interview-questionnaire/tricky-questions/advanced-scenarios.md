# Tricky Questions & Scenarios

These are the questions that differentiate candidates who have "read about HFT" from those who have thought deeply about building one.

---

## Tricky Question 1: Your feed thread is correctly publishing best bid/ask using atomics. Suddenly during a burst your p99 decision latency jumps from 80µs to 1.8ms. The feed thread CPU usage is low. Strategy thread CPU is high. What do you investigate?

**Strong Answer**:

### Immediate Hypotheses
1. Strategy thread is doing too much work per update (maybe full book scan, allocation, logging).
2. The queue between feed and strategy (if any) is building up — backpressure.
3. Strategy is contending on something (lock, allocator, or even just cache lines).
4. The OS scheduled the strategy thread off its core.
5. Memory ordering is too strong (unlikely to cause 20x jump).

### Investigation Order
- Add lightweight atomic histograms on "time from publish to strategy seeing the update".
- Check if the number of decisions per second dropped while updates continued (means strategy is falling behind).
- Look at `perf` or `bpftrace` on the strategy thread specifically.
- Add a "work budget" — if strategy can't keep up, it should deliberately drop some updates instead of building latency.
- Check for accidental clones or `format!` calls that only happen during high update rates.

**Key Insight**: The symptom is in the strategy thread, not the feed. Many candidates immediately blame the publisher.

---

## Tricky Question 2: Design a reconnect + deduplication strategy that guarantees you never lose a fill and never place a duplicate order.

**Critical Points to Cover**:

### For Market Data (Depth)
- Detect gap via `lastUpdateId`.
- Do **not** drop your current book immediately.
- Fetch snapshot on separate REST connection (cold path).
- Buffer deltas received during the fetch.
- Apply snapshot + only newer deltas.
- Resume publishing top-of-book only after the book is consistent.

### For Orders & Fills
- Every order you send has a unique, monotonically increasing client order ID (or use UUIDs).
- On reconnect of the user data stream:
  - Re-establish listenKey.
  - After reconnect, query open orders via REST to reconcile state.
  - Never assume a previously sent order was lost — it might have been accepted.
- For critical orders (hedges), you may implement a "resend with same client ID" pattern. Binance will treat it as idempotent.

**Common Fatal Mistakes**:
- Creating a new client order ID on every reconnect attempt → duplicate orders.
- Trusting only the WebSocket stream for fills after a disconnect.

---

## Tricky Question 3: You need to maintain inventory across two venues (Binance + Deribit) with sub-millisecond reaction on fills. How do you structure the data flow?

**Key Challenges**:
- Fills arrive asynchronously on different connections.
- You need a consistent view of net exposure for quoting and hedging decisions.
- You cannot afford to lock the hot path while updating inventory.

**Good Architecture**:
- Each venue has its own feed handler that publishes fills into a lock-free or very-short-critical-section inventory update.
- Inventory is stored in a way that is cheap to read (atomic counters per symbol + total USD exposure, or a small seqlock struct).
- Hedging logic lives on a slightly warmer path. It reacts to inventory changes but does not block market data ingestion.
- You accept that for a few microseconds the two venues may have slightly inconsistent views — you design risk limits around that reality.

**What to Emphasize**:
- "I would make inventory reads extremely cheap and non-blocking. The cost of slightly stale inventory for a few microseconds is much lower than the cost of blocking the feed thread."

---

## Tricky Question 4: Why might increasing the number of strategy threads make your overall system slower under load?

**Possible Reasons**:
- False sharing on shared hot data structures.
- Cache thrashing (multiple cores fighting over the same cache lines containing the order book snapshot).
- Lock contention that wasn't obvious (if any locks exist).
- NUMA effects if cores are on different sockets.
- The work per update is so small that thread scheduling and context switch overhead dominates.

**Strong Answer**: "I would first try to keep the hot read path on as few cores as possible and use non-blocking publication. More threads only help if the work is actually parallelizable without introducing coherence traffic."

---

## Tricky Question 5: Design a rate limiter for your OMS that is both correct and does not add tail latency.

**Requirements**:
- Never exceed exchange rate limits.
- Under burst, you still want to send as many orders as legally possible without going over.
- The check itself must be extremely fast and predictable.

**Good Approaches**:
- Token bucket implemented with atomics (or a very small critical section).
- Pre-compute "next allowed send time" using atomic.
- Separate the rate limiter from the actual sending thread when possible.

**Bad Approach**: Using a `Mutex` around a sleep or around a shared token counter on every order.

---

Remember: In tricky questions, the interviewer is listening for **trade-off reasoning** and **production war stories** (or at least realistic mental simulations), not just textbook definitions.
