# Resume Claims → Interview Mastery Strategy

## The Goal
You don't just want to "know" these technologies. You want to be able to:
- Tell compelling, specific stories
- Explain **why** you made decisions
- Discuss **tradeoffs** intelligently
- Show you understand **internals**, not just APIs
- Connect skills across categories (e.g., low-latency Rust + PySpark + observability)

## Recommended Preparation Flow

For each major category:

1. **Read the deep material** in the corresponding folder.
2. **Prepare 2-3 stories** using this template:
   - Situation / Business context
   - Technical challenge (scale, latency, correctness, cost)
   - What you did (specific tools + why)
   - How you measured success (numbers!)
   - What you would do differently now
3. **Practice the "Why → What → How → Tradeoffs" structure** out loud.
4. **Review common interview questions** in the notes and formulate answers.
5. **Have a "demo" or code artifact** ready to talk about or show (GitHub link, architecture diagram, benchmark numbers).

## Cross-Cutting Themes Interviewers Love

- **Performance & Latency Thinking**: Applies to Rust, data pipelines, APIs, databases, and messaging. Always be ready to talk p99, profiling, and bottlenecks.
- **Data Quality & Reliability**: Validation, deduplication, idempotency, exactly-once vs at-least-once.
- **Cost & Scale Tradeoffs**: Cloud spend, shuffle in Spark, cardinality in metrics, etc.
- **Observability as a First-Class Citizen**: How do you know if it's working? How do you debug when it's not?
- **Hybrid Architectures**: Python + Rust, batch + streaming, SQL + DataFrames, managed + self-managed.

## Example Strong Resume Bullets (Adapt These)

**Systems & Performance**:
- "Developed high-performance data processing components in Rust (exposed via PyO3/Maturin) for zero-copy parsing and stateful logic, achieving 40-60x latency reduction on hot paths while integrating with existing Python data stack."

**Large Scale Data**:
- "Designed and operated a Medallion architecture lakehouse using PySpark, Airflow, and Delta Lake. Implemented custom partitioners + salting for skew, vectorized UDFs with Arrow, and multi-stage data quality validation."

**Vectorized + Databases**:
- "Replaced slow Pandas-heavy transformations with Polars LazyFrames + DuckDB, and optimized critical PostgreSQL/ClickHouse queries, reducing daily pipeline runtime from 3 hours to 25 minutes."

**Messaging + Backend**:
- "Built event-driven services using FastAPI + Redis Streams + ZMQ for low-latency paths. Implemented outbox pattern and idempotency to guarantee reliable delivery across services."

**Cloud + Observability**:
- "Migrated workloads to AWS EKS/Fargate with GitHub Actions CI/CD. Implemented full OpenTelemetry instrumentation exported to Prometheus + Grafana (SLOs) and New Relic (APM), reducing MTTD by 65%."

## Final Tips

- Numbers win interviews (latency improvement, cost reduction, data volume, error rate drop).
- Be honest about what you owned vs. what the team did.
- When you don't know something, say "I haven't used X in production, but here's how I would approach evaluating it..." — then show structured thinking.
- Connect skills: "I used low-latency Rust components inside our Spark UDFs..." or "I added distributed tracing across our FastAPI services and the downstream PySpark jobs."

This branch + your personal project stories should make every resume claim defensible and impressive.

## Practical HFT Scenario Bank for Interviews

Interviewers in HFT data roles love drilling into **real scenarios** involving low tail latency, correctness under bursts, hot/cold path decisions, and data pipelines. Prepare 1-2 detailed stories per category using the STAR method (Situation, Task, Action, Result) + "Tradeoffs & Lessons" + "If I did it again...".

### Core HFT Data Pipeline Scenario (Cross-Cutting)
**Scenario**: "Design and implement a market data ingestion pipeline for a crypto market maker handling 50k+ updates/sec on major pairs during volatility. It must feed a low-latency Rust strategy engine, a Python analytics layer for features, and persistent storage for replay/backtesting, all while surviving reconnects without data loss or duplicates."

**Strong Answer Structure** (follow our learning style):
- **Why it matters**: In HFT market making, stale or lost ticks lead to adverse selection or missed hedges. Predictable p99 tick-to-book latency (<100µs) and 100% message fidelity under burst load are table stakes for profitability.
- **What problem**: High-volume feeds (Binance depth/aggTrade) with gaps on reconnects; need to separate hot path (feed → normalized book → strategy) from cold (storage/analytics); Python is great for Polars features but slow/GC-prone for hot ingestion.
- **How we solved it**:
  - Hot path in Rust: Zero-copy parser (using `bytes` + custom offsets) + lock-free ring buffer (SPSC via `rtrb` or custom Disruptor-style) to publish atomic snapshots (seqlock for top-of-book + imbalance).
  - Normalization: Snapshot + delta reconstruction with sequence gap detection; fallback to REST snapshot on cold path.
  - Fan-out: Atomic publish for strategy/risk (hot); Arrow RecordBatch handoff via PyO3 for Python Polars lazy features (cold).
  - Storage: Append-only Parquet with partitioning by symbol/date + memmap for fast replay; ClickHouse for analytics queries.
  - Orchestration: Airflow DAGs for daily backfill jobs with Medallion layers (Bronze raw → Silver cleaned books → Gold features).
  - Observability: OpenTelemetry for traces across Rust/Python boundary; Prometheus histograms for p50/p99 latencies; data quality checks (dedup via seq, null rates).
- **Results**: Reduced p99 ingestion-to-feature latency from 2.3ms to 85µs; 0 message loss in 3 simulated volatility bursts (10x normal rate); backtests now replay at 50x real-time speed.
- **Tradeoffs & Gotchas**: Chose ring buffer over Kafka for hot path (Kafka adds ~200-500µs jitter and is overkill for single-process); released GIL in PyO3 boundary to avoid blocking; false sharing on atomics was a surprise (fixed with padding).
- **If I did it again**: Add more aggressive coalescing for high-update symbols; use eBPF for deeper observability on NIC drops.

**Weak Answer Red Flags**: Talking only averages ("it was fast"), no numbers, ignoring reconnects/gaps, claiming "we just used Spark for everything."

Practice telling this out loud in 3-4 minutes, then be ready for follow-ups: "How did you handle skew in the book data?" "What if the Rust side crashes?"

### Other High-Value Scenarios to Prepare
- **Skew in large-scale processing**: "Our PySpark job for building daily order book snapshots was straggling on 3 popular symbols (80% of volume). How did you fix it and what was the impact on pipeline SLA?"
- **Low-latency messaging**: "We needed sub-millisecond fan-out from a Rust feed handler to multiple Python risk engines without blocking the producer."
- **Hybrid Rust/Python boundary**: "Python ETL for tick features was too slow and allocating heavily; we moved hot parsing to Rust via PyO3/Maturin."
- **Observability in production**: "p99 latency looked good in staging but spiked in prod during market events. Root cause and fix?"
- **Storage & replay**: "Build a tick storage system that supports both real-time appends and deterministic historical replay for backtesting strategies."

Use the scenario bank below in each category README to flesh these out with specifics.

## Hands-on Exercises to Build Stories
For each category, pick 1-2 and implement/extend (even small prototypes):
- Wire a Rust zero-copy parser to a Polars lazy DataFrame via Arrow.
- Implement a basic lock-free ring buffer and benchmark hot path latency under load.
- Design an Airflow DAG + PySpark job for Medallion tick data with salting for skew.
- Add OpenTelemetry instrumentation to a FastAPI + Redis Streams service and build a Grafana dashboard for data freshness SLOs.

Document your work with diagrams, before/after metrics, and "lessons learned" — this becomes gold for interviews.
