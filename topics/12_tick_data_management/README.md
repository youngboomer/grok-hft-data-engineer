# 12 — Tick Data Management and Storage for HFT Pipelines

## Why This Matters

HFT data roles spend a huge amount of time on managing tick data:
- Ingesting raw feeds
- Normalizing and cleaning
- Storing for replay and research
- Providing fast access for backtesting
- Ensuring deterministic reconstruction

Bad storage design means either:
- You can't replay accurately
- Backtests are too slow
- You lose data during high-volume periods
- Storage costs explode

## What Problem Does It Solve?

You need a system that can:
- Persist every event with nanosecond timestamps
- Allow fast random + sequential access
- Support both live replay (accelerated) and historical queries
- Compress well without losing precision
- Handle gaps, duplicates, and corrections gracefully

## How Does It Solve It?

### Core Model: Append-Only Event Log + Derived Snapshots

1. **Raw Event Log** (immutable)
   - Every incoming message (depth delta, trade, etc.) written as-is with receive timestamp.
   - Use binary format for compactness (Arrow IPC, custom binary, or Parquet row groups).

2. **State Snapshots**
   - Periodically (or on demand) materialize the current order book / position state.
   - Allows fast startup: load latest snapshot + replay only recent deltas.

3. **Indexing**
   - Time-based index (for range queries)
   - Symbol + sequence index (for exact replay)

### Storage Formats (Rust + Python)

**Hot / Recent data**:
- In-memory ring buffers + memory-mapped files
- Custom binary log (append only)

**Warm / Historical**:
- Apache Arrow + Parquet (excellent compression + fast columnar access)
- ClickHouse or QuestDB for analytical queries on ticks

**Rust implementation focus**:
- Use `arrow` crate + `parquet` crate for writing
- `memmap2` for fast read access to log files
- Custom binary writer for lowest latency ingestion

**Python**:
- `polars` + `pyarrow` for reading and transforming
- Great for research pipelines

### Key Algorithms

- **Deterministic replay**: Feed events in order, apply exactly as live.
- **Gap detection and repair**: Use sequence numbers.
- **Compression**: Dictionary + delta encoding per symbol.
- **Partitioning**: By date + symbol for efficient queries.

## Applicability

In crypto HFT you often deal with:
- High message rates on major pairs
- Frequent reconnects
- Need to replay for strategy tuning

In traditional markets: multicast feeds, larger books, more complex order types.

## Demo Code

See `demo_code/` for:
- Rust binary logger + reader using Arrow
- Python loader with Polars for analysis
- Simple replay engine

## Gaps & Nice-to-Haves

- Full production tick database (kdb+ style in Rust is rare)
- FPGA-accelerated ingestion (explained only)
- Advanced time-series compression (Gorilla, etc.)

Reference crates: `arrow`, `parquet`, `memmap2`, `polars` (Python).
