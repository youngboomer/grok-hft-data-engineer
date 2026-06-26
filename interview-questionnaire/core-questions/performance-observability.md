# Core Questions: Performance, Observability & Testing

## Question: How do you measure real tick-to-trade latency in production?

### Strong Answer

You need **end-to-end** measurement on the actual path:

1. Capture a high-resolution timestamp as close as possible to when the kernel delivers the frame (use `SO_TIMESTAMP` or `recvmsg` with timestamps if available, or timestamp immediately after `read`/`recv`).

2. Timestamp again when the order is handed to the kernel for send (`send` or `sendmsg`).

3. For more granularity, add timestamps at stage boundaries:
   - Frame received
   - Parsed
   - Book updated + published
   - Decision made
   - Order serialized + signed
   - Order on wire

4. Use **atomic histograms** or a lock-free ring buffer of recent latencies. Do not use heavy logging or blocking operations.

5. Export percentiles (p50, p95, p99, p99.9, max) on a cold path.

### Important Points
- Synthetic benchmarks are useful for micro-optimization but insufficient for proving production behavior.
- You must measure under realistic burst load (replay recorded bursts or synthetic high-rate generators).
- Correlate latency spikes with market events (liquidations, news, specific symbols).

## Question: Design a low-overhead metrics system for a hot path trading engine.

**Principles**:
- Measurement itself must not become a source of jitter.
- Prefer atomics and pre-allocated structures.
- Aggregate on the cold path.
- Sample when full histograms are too expensive.

**Common tools**:
- Atomic counters + atomic histograms (power-of-two buckets are easy).
- `hdrhistogram` (with care about allocation).
- Custom ring buffer of recent samples dumped periodically.

**What to measure** (at minimum):
- Tick-to-decision
- Decision-to-wire
- Update-to-publish latency (for the book)
- Queue depths (if using channels)
- Gap detection events
- Reconnect frequency and recovery time

## Question: What makes a good golden test for a market data parser or order book?

**Good properties**:
- Uses real (or very realistic) wire-format messages captured from Binance.
- Commits both the raw bytes **and** the expected parsed/updated state.
- Tests snapshot + multiple deltas, including gap scenarios.
- Tests edge cases: empty levels, price = 0 qty, very large qtys, duplicate updateIds.
- Runs fast and is part of CI.

This catches both protocol changes and logic regressions.
