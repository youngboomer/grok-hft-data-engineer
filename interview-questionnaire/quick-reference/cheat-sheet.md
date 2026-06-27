# Quick Reference Cheat Sheet

Use this for last-minute review.

## Memory Ordering Quick Rules

- **Release** (writer): "Everything I did before this store is visible to anyone who sees this store with Acquire."
- **Acquire** (reader): "If I see a Release store, I also see everything that happened before it on the writer side."
- **Relaxed**: Fast but no ordering guarantees between threads.
- Default safe pattern for top-of-book: Write data with Release, bump sequence with Release. Reader: read sequence with Acquire, read data with Acquire, optionally re-check sequence.

## Binance Depth Rules (Memorize)

- Every message has `lastUpdateId`.
- Snapshot also has `lastUpdateId`.
- Buffer deltas while getting snapshot.
- Only apply deltas where `lastUpdateId >` snapshot's lastUpdateId.
- Ignore any message older than what you have already processed.

## Hot Path Commandments

1. No allocations after startup (or use arenas/bump).
2. No locks on the publication path of market data.
3. Prefer atomics + sequence or SmallVec over complex structures.
4. Pin hot threads to isolated cores when possible.
5. Measure distributions (p99, p99.9), not averages.
6. Reconnects, snapshots, logging = cold path only.

## Red Flags in Code Review (HFT)

- `.clone()` inside per-update loop
- `Mutex` or `RwLock` protecting market data
- `println!`, `format!`, or `log::info!` on hot path
- `HashMap` used for price levels on hot path
- Holding any guard across a function call that might do work
- Using `f64` for prices in the critical matching or quoting logic
- Not pre-sizing collections

## Strong Phrases to Use in Interviews

- "I would make the feed thread the single source of truth and never block it."
- "The strategy can drop updates if it falls behind. The feed must not."
- "I treat inventory reads as extremely cheap and non-blocking. Slight staleness for a few microseconds is acceptable if it keeps the hot path predictable."
- "I would add atomic histograms around the real path in production, not just synthetic benchmarks."
- "False sharing is invisible in single-threaded tests but destroys p99 under load."

## One-Page Mental Model

Feed Thread (pinned) → Atomic/Seqlock Top-of-Book → Strategy Thread(s) → Intent Queue (bounded) → OMS Thread → Rate Limiter + Sign + Send

Fills come back via separate user data stream → Inventory update (cheap) → visible to next strategy decision.

Everything else (reconnect, snapshot, metrics, hedging decision engine) lives off this path as much as possible.

---

Good luck. The goal is not to sound impressive — it's to sound like someone who has actually thought about what happens when the market is on fire at 3:17 AM.
