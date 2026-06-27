# Hands-on Exercises for Resume-Claims Mastery

These exercises are designed to give you concrete artifacts and stories. Do them in Rust + Python where possible. Focus on measuring latency distributions under load and correctness.

## Systems & Performance
1. Build a small PyO3 module that takes a bytes buffer, does zero-copy parsing of a fake depth delta, and returns an Arrow batch. Benchmark the boundary cost.
2. Implement a seqlock atomic snapshot publisher and a reader that detects torn reads. Add false-sharing padding and measure impact.

## Large-Scale Data Processing
1. Take a skewed synthetic tick dataset and implement salting + custom partitioning in PySpark (or Polars) to build a simple order book snapshot. Compare runtime and data skew metrics.
2. Create an Airflow DAG skeleton (or Prefect) that orchestrates Bronze → Silver → Gold for tick data with data quality checks using Great Expectations or Deequ.

## Vectorized Compute + Messaging
1. Wire a Rust ring buffer producer (from topic 11 style) to emit Arrow batches. Consume in Polars lazy and compute rolling imbalance.
2. Prototype a Redis Streams producer/consumer with idempotency keys and outbox pattern for "fills". Simulate reconnect and prove no duplicates.

## Databases + Observability
1. Design and benchmark a hybrid storage setup: in-memory hot positions (Rust) + ClickHouse/QuestDB for historical ticks. Write a simple dedup + replay query.
2. Instrument a small FastAPI + Rust service with OpenTelemetry. Export to Prometheus + build a Grafana panel for p99 freshness and error budget burn rate.

## Cross-Cutting Capstone
Build (even a toy version of) the end-to-end case study in `case-studies/end-to-end-low-latency-pipeline/`:
- Rust hot parser + book + ring buffer.
- Arrow handoff to Polars.
- Parquet writer + basic replay.
- Simple observability metrics.

Document with:
- Architecture diagram (ASCII or image)
- Before/after p99 numbers under burst load
- One "war story" of something that went wrong
- Tradeoffs you considered

These projects become your strongest interview material. Record short loom videos of you walking through the diagram and metrics — extremely effective.
