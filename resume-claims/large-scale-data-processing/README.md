# Large-Scale Data Processing (PySpark, Airflow, Medallion, etc.)

This section prepares you to deeply claim expertise in building reliable, scalable data pipelines at high volume.

## PySpark Deep Mastery

### Spark Architecture Overview (Must Know Internals)
- **Driver**: Orchestrates, plans jobs, communicates with Cluster Manager.
- **Executors**: Run tasks, hold data in memory/disk.
- **Cluster Manager**: YARN, Kubernetes, Standalone, or EMR.
- **Catalyst Optimizer**: Rule-based + cost-based optimization of logical plans.
- **Tungsten**: Whole-stage code generation, off-heap memory management, vectorized execution (columnar), cache-friendly data layout.

**Why this matters**: Most people use PySpark as "Pandas but distributed". Interviewers love when you can talk about how Catalyst rewrites your query or how Tungsten reduces JVM overhead.

### Spark SQL / Catalyst & Tungsten Tuning

**Key Tuning Levers**:
- `spark.sql.adaptive.enabled=true` (AQE - Adaptive Query Execution) — dynamically coalesces partitions, switches join strategies.
- Broadcast joins for small tables (`spark.sql.autoBroadcastJoinThreshold`).
- Skew handling (see below).
- `spark.sql.shuffle.partitions` — often too high or too low.
- Whole-stage codegen (enabled by default, but can be disabled for debugging).
- Off-heap memory (`spark.memory.offHeap.enabled`).
- Columnar formats (Parquet with proper compression and stats).

**Common Interview Question**:
"Explain the difference between a logical plan and physical plan. How does Catalyst optimize a join + filter + aggregate?"

Strong answer walks through rule application (predicate pushdown, constant folding) and then Tungsten's codegen for the physical operators.

### Custom Partitioners & Salting for Skew

**Data Skew Problem**:
One key (e.g., a popular user or product) has millions of records while others have few. Default hash partitioning puts everything on few tasks → stragglers.

**Techniques**:
1. **Salting**: Add random prefix to hot keys.
   ```python
   # PySpark example
   from pyspark.sql.functions import rand, concat, lit
   df = df.withColumn("salt", (rand() * 10).cast("int"))
   df = df.withColumn("salted_key", concat(col("user_id"), lit("_"), col("salt")))
   # Then aggregate on salted_key and later combine
   ```
2. **Custom Partitioner** (Scala/Java mostly, but you can achieve via repartition + custom logic or use `repartitionByRange`).
3. **Isolate hot keys**: Process hot keys separately with different parallelism.

**Gotcha**: Salting increases shuffle volume. Use it judiciously and only on known hot keys.

### Window Functions & Stateful Processing

**Window Functions**:
- `row_number()`, `rank()`, `dense_rank()`, `lag/lead`, `sum() over (partition by ... order by ... rows between ...)`
- Very powerful for sessionization, running totals, previous value comparisons.
- Performance note: Windows can cause large shuffles if not careful with partition keys.

**Stateful Processing** (Structured Streaming):
- `mapGroupsWithState` / `flatMapGroupsWithState`
- `groupBy(...).agg(..., countDistinct...)` with state
- Watermarking for late data
- State store (HDFS, RocksDB backend in newer Spark)

You should be able to describe a use case like "maintaining per-user session state across a high-volume clickstream with exactly-once semantics."

### Pandas UDFs / Arrow

**Traditional Python UDFs**: Very slow (row-by-row, GIL, serialization).

**Pandas UDFs (Vectorized)**:
- Use Arrow for efficient data transfer.
- Operate on `pandas.Series` or `DataFrame` in batches.
- Types: `SCALAR`, `GROUPED_MAP`, `GROUPED_AGG`, etc. (check current Spark version).

**Best Practice**:
```python
from pyspark.sql.functions import pandas_udf
from pyspark.sql.types import DoubleType
import pandas as pd

@pandas_udf(DoubleType())
def my_fast_func(s: pd.Series) -> pd.Series:
    return s * 2 + 1   # vectorized pandas/numpy operations
```

Even better in modern Spark: Use native Spark functions or Polars inside some UDFs when possible.

**Arrow in Spark**:
- `spark.sql.execution.arrow.pyspark.enabled`
- Dramatically faster roundtrips between JVM and Python.

### Airflow (DAG Orchestration)

**Core Concepts**:
- DAG = Directed Acyclic Graph of Tasks
- Operators: PythonOperator, BashOperator, SparkSubmitOperator, KubernetesPodOperator, etc.
- Sensors, XCom, TaskFlow API (@task)
- Scheduling, backfilling, catchup, depends_on_past
- Pools, queues, priority for resource control
- Connections, Variables, Secrets

**Modern Best Practices**:
- Use TaskFlow / decorators over classic operators when possible.
- Deferrable operators / async sensors to free workers.
- Dynamic task mapping (Airflow 2.3+).
- Separate orchestration from heavy computation (orchestrate Spark jobs, don't do heavy work inside PythonOperator).
- Use `KubernetesExecutor` or `CeleryExecutor` for scale.
- Idempotency: tasks should be safe to re-run.
- Observability: good logging, XCom for small data, external systems for large state.

**Common Pitfalls**:
- Putting business logic inside Airflow instead of in the actual processing engines.
- Too many small tasks (scheduling overhead).
- Not handling retries and SLA misses properly.
- Mutable global state or non-idempotent tasks.

**Interview Story**:
"I designed a Medallion pipeline orchestrated by Airflow with dynamic task mapping for daily partitions. We used SparkSubmitOperator with tuned clusters, added proper sensors for upstream data arrival, and implemented alerting on task duration percentiles."

### Medallion Architecture (Bronze / Silver / Gold)

**Bronze (Raw)**: Ingested as-is (Parquet, JSON, etc.). Minimal transformation. Good for reprocessing.

**Silver (Cleaned)**: Validated, deduplicated, schema enforced, lightly enriched. Business keys identified. Slowly changing dimensions handled.

**Gold (Business)**: Aggregated, modeled for consumption (star schema, wide tables for BI/ML). Highly optimized for query patterns.

**Benefits**:
- Separation of concerns
- Reprocessing from Bronze when logic changes
- Different SLAs and retention per layer
- Easier data quality gates between layers

**Implementation Notes**:
- Use Delta Lake / Iceberg / Hudi for ACID + time travel + schema evolution.
- Incremental processing where possible (merge, windowing).
- Data contracts between layers.

### ETL vs ELT + Data Validation & Deduplication

**ETL** (Extract-Transform-Load): Transform before load (traditional, good when compute at target is expensive).

**ELT** (Extract-Load-Transform): Load raw, transform in the target warehouse/engine (modern with powerful engines like Spark, BigQuery, Snowflake, ClickHouse).

**Data Quality Patterns**:
- Great Expectations or custom validation frameworks
- Deequ (Spark-specific)
- Checks: null rates, uniqueness, referential integrity, distribution shifts, schema drift
- Quarantine bad records instead of failing whole pipelines
- Deduplication strategies:
  - `row_number() over (partition by business_key order by ingestion_ts desc)` + filter
  - Delta MERGE
  - Bloom filters / approximate methods for very large scale

**Strong claim**:
"Built a Medallion lakehouse with Airflow orchestration and PySpark. Implemented custom salting for skewed joins, vectorized Pandas UDFs with Arrow, and multi-layer data quality validation that reduced bad data incidents by 90%."

## Practical HFT Use Cases & Scenarios

### Scenario 1: Skew-Resistant Medallion Pipeline for Tick Data Lake (Crypto MM)
**Business Context**: Building daily features (imbalance, realized vol, order flow toxicity) from raw Binance tick data for 200+ symbols. The PySpark job for the Silver layer was straggling badly on 5-8 high-volume pairs (they accounted for ~70% of messages), causing the entire daily feature job to miss its SLA and delay strategy research.

**Technical Challenge**: Classic data skew in market data; need to preserve exact sequence for correct book reconstruction; must support fast replay for backtesting 3 months of data; hot path needs to stay low-latency while cold analytics run heavy.

**What We Did (How)**:
- **Ingestion (Bronze)**: Rust hot path parser (zero-copy) wrote raw deltas + receive timestamps to partitioned Parquet (symbol/date/hour). Used append-only + simple seq per file.
- **Silver Layer**: PySpark with custom partitioner + salting. For hot symbols we added a random salt (0-31) to the key during groupBy for book reconstruction, then unsalt + merge at the end. Used `repartitionByRange` on price for volume profiles.
- **Vectorized UDFs**: Arrow-enabled Pandas UDFs (or Polars inside) for cleaning (gap detection, dedup via lastUpdateId).
- **Gold Features**: Window functions + aggregations on top of reconstructed books. Watermarking in Structured Streaming version for near-real-time features.
- **Orchestration**: Airflow with dynamic task mapping for symbols. Sensors waited for Bronze completeness. Separate DAG for backfill/replay.
- **Hot/Cold Split**: Live normalized books published via Redis Streams / ring buffer for strategy (hot, low latency). Full historical + features went to the lake (cold).

**ASCII Architecture (SDD)**:
```
[Binance Feeds] → [Rust Parser (hot)] → Bronze Parquet (S3)
                                       ↓
[Airflow Sensor] → PySpark Silver (salted + custom partitioner)
                                       ↓ (Delta MERGE)
Gold Features (Polars/ClickHouse) ←──┘
   ↑ (replay from Parquet)
Backtesting Engine
```

**Results**:
- Job runtime: 4.5h → 47min (p99 job duration).
- Skew stragglers eliminated; 99.8% of partitions finished within 30min of each other.
- Replay accuracy: 100% match vs live state (validated with sequence numbers).
- Bad data quarantined early; reduced downstream feature bugs by ~85%.

**Gotchas & When NOT**:
- Salting increased shuffle by ~15x on hot symbols — we mitigated with broadcast for small lookups.
- Don't use full Spark for the absolute hot path (use Rust ring buffer instead).
- Alternative: If scale is moderate, pure Polars + DuckDB on a fat VM can be simpler and faster to iterate than Spark.

**Interview Talking Points**:
- "We treated the live book reconstruction as hot path (low latency, exactly-once via seq) and the feature computation as cold (scale + analytics)."
- "Catalyst + Tungsten made the salted aggregates fast once we fixed the partitioning."

### Scenario 2: Airflow + Stateful Streaming for Real-time Risk Aggregation
(Details on maintaining per-venue inventory + cross-venue net exposure with exactly-once semantics under high fill rates, using mapGroupsWithState + watermarking, feeding both real-time alerts and daily PnL.)

## Interview Preparation Tips for This Area

Prepare 2-3 detailed stories:
1. A complex pipeline you built end-to-end (ingestion → bronze → silver → gold).
2. A performance problem you solved (skew, shuffle explosion, slow UDFs).
3. A reliability incident and how you made it idempotent / observable / self-healing.

Be ready to draw the architecture on a whiteboard and discuss tradeoffs (batch vs streaming, Delta vs Iceberg, Airflow vs Prefect vs Dagster, etc.).
