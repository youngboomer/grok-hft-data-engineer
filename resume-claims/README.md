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

1. For each major area, read the detailed notes (Why → What → How → Applicability).
2. Run any code examples (prefer `--release` / optimized modes for realistic latency numbers).
3. Study the **Practical HFT Use Cases & Scenarios** (added for rich interview stories) and ASCII diagrams.
4. Practice explaining out loud using the "Why → What → How → Tradeoffs" structure + hot/cold path distinctions.
5. Prepare 2-3 personal STAR stories per skill using the template in `INTERVIEW-STRATEGY.md` (Situation, Task, Action with metrics, Result, Tradeoffs, "If I did it again...").
6. Do the hands-on exercises and build 1-2 end-to-end case studies.
7. Review "Interview Questions & Strong Answers" + the scenario bank in `INTERVIEW-STRATEGY.md`.

**Pro tip**: Treat this like HFT prep — focus on tail latency, correctness under burst load, and data fidelity. Always quantify (p99, messages/sec, error rates).

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
- `case-studies/` — Rich end-to-end HFT data pipeline examples with diagrams and metrics
- `exercises.md` — Hands-on build challenges to generate real artifacts and stories
- `war-stories-and-anti-patterns.md` — Common HFT data pipeline failures + lessons (excellent for "tell me about a time it went wrong" questions)
- `interview-cheatsheet.md` — Quick reference for interviews (key phrases, red flags, questions to ask)

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
