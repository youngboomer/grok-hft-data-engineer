# HFT Data Management & Data Pipelines Learning Path

This document (and the associated topics 10–14) extend the core repo with focused, deep material on data management and pipelines for HFT trading roles.

## Philosophy (same as rest of repo)
- Newcomer friendly with clear explanations and analogies
- Keep all technical terminology for accurate mental mapping
- Deep, runnable implementations in Rust (from scratch + reference real crates)
- Python where it complements (analytics, orchestration)
- Brief explanations (no full code) for important non-Rust topics (FPGA, DPDK, Aeron, etc.)
- Tables, ASCII diagrams, hot/cold path emphasis
- Interview-focused questions and strong answers

## Recommended Order

1. **Topic 10: HFT Data Pipelines Architecture** — big picture, layers, technologies overview
2. **Topic 11: Advanced Rust Data Structures** — ring buffers, LOB representations, pools (core primitives)
3. **Topic 13: Lock-free Stream Processing** — Disruptor-style multi-consumer pipelines
4. **Topic 12: Tick Data Management** — storage, replay, persistence
5. **Topic 14: Analytical Data Pipelines** — Arrow, Polars, DataFusion handoff

## Key Technologies Covered

**Rust (deep impls):**
- Lock-free ring buffers & Disruptor patterns
- Efficient LOB structures (flat arrays, seqlocks)
- Object pools / slabs
- Arrow integration

**Python:**
- Polars (lazy) for analytics
- Arrow handoff

**Referenced crates (use in production, study source):**
- rtrb, ringbuf, disruptor-rs
- arrow, parquet, polars, datafusion
- crossbeam, smallvec, bumpalo

**Explained (non-Rust/Python):**
- FPGA / SmartNIC offload
- DPDK kernel bypass
- Aeron transport
- SBE protocol
- kdb+/q (historical context)

## Gaps & Nice-to-Haves Addressed
See individual topic `gaps_and_nice_to_haves.md` and the dedicated one in topic 11.

## How to Use
- Read the topic READMEs in order (10 → 14 recommended).
- Run all demos with `--release`.
- Experiment as suggested in each demo's README.
- Build on top (e.g., wire the ring buffer from topic 11 into the pipeline in topic 13).
- Cross-reference with the main `roadmap_hft_de.md` for full context across the repo.

This gives a dedicated newcomer a complete, practical foundation for HFT data engineering roles while going deep enough to impress in interviews and contribute meaningfully.
