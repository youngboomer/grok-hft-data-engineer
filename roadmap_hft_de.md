# HFT Data Engineer Roadmap

**Goal**: Build the skills to work in high-frequency trading (HFT) roles focused on **data management, market data pipelines, low-latency systems, and real-time data engineering** — especially in crypto or traditional finance environments.

This roadmap is tailored to the structure of this repository (`grok-hft-data-engineer`). It integrates the original low-latency systems topics with newer content (topics 10–14) focused on data management and pipelines.
- Core low-latency systems thinking
- Rust for performance-critical code
- Modern data engineering (PySpark, Polars, Arrow, etc.)
- Practical implementation + interview readiness

## Existing Learning Instructions in This Repo

Before diving in, know where the "how to learn" guidance already lives:

- **[LEARNING_STYLE.md](LEARNING_STYLE.md)** — The canonical guide for how the owner of this repo likes to study and have topics explained. Every good topic README follows this pattern: Why → What → How → Applicability (with gotchas, when to use vs not) + analogies + visuals + interview focus. **Read this first if you want to internalize the meta-learning approach.**

- **HFT_DATA_PIPELINES_PLAN.md** — Specific recommended order and philosophy for the newer data-pipelines focused topics (10–14).

- **Main [README.md](README.md)** — High-level overview of the entire repo, the tick-to-trade mindset, study order for the original 9 topics, and pointers to branches/content.

- **Each topic's README.md** — Follows a consistent structure with "How to Use", demo instructions, and "Interview Focus" sections.

- **[interview-prep/](interview-prep/)** — Contains:
  - `questions_by_topic.md`
  - `rigorous_practice_exercises.md` (concrete things to build)
  - `mock_scenarios.md`

- **[resume-claims/INTERVIEW-STRATEGY.md](resume-claims/INTERVIEW-STRATEGY.md)** — How to turn knowledge into compelling stories and strong answers for data engineering + systems interviews.

- **Individual topic demo READMEs** — Always contain "Suggested Experiments" and "How to Run".

Use these as your operating manual. Do not treat topics as passive reading — treat them as guided projects + explanation practice.

## Recommended Pursuit Order

### Phase 0: Foundations & Mindset (1–2 weeks)
1. Read **[LEARNING_STYLE.md](LEARNING_STYLE.md)** completely.
2. Read the main **[README.md](README.md)**.
3. Go through **Topic 01: System Architecture Overview** + its toy pipeline demo.
4. Skim **Topic 09: Performance, Observability & Tooling** early (you'll keep coming back to profiling and measurement).

**Goal**: Internalize hot path vs cold path, tick-to-trade, predictability under burst load, and the measurement culture.

### Phase 1: Rust + Low-Latency Systems Core (3–5 weeks)
Work through these in order. Implement or extend at least one demo per topic.

- Topic 02: Rust for Ultra Low Latency
- Topic 03: Concurrency, Lock-Free & Threading Models
- Topic 04: Memory, Zero-Copy & Data Structures
- Topic 05: Linux Networking & Sockets Tuning
- Topic 06: Exchange Protocols (WS/FIX/SBE)
- Topic 07: Market Data & Order Book Management (implement the mini order book and experiment with deltas)

**Parallel practice**:
- Use `interview-prep/rigorous_practice_exercises.md`
- Start building small things from scratch (e.g., a toy lock-free SPSC queue).

### Phase 2: Data Pipelines & Modern Data Engineering (4–6 weeks)
This is the newer, deeper content added for data management roles.

Recommended order (see also `HFT_DATA_PIPELINES_PLAN.md`):

1. **Topic 10: HFT Data Pipelines Architecture** — Big picture + tech landscape.
2. **Topic 11: Advanced Rust Data Structures** — Ring buffers, LOB variants, seqlocks, object pools. Do the from-scratch implementations.
3. **Topic 13: Lock-Free Stream Processing** — Disruptor-style patterns and multi-consumer pipelines.
4. **Topic 12: Tick Data Management & Storage** — Snapshot+delta, Parquet/Arrow storage, replay engines, memmap.
5. **Topic 14: Analytical Data Pipelines** — Arrow handoff, Polars (lazy), DataFusion, Rust ↔ Python zero-copy.

While doing these:
- Run every demo in `--release`.
- Extend at least one demo (add a second consumer, add metrics, add Python Polars analysis side).
- Reference the real crates mentioned but also build improved/educational versions from scratch as instructed.

**Rust + Python rule**: Core hot path and performance-sensitive pieces in Rust. Analytics, orchestration, and validation layers can be Python (Polars, PyArrow, etc.).

### Phase 3: Domain + Interview Depth (ongoing + 3–4 weeks intensive)
- Finish the original Topics 08 (OMS/Strategy/Hedging) if you skipped it.
- Go deep on **interview-prep/**:
  - Answer all `questions_by_topic.md`.
  - Do the exercises in `rigorous_practice_exercises.md`.
  - Run through `mock_scenarios.md` (whiteboard or code them).
- Work through **resume-claims/** (especially `INTERVIEW-STRATEGY.md` and the skill-area READMEs). Turn your project work into STAR stories.
- **interview-questionnaire/** — Use the glossary, data structures, core/tricky questions for rapid revision.
- **resources/curated_reading.md** — Prioritize the high-signal links.

### Phase 4: Projects & Portfolio (parallel from Phase 2 onward)
Build 2–3 substantial projects that combine pieces:

- A full feed → ring buffer → lock-free LOB → atomic publish pipeline (with replay from Parquet).
- An analytical layer that consumes Arrow batches from the hot path and computes features/imbalance using Polars/DataFusion.
- A tick data storage + backtesting data engine that can replay at 10x–100x real-time speed.
- Something that mixes the original topics with new ones (e.g., low-latency Rust parser feeding a PySpark-style job via Arrow).

Document them with architecture diagrams (ASCII or draw.io), latency numbers, and "what I would change" reflections.

## Daily/Weekly Practice Rhythm (Recommended)

- **Understand**: Read the topic README fully (Why → Interview Focus).
- **Learn**: Study the diagrams/tables + run the demo.
- **Practice**:
  - Modify/extend the demo.
  - Re-implement a core piece from memory (e.g., the seqlock publish or a simple ring buffer).
  - Explain the topic out loud or in writing using the LEARNING_STYLE structure.
- **Apply**: Connect it to a larger project or a mock interview answer.
- **Review**: Re-read key sections after building something that uses the concept.

Measure yourself: Can you explain the topic to a smart friend in 5–10 minutes using the proper terminology, analogies, and trade-offs?

## Non-Technical but Critical

- Linux internals & performance tools (perf, bpftrace, etc.)
- Basic networking and hardware awareness (NUMA, cache lines, NIC features)
- Observability culture (always measure distributions, not averages)
- Trade-off thinking (latency vs throughput vs complexity vs correctness)

## Tracking Progress

- Mark topics as "Read + Demo Run + Extended + Explained".
- Maintain a personal "war stories" document (problems you debugged or designs you made while following this).
- Use the `interview-prep/` and `resume-claims/` materials to simulate interviews every 3–4 weeks.

## Final Advice

This material is deliberately deep and opinionated. The goal is not to memorize — it is to develop **intuition** about hot vs cold paths, when to reach for lock-free structures, how data actually flows in a real trading system, and how to make correct, predictable decisions under load.

Start today with `LEARNING_STYLE.md` + Topic 01 + Topic 10. Then follow the order above, building as you go.

Good luck. This path, followed with dedication, will give you a very strong foundation for HFT data engineering and systems roles.