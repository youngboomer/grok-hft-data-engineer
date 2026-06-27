# Databases: PostgreSQL, MongoDB, ClickHouse, QuestDB, DuckDB, SQLite + SQL Optimization

## PostgreSQL

**Strengths**: ACID, excellent JSON support, powerful indexing, extensions (PostGIS, pg_trgm, etc.), mature ecosystem.

**Deep Topics**:
- Indexing strategies: B-tree, GiST, GIN, BRIN, partial indexes, expression indexes, covering indexes (`INCLUDE`).
- Query planning: `EXPLAIN (ANALYZE, BUFFERS, FORMAT JSON)`.
- MVCC and vacuuming (bloat, autovacuum tuning).
- Partitioning (declarative).
- Connection pooling (PgBouncer).
- Logical replication.
- JSONB best practices (avoid large objects, use generated columns, GIN indexes on jsonb).

**Common Optimization**:
- Missing indexes on foreign keys and filter columns.
- N+1 queries (use `select_related` / `prefetch_related` in ORMs or proper JOINs).
- Over-fetching columns.

## MongoDB

**When it shines**: Flexible schema, high write throughput, good for semi-structured or rapidly evolving data, horizontal scaling via sharding.

**Deep Topics**:
- Document model design (embed vs reference).
- Indexing (single field, compound, multikey, text, wildcard, partial).
- Aggregation pipeline (very powerful, but can be slow if not indexed).
- Change streams for event-driven patterns.
- Write concern / read concern tradeoffs.
- Sharding key selection (cardinality and query patterns are critical).

**Pitfalls**:
- Poor schema design leading to hot shards or large documents.
- Using Mongo as a relational database (anti-pattern for complex joins).
- Lack of transactions in older versions (multi-document transactions exist now but have limits).

## Analytical / Time-Series Databases

### ClickHouse
Column-oriented, extremely fast for analytics on large datasets. Excellent compression. Vectorized execution.

**Key Strengths**:
- Blazing fast `GROUP BY`, window functions, approximate algorithms.
- Materialized views for pre-aggregation.
- TTL for automatic data expiration.
- Replicated and distributed tables.

**Use when**: You need sub-second queries on billions of rows for dashboards, logs, metrics, or event data.

### QuestDB
Time-series focused, built on a column store with SIMD and a custom storage engine. Very strong on ingestion rate + time-based queries.

**Notable**:
- SQL with time-series extensions (`SAMPLE BY`, `LATEST ON`).
- High ingest with low resource usage.
- Good integration with Pandas/Polars via Arrow.

### DuckDB
Embedded analytical database (like SQLite but for analytics). Runs in-process.

**Superpowers**:
- Zero-copy with Pandas/Polars/Arrow.
- Excellent for local analysis of large Parquet/CSV files.
- Fast joins and aggregations.
- Great for data transformation steps before loading into a warehouse.

**Modern pattern**: Use DuckDB inside Python jobs for heavy local processing instead of spinning up Spark for medium data.

### SQLite
Small, reliable, zero-config. Surprisingly powerful for many workloads.

**Advanced**:
- WAL mode for better concurrency.
- Custom functions and virtual tables.
- Good for edge, testing, or small-to-medium services.

## SQL Optimization (Universal)

**Must-know techniques**:
- Index usage (avoid functions on indexed columns in WHERE).
- Proper JOIN order and types.
- Covering indexes to avoid table lookups.
- Window functions instead of self-joins or correlated subqueries.
- `EXISTS` vs `IN` vs `JOIN`.
- Approximate methods when exact is not needed (HyperLogLog style in some DBs).
- Partition pruning.

**Tools**:
- `EXPLAIN ANALYZE`
- Query profilers in each DB
- `pg_stat_statements`, `SHOW PROFILE` equivalents, ClickHouse `system.query_log`

## Comparison Table (Rough Guidance)

| DB          | Best For                     | Write Latency | Analytical Speed | Horizontal Scale | Schema Flexibility |
|-------------|------------------------------|---------------|------------------|------------------|--------------------|
| PostgreSQL  | General OLTP + some OLAP     | Low           | Good             | Vertical + logical rep | Moderate          |
| MongoDB     | Flexible documents, high write | Very Low     | Medium           | Excellent (sharding) | Very High         |
| ClickHouse  | Large-scale analytics        | High (batch)  | Excellent        | Excellent        | Low (columnar)     |
| QuestDB     | High-ingest time series      | Very Low      | Excellent        | Good             | Low                |
| DuckDB      | Local/embedded analytics     | N/A           | Excellent        | N/A              | Flexible           |
| SQLite      | Embedded / edge / small apps | Low           | Good             | None             | Flexible           |

**Interview Claim Example**:
"I designed a hybrid storage strategy using PostgreSQL for transactional state, ClickHouse for high-cardinality analytics, and DuckDB for ad-hoc transformations inside our Python data jobs. This gave us both strong consistency where needed and sub-second analytical queries on terabytes of data."

## Practical HFT Use Cases & Scenarios

### Scenario: Hybrid Storage for Tick Data + Real-time Positions
**Context**: Need fast appends for raw ticks, low-latency reads for current positions/inventory during trading, and heavy analytical queries for research (e.g., "volume profile over last 10k ticks").

**Approach**:
- Hot: In-memory structures (from topic 11) + Redis for latest positions (sub-ms reads).
- Warm appends: QuestDB or ClickHouse for time-series ticks (high ingest, fast time-range queries).
- Analytical: DuckDB (embedded, Arrow-native) inside Python jobs for ad-hoc or Polars jobs; PostgreSQL for transactional order/position state with proper indexing.
- Dedup on ingest using sequence + bloom or last-seen map.

**Hot/Cold**: Hot reads (current best + position) must be lock-free and in-memory. Cold queries (historical analysis) can scan columnar storage.

**Metrics**: 50k ticks/sec ingest with <5ms p99 query for last 1k ticks on a symbol.

**Tradeoff**: Chose ClickHouse/QuestDB over Timescale for raw speed on appends; accepted eventual consistency between hot cache and DB for the analytical layer.
