# How This User Likes to Learn (For LLMs)

This document describes the preferred learning and explanation style for the owner of the `grok-hft-data-engineer` repository.

## Core Structure for Any Technical Topic

Always follow this order:

1. **Why This Matters** (in real ultra-low-latency crypto trading / HFT market making / hedging)
2. **What Problem Does It Solve?**
3. **How Does It Solve It?** (mechanisms, internals)
4. **Applicability**
   - How to apply it effectively (step-by-step, with concrete examples)
   - Key gotchas and pitfalls (especially tail latency, jitter, state corruption, crashes under load)
   - When to use vs when NOT to use + better alternatives
5. **Visuals** when helpful:
   - ASCII DFD (Data Flow Diagrams) and SDD (System/Structure Design Diagrams) using box-drawing characters
   - Clear tables for data structure / primitive comparisons (latency, predictability, cache behavior, hot-path suitability, complexity, trading example)
6. **Hot Path vs Cold Path** distinction (always call this out explicitly)
7. **Analogies** — Explain complex concepts (especially Rust ownership, memory ordering, concurrency) like you are teaching a very smart, curious 10-year-old. Use accurate, intuitive analogies (library books, toys, traffic lights, newspapers, emergency lanes, etc.). Then immediately connect the analogy back to the trading system implication.
8. **Binance / Real Trading Grounding** — Use Binance WebSocket depth, aggTrade, user data streams, snapshot+delta rules, HMAC orders, rate limits, and reconnect realities as the primary concrete examples.
9. **Interview Focus** — End relevant sections with realistic interview questions + what a strong answer demonstrates.

## Key Emphasis Areas

- **Predictable low tail latency** (p99 / p99.9) and **correctness under burst load** are more important than average-case speed.
- **Hot path is sacred**: zero or near-zero allocations, minimal branches, no locks on publication paths, cache-friendly layouts.
- **Feedback loops** matter (fills → inventory → quoting/hedging decisions).
- Rust explanations must be deep and practical: ownership, borrowing, lifetimes, moves vs clones, atomics + memory ordering, `SmallVec`/`bytes`, panic=abort, release profiles, etc.
- Distinguish between "convenient" code and "production HFT under volatility" code.

## Code & Examples Preferences

- Prefer runnable, minimal examples (single-file `.rs` with dependency comments, or small self-contained Cargo projects).
- Always discuss `--release` behavior.
- Include comments explaining the "why" and hot-path considerations.
- Show both the good pattern and the common bad pattern that creates jitter or bugs.

## Anti-Patterns to Avoid When Explaining

- Long walls of theory without immediate practical connection to tick-to-trade or Binance.
- Only showing p50 / average numbers.
- Ignoring reconnect, gap handling, or burst scenarios.
- Using vague "it's faster" without mechanisms or trade-offs.
- Forgetting to call out when something would be disastrous on the hot path.

## Overall Goal

The user wants to **deeply understand** systems so they can:
- Design and debug real ultra-low-latency trading engines
- Perform well in technical interviews for HFT/crypto market making roles
- Build correct, predictable, maintainable code

Explanations should feel rigorous, encouraging, honest about trade-offs, and directly useful for someone who will work on production systems where microseconds and correctness under load determine P&L.

---

When in doubt, structure your response using the 9 points above and ask: "Would a smart person reading this be able to implement something better and explain it clearly in an interview?"
