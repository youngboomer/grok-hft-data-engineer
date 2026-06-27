# HFT Data Engineer Interview Cheatsheet

## Core Mental Models
- **Hot path is sacred**: zero allocations, minimal branches, no locks on publish, cache-friendly. Everything else is cold.
- **Correctness under burst > micro-benchmark speed**.
- **Snapshot + Delta + Sequence** is the universal pattern for market data.
- **Measure distributions** (p99, p99.9), not averages. Always under load.

## Key Numbers to Have Ready
- Typical tick-to-decision targets: < 100µs p99 for hot crypto MM.
- What "fast" looks like for ring buffer publish, seqlock read, etc.
- Your personal project metrics (always have 2-3 concrete numbers + what changed them).

## Common Interview Questions + Strong Answer Starters
1. "Walk through your market data pipeline from wire to strategy decision."
   - "Hot path in Rust: zero-copy parse → lock-free ring → book reconstruction using flat arrays + seqlock snapshot. Cold path handles storage and heavy Polars features via Arrow handoff."

2. "How do you handle reconnects and gaps without bad state?"
   - "Strict sequence checking. On gap → mark stale, cold snapshot + replay deltas. Only resume publishing when consistent."

3. "How did you deal with skew in your Spark/PySpark job?"
   - "Statistical detection + domain knowledge (hot symbols). Salting + custom partitioners + range repartition for price levels."

4. "Rust vs Python for this part?"
   - "Hot path (parsing, state update, low-lat publish) → Rust. Analytics, orchestration, backtesting prep → Python/Polars when the latency budget allows."

5. "Tell me about a time performance was bad under load."
   - Use one of the war stories. Always end with numbers, root cause, and what you learned about hot/cold separation.

## Quick Tradeoff Language
- "We chose a bounded ring buffer over a channel because we needed zero-allocation and the ability to drop updates rather than block the producer."
- "Arrow gave us zero-copy between Rust and Polars without giving up type safety or performance."
- "For this workload, the hot path had to be single-writer. Multiple writers would have required too much synchronization."

## Red Flags to Avoid Saying
- "It was fast enough on my laptop."
- "We just used Kafka / Spark for everything."
- "Averages were good."
- "We didn't really measure reconnect scenarios."

## Good Questions to Ask Interviewers
- "What does your hot path vs cold path split look like today?"
- "How do you currently handle book reconstruction and gap recovery?"
- "What are the biggest sources of tail latency in your data pipelines right now?"

Print this or have it handy for last-minute review. Good luck!
