# 10 — HFT Data Pipelines Architecture for Data Management

## Why This Matters in Ultra-Low-Latency Crypto Trading

In HFT data trading roles, especially those focused on **data management and data pipelines**, you are no longer just consuming market data — you are responsible for building the systems that ingest, normalize, route, store, replay, and analyze it reliably and efficiently.

A modern HFT firm (crypto or traditional) handles:
- Millions of messages per second across venues
- Need for nanosecond-accurate timestamps
- Reconstruction of consistent order books across symbols and venues
- Low-latency distribution to strategies, risk, and execution
- High-fidelity historical data for backtesting and research
- Compliance, audit, and replay capabilities

Poor pipeline design leads to:
- Message loss or duplication during volatility spikes
- Stale data reaching strategies
- Inability to replay events deterministically
- High operational cost or complexity
- Tail latency that kills P&L

This topic gives you the architectural blueprint. Subsequent topics dive into the concrete data structures, algorithms, and implementations (mostly in Rust, with Python where it makes sense).

## What Problem Does It Solve?

Traditional "data engineering" pipelines (Kafka + Spark + S3) are often too slow or too lossy for HFT.

HFT data pipelines must solve simultaneously:
- **Ultra-low latency** on the hot path (feed handler → strategy decision)
- **High correctness** (no lost updates, correct book state even after reconnects/gaps)
- **Replayability** (exactly the same state when replaying historical data)
- **Scalability** across symbols, venues, and consumers
- **Separation of concerns** between hot path (must be fast/predictable) and cold path (can be heavier: storage, analytics, ML)

## How Does It Solve the Problem?

### Core Architectural Principles

1. **Hot Path vs Cold Path Separation** (reinforced from topic 01)
   - Hot path: socket → parse → normalize → publish state → consumers
   - Cold path: persistence, full-depth storage, analytics, backtesting data prep, monitoring

2. **Event Sourcing + CQRS flavor**
   - Treat every market data update, trade, and order as an immutable event.
   - Derive current state (order book, positions) from the event stream.
   - This enables perfect replay.

3. **Layered Architecture** (Medallion-like but optimized for latency)

```
Raw Ingestion (Bronze)
  ↓ (zero-copy, lock-free)
Normalized Stream (Silver)   ← hot path
  ↓ (publish via ring buffers / atomics)
Derived State (Gold L1)      ← order books, positions, signals (low latency)
  ↓
Analytical / Storage (Gold L2) ← Arrow, Parquet, ClickHouse, etc. (higher latency OK)
```

4. **Publish-Subscribe with Backpressure**
   - Use lock-free ring buffers or Disruptor-style between stages.
   - Bounded queues + clear drop or block policy.

5. **Idempotency and Sequencing**
   - Every feed has sequence numbers.
   - Snapshot + Delta model (see Binance depth rules + generalization).

### Key Technologies (2026 landscape)

**Rust ecosystem (hot path):**
- `bytes`, `zerocopy` for parsing
- `rtrb`, `ringbuf`, custom Disruptor ports for inter-thread comms
- `crossbeam`, `tokio` (with care — often plain threads + affinity better for true hot path)
- `polars` (Rust) + `arrow` / `datafusion` for analytical parts
- `bumpalo`, custom slabs for allocation control

**Python ecosystem (orchestration, analytics, backtesting):**
- `polars` (lazy) for fast vectorized processing
- `pyo3` / `maturin` to call Rust hot components
- `pyarrow` for zero-copy handoff
- Airflow / Prefect / custom for orchestration (but keep heavy work out of it)

**Non-Rust/Python (explain briefly, as they are important):**
- **FPGA / SmartNICs**: Offload parsing and basic book updates to hardware for lowest latency. You interact via DMA or high-speed interconnects. Not written in Rust/Python.
- **DPDK / kernel bypass**: User-space networking to avoid kernel overhead on market data reception.
- **Aeron**: High-performance transport (UDP + shared memory) used for low-latency messaging between processes/machines.
- **SBE (Simple Binary Encoding)**: Zero-copy binary protocol generation (common in traditional finance).
- **Redpanda / NATS JetStream**: Lower-latency Kafka alternatives when you need durable streaming.
- **ClickHouse / QuestDB / Timescale**: For analytical storage of tick data.

### Data Flow Diagram (ASCII)

```
Exchange (Binance WS / Multicast / FIX)
           │
           ▼  (hot thread, pinned core, DPDK optional)
   [Rust Feed Handler]
           │  (parse with zerocopy)
           ▼
   [Lock-free Ring Buffer]  ──▶ [Rust Normalizer / Book Reconstructor]
           │                           │
           │ (atomic snapshot / seqlock)│
           ▼                           ▼
   [Strategy Consumers]         [Risk / Position Engine]
           │                           │
           └───────────┬───────────────┘
                       ▼
               [Cold Path: Persistence + Analytics]
                       │
                       ▼
               Parquet / ClickHouse / Replay Store
```

## Applicability in Real Trading Systems

### How to Apply It Effectively
- Always design for **replay first**. If you can't replay the exact sequence of events and get identical state, your pipeline has bugs.
- Use **bounded, lock-free** structures on hot paths.
- Separate **symbol** handling (per-symbol books) from **cross-symbol** aggregation.
- Timestamp everything at the earliest possible point (kernel or NIC timestamp if available).
- Version your data formats and events.

### Key Gotchas and Pitfalls
- Treating WebSocket as reliable ordered stream (it's not during reconnects).
- Using general-purpose queues (tokio::mpsc, std channels) on hot paths → hidden allocations and contention.
- Reconstructing books incorrectly from deltas (double application or missed gaps).
- Mixing hot and cold work in the same thread.
- Underestimating the cost of copying data between Rust and Python.

### When to Use vs When NOT to Use
- Heavy frameworks (Spark, Flink) → only for cold/offline analytics, not live data planes.
- Pure Python pipelines → acceptable for research or low-frequency, disastrous for true HFT.
- Full in-memory only → great for low-latency but you need persistence strategy for recovery and backtesting.

## Visual Understanding

### Layered Data Pipeline (SDD)

```
┌────────────────────────────────────────────────────────────┐
│                    Hot Path (colo / low latency)           │
│  Feed Threads → Ring Buffers → Normalizers → State (LOB)   │
│                  (lock-free, pinned cores)                 │
└────────────────────────────────────────────────────────────┘
                              │ (atomic publish)
                              ▼
┌────────────────────────────────────────────────────────────┐
│                 Consumers (strategy, risk, exec)           │
└────────────────────────────────────────────────────────────┘
                              │ (sampling or async)
                              ▼
┌────────────────────────────────────────────────────────────┐
│                 Cold Path (storage, analytics, ML)         │
│  Arrow / Polars → Parquet / DB → Backtesting / Research    │
└────────────────────────────────────────────────────────────┘
```

## Data Structures Comparison

| Component              | Hot Path Choice                  | Why                                      | Rust Crate / Impl          | Python Equivalent     |
|------------------------|----------------------------------|------------------------------------------|----------------------------|-----------------------|
| Inter-thread queue     | Lock-free ring buffer            | Zero allocation, predictable             | rtrb / custom Disruptor    | multiprocessing.Queue (slow) |
| Order book state       | Flat array + SmallVec            | Cache friendly, fast top-N               | Custom + smallvec          | Polars (analytical)   |
| State publishing       | Seqlock / Atomic snapshot        | Readers never block writer               | Custom atomics             | Not directly          |
| Historical storage     | Append-only + columnar           | Fast replay + compression                | Arrow + Parquet            | Polars / PyArrow      |
| Analytical queries     | DataFusion / Polars lazy         | Vectorized, multi-threaded               | polars / datafusion        | Same                  |

## Demo Code & Examples

See the `demo_code/` directories under later topics for concrete implementations:
- Topic 11: from-scratch ring buffer + LOB structures
- Topic 13: full Disruptor-style multi-consumer pipeline
- Topic 14: Arrow-based handoff between Rust and Polars

All demos are runnable with `cargo run --release` or Python equivalents.

## Further Reading

- LMAX Disruptor paper
- "Mechanical Sympathy" blog (Martin Thompson)
- Databento / Jump Trading tech blogs on market data architecture
- Aeron documentation
- Arrow / DataFusion docs

## Interview Focus

- "Design a market data pipeline that can replay deterministically and feed both a low-latency strategy and a heavy analytics engine."
- "How do you handle gaps and reconnects without ever publishing inconsistent state?"
- "Compare ring buffers vs channels vs Disruptor for a feed → multiple consumers setup."
- "What would go wrong if you used Kafka directly in the hot path?"
- "Walk through how you would move data from a Rust hot path into a Polars analytical pipeline with minimal copying."

## Potential Gaps & Nice-to-Haves

Gaps addressed in following topics:
- Concrete deep implementations of the structures mentioned
- Full tick storage and replay engine
- End-to-end lock-free streaming
- Analytical layer integration

Nice-to-haves (brief explanations provided where full impl not practical):
- FPGA offload for parsing/normalization
- DPDK integration
- Aeron transport layer
- Production-grade monitoring with nanosecond histograms
- Multi-venue book consolidation and latency normalization

This topic sets the mental model. Next topics deliver the code.
