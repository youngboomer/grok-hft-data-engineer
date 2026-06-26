# Toy Tick-to-Trade Pipeline Demo

This is a minimal but complete demonstration of the architectural separation that real ultra-low-latency market making systems use.

## What It Demonstrates

- **Hot path isolation**: The feed thread publishes best bid/ask using only atomics. No locks, no allocations, no channels on the publish side.
- **Non-blocking read**: Strategy reads the latest quote without ever blocking the feed thread.
- **Bounded backpressure**: A bounded crossbeam channel between strategy and OMS. If OMS is slow, intents are dropped rather than letting queues grow forever (configurable in real systems).
- **Feedback loop**: Fills generated in the OMS thread flow back and affect future quoting decisions via inventory.
- **Burst behavior**: The synthetic feed sends bursts of updates (mimicking Binance volatility spikes).

## How to Run

```bash
cd topics/01_system_architecture_overview/demo_code/toy-pipeline
cargo run --release
```

Always use `--release`. Debug builds have very different performance characteristics and will hide the real effects.

## Suggested Experiments

1. **Make strategy slow**  
   Inside the strategy loop, add `thread::sleep(Duration::from_millis(1));` after every 50 decisions.  
   Observe that feed update count and timing are unaffected.

2. **Watch inventory drift**  
   Change the fill probability or qty. Watch how the "skew" logic in strategy reacts.

3. **Measure real elapsed time**  
   Add `std::time::Instant` around the publish and decision sections. Print histograms of "decision lag".

4. **Try relaxed atomics everywhere**  
   Change the publish/read orderings to `Relaxed`. You may see torn reads (quote with mismatched seq). This is why `Acquire`/`Release` matters.

5. **Increase burst size dramatically**  
   Change `BURST_SIZE` to 50_000 and `NUM_BURSTS` to 3. Watch the channel fill up and see how many intents get dropped.

## Why This Architecture Matters for Binance Market Making

- During a large liquidation cascade, depth updates can arrive at 5k–20k+ per second on major pairs.
- If your feed thread ever blocks, your local book becomes stale within a few hundred microseconds.
- A stale book means you either:
  - Quote on the wrong side of the market (adverse selection), or
  - Miss the opportunity to hedge when inventory moves against you.
- The atomic publication pattern (or a proper seqlock / lock-free snapshot) makes "everyone sees the latest book" and "feed never waits" simultaneously true.

## Next Steps After This Demo

- Read the full topic README.
- Move on to topic 02 (Rust fundamentals) and 03 (concurrency models).
- Later come back and replace the toy atomic quote with a real mini order book (topic 07) and a real Binance WS client (topic 06).
