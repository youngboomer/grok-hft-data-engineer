# War Stories & Anti-Patterns in HFT Data Pipelines

This file collects real-feeling stories and common mistakes. Use them to prepare "tell me about a time..." answers and to avoid them in your own work.

## War Story 1: The Silent Gap
**Situation**: A feed handler for Binance depth data was dropping updates during high volatility. The strategy started quoting on stale books.

**What happened**: Reconnect logic fetched a snapshot but didn't properly buffer and apply deltas that arrived during the fetch. Sequence numbers were not checked strictly.

**Root cause**: "Good enough" gap detection that only logged instead of halting the hot path.

**Fix**: Strict sequence checking; on gap, mark book stale, switch to cold-path snapshot + replay, and only resume publishing when consistent.

**Metrics**: p99 decision latency stayed low; book correctness went from ~95% to 100% in replays.

**Lesson**: In HFT, "eventual consistency" on the hot path is usually just "wrong".

## War Story 2: The GIL Tax at the Boundary
**Situation**: Rust parser via PyO3 was "fast" in benchmarks, but end-to-end from tick to Polars feature was slow in prod.

**What happened**: The Python side was creating objects and holding the GIL right at the handoff. Under burst, this serialized everything.

**Fix**: Release GIL in Rust, return Arrow batches directly, do as much aggregation as possible in Rust before crossing.

**Lesson**: The boundary between hot Rust and cold Python is where most performance is lost if not designed carefully.

## Common Anti-Patterns

- Treating the hot data path like a normal ETL pipeline (using Pandas on every tick, unbounded queues, etc.).
- Ignoring sequence numbers on reconnect ("we'll just take the latest snapshot").
- Using general-purpose queues (tokio channels, std mpsc) for high-rate market data instead of purpose-built ring buffers.
- Mixing logging / metrics into the hot path "just for now".
- Assuming that because it works for one symbol it will work for 50 under load.
- Building the entire backtest in the same code path as live without a clean replay abstraction.

Document your own war stories as you build. They are gold in interviews.

See the case studies for more positive examples of what "good" looks like.