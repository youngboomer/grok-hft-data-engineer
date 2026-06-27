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
