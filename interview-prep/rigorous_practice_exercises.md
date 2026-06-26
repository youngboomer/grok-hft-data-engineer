# Rigorous Practice Exercises

These are concrete things you should build from scratch. Do them in release mode and measure.

1. **Minimal Lock-Free SPSC Queue**
   - Implement a bounded SPSC ring using only atomics (no crossbeam).
   - Benchmark push/pop latency and correctness under high contention.

2. **Toy Binance Depth Book Maintainer**
   - Build a book that ingests synthetic snapshots + deltas.
   - Must correctly handle gaps by requesting snapshot and replaying buffered deltas.
   - Measure update latency distribution.

3. **Golden Wire-Format Tests**
   - Pick a small binary message format (or a subset of Binance JSON you normalize).
   - Generate known-good serialized bytes once.
   - Write tests that assert your parser always produces the exact same structure.

4. **End-to-End Synthetic Tick-to-Trade Loop**
   - Three threads: feed → strategy → OMS.
   - Use atomics for top-of-book publication.
   - Synthetic bursts.
   - Instrument and optimize until you are happy with p99 under load.
   - Then deliberately add one bad thing (a clone, a lock, a print) and measure the damage.

5. **Atomic Histogram**
   - Implement a simple low-overhead histogram using atomics (buckets for ns/µs ranges).
   - Feed it from your timing harness and export percentiles.

6. **Reconnect + Dedup Scenario**
   - Simulate a WS drop + reconnect.
   - Ensure you never double-apply deltas and never place duplicate orders (use client order IDs properly).
