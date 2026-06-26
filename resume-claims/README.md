# resume-claims

**Purpose**: Deep, interview-ready mastery of the skills listed on your resume. This branch exists so you can speak with authority, provide concrete examples, explain internals, discuss trade-offs, and handle tough technical questions about every claim.

This follows the same rigorous learning philosophy used throughout the repository:
- **Why** this skill matters in real production systems (especially high-scale, low-latency, data-intensive environments)
- **What** problem it solves
- **How** it works (mechanisms, internals, architecture)
- **Applicability**: How to use it effectively, key gotchas/pitfalls (latency, correctness, scalability, cost), when to use vs. when NOT to use + better alternatives
- Data structure / technology comparisons with tables
- ASCII diagrams for data flows and architectures
- Practical code examples and runnable snippets
- Strong interview answers + common weak answers
- Connections to low-latency systems, HFT-style thinking, and real-world data engineering

## How to Use This Branch

1. For each major area, read the detailed notes.
2. Run any code examples (prefer release / optimized modes).
3. Practice explaining concepts out loud using the "Why → What → How → Tradeoffs" structure.
4. Prepare 2-3 personal stories per major skill (a project where you used it, a problem you solved, a mistake you learned from).
5. Review the "Interview Questions & Strong Answers" sections.

## Categories (Deep Dives)

- `systems-performance/` — Rust (PyO3/Maturin), zero-copy parsing, profiling, memory/latency optimization, low-latency streams, system design
- `large-scale-data-processing/` — PySpark (Catalyst/Tungsten, skew, UDFs), Airflow, Medallion architecture, ETL/ELT, validation & dedup
- `vectorized-compute/` — Pandas, Polars (lazy evaluation), NumPy internals and optimization
- `messaging-event-driven/` — ZMQ, MQTT/NanoMQ, AMQP, WebSockets, Redis Streams, event-driven patterns & tradeoffs
- `backend-apis/` — FastAPI, Django, REST/WebSocket best practices, auth, caching
- `databases/` — PostgreSQL, MongoDB, ClickHouse, QuestDB, DuckDB, SQLite + serious SQL optimization
- `cloud-platform/` — Deep AWS (EC2/S3/EMR/Athena/Lambda/EKS etc.), Docker, GitHub Actions/Jenkins, Linux ops
- `observability/` — OpenTelemetry, Prometheus + Grafana, New Relic, SLOs

Additional:
- `INTERVIEW-STRATEGY.md` — How to turn this knowledge into compelling interview stories and resume claims

## Resume Claim Strategy

For every bullet on your resume, you should be able to:
- Explain the **problem** you were solving at a business + technical level
- Describe the **architecture** and **why** you chose specific tools
- Discuss **trade-offs** and alternatives considered
- Talk about **metrics** (latency, throughput, cost, data quality)
- Describe a **challenge** and how you debugged/optimized it
- Show you understand the **internals** (not just "I used X")

Use this material to build those stories.

Start with the areas you feel least confident on or that are most central to the target role.
