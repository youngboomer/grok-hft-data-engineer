# Case Study: End-to-End Low-Latency Market Data Pipeline for Crypto Market Maker

## Business Context & Problem
A crypto prop trading firm runs a market making bot on 12 major Binance pairs. Their existing Python-only pipeline (WebSocket consumer → Pandas processing → quoting) had:
- p99 latency from tick receipt to quote decision of 2.1–4.8 ms during normal times, spiking to 15+ ms in volatility.
- Frequent stale books after reconnects (lost deltas).
- Backtests were slow and didn't match live behavior.
- Risk engine occasionally saw duplicate fills, causing bad hedges.

**Goal**: Build a production-grade data pipeline that keeps hot-path tick-to-decision under 100µs p99, survives bursts/reconnects with 100% fidelity, feeds both real-time quoting and analytical feature computation, and supports fast deterministic replay.

## Architecture (SDD + DFD)

**High-Level System Structure Design Diagram**:

```
┌─────────────────────────────────────────────────────────────────┐
│                    Hot Path (colo, pinned cores)                │
│  [Binance WS Client (Rust)] → [Zero-Copy Parser]                │
│                                    │                            │
│                                    ▼                            │
│  [Lock-free Ring Buffer (SPSC)] ──▶ [Order Book Reconstructor]  │
│                                    │ (seqlock snapshots)        │
│                                    ▼                            │
│  Strategy Engine (Rust hot + Python Polars features)            │
│  Risk (inventory skew)                                          │
└─────────────────────────────────────────────────────────────────┘
                                    │ (sampled Arrow batches)
                                    ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Cold Path                                   │
│  Redis Streams (durable fan-out) → Analytics Lake (Medallion)   │
│  Parquet + ClickHouse for replay & features                     │
│  Airflow orchestration                                          │
└─────────────────────────────────────────────────────────────────┘
```

**Data Flow Diagram (Simplified)**:

```
Exchange
   │
   ▼ (hot)
Rust Parser (zero-copy with bytes + offsets)
   │
   ▼ (publish via ring buffer)
Book State (flat array + atomic top-N + imbalance)
   ├──▶ Quoting logic (hot decisions)
   └──▶ Arrow handoff → Polars (features)
            │
            ▼ (cold)
   Redis Streams + Parquet Writer
            │
            ▼
   Backtesting Replay Engine + Gold Features
```

## Key Design Decisions & Why (Following Learning Style)

**Why Rust for hot path**:
- Python GIL + interpreter overhead made sub-100µs impossible under load.
- Needed predictable performance (no GC pauses, controlled allocations).

**What problem the hot path solved**:
- Ingest + normalize + maintain consistent book state at line rate.
- Publish latest view to multiple consumers without blocking the producer.

**How**:
- Parser: `bytes::Bytes` + manual deserialization (no serde in hot loop).
- State: Price levels in SmallVec (top 32) + hash for full depth when needed. Updates are in-place.
- Publication: Seqlock-style atomic snapshot (Acquire/Release) for best bid/ask + simple imbalance.
- Ring buffer: Custom or `rtrb` for SPSC between parser and book updater.
- Boundary to Python: PyO3 returning Arrow RecordBatch (zero-copy via `pyarrow`).

**Cold path**:
- Outbox pattern: Book updater writes to local append log + emits to Redis Streams.
- Storage: Parquet partitioned by symbol/date with dictionary + delta encoding.
- Replay: Load snapshot + apply deltas in exact sequence order (validated by seq numbers).
- Analytics: Polars lazy or DataFusion for feature jobs; Airflow for scheduling.

**Hot vs Cold**:
- Hot: Parser, book reconstruction, atomic publish, basic risk checks. Must be allocation-free after init, lock-free on publish path.
- Cold: Full storage, complex features, backfills, ML. Can tolerate higher latency and allocations.

## Implementation Highlights (Deep but Practical)

**Rust Parser Snippet (Zero-Copy Style)**:
```rust
fn parse_depth_delta(raw: &bytes::Bytes) -> Option<DepthDelta> {
    // manual offsets, no allocation until we extract the small Copy data we need
    // ...
    Some(DepthDelta { last_update_id, bids: small_vec, asks: small_vec })
}
```

**Atomic Publish**:
(See systems-performance topic for seqlock details.)

**Python Side (Polars)**:
```python
import polars as pl
df = pl.from_arrow(arrow_batch)
features = df.lazy().with_columns([...]).collect()
```

**Replay Engine**:
- Start from latest snapshot (stored as Parquet).
- Apply only deltas with higher sequence.
- Assert no gaps or duplicates.

## Results & Metrics
- p99 tick-to-decision: 2.4ms → 71µs.
- Survived 5 simulated liquidation cascades (15-30x normal rate) with 0 message loss and correct books.
- Backtest fidelity: 100% match to live P&L on 2-week replay.
- Daily feature jobs: 3.2h → 41min after skew fixes + Arrow.
- MTTD for data issues dropped dramatically with OTel traces + data quality metrics.

## Tradeoffs & Gotchas Encountered
- **Ring buffer vs Kafka**: Chose ring buffer for hot (Kafka added unacceptable jitter). Used Redis Streams only for the durable leg.
- **False sharing** on atomic counters — fixed with padding after seeing it in perf.
- **GIL in boundary**: Releasing the GIL was mandatory; creating Python objects in the hot loop was fatal.
- **Snapshot + delta correctness**: Most common source of "why does the book look wrong after reconnect?"
- **When we would NOT do this**: Lower frequency strategies or when team velocity on pure Python is more important than 100µs.

**Alternatives Considered**:
- Pure Polars ingestion (too slow under burst).
- Full Rust strategy (high rewrite cost for marginal gain at the time).
- kdb+ (powerful but steep curve and licensing).

## If We Did It Again / Future Improvements
- Add eBPF for deeper visibility into NIC drops.
- Coalesce updates for very high-update symbols.
- More aggressive use of DataFusion for complex streaming features.
- FPGA offload for the absolute parser (we only explained the concept).

## How to Talk About This in Interviews (Strong Answer Template)
" The core problem was that our data plane was on the critical path for quoting decisions, but our Python stack couldn't deliver predictable latency under real market conditions.

We split the pipeline into hot (Rust: zero-copy parse + lock-free book + atomic publish) and cold (Python Polars + storage). We used sequence numbers and the snapshot+delta pattern so reconnects never produced bad state. Metrics improved from milliseconds to tens of microseconds, and we validated everything with deterministic replay.

Key trade-off: we accepted some operational complexity in the hybrid boundary to protect the hot path. If the team was smaller or the frequency lower, we might have stayed pure Python with Polars."

**Common Follow-ups**:
- How did you choose between ring buffer and other queues?
- Walk through exactly how a gap is detected and recovered.
- What observability did you add for data quality?
- How do the hot and cold layers stay consistent?

This case study (or a version from your own experience) is extremely powerful for HFT data roles. Build a small prototype of the hot path pieces and you'll have artifacts + numbers to discuss.