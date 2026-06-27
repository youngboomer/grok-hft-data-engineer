# Rust Excellence for Ultra-Low-Latency HFT

This branch exists to take your Rust skills from "I can write correct code" to "I can write code that is obviously fast, obviously safe on the hot path, and survives production volatility at 3am."

We use the **exact same learning philosophy** as the rest of the repository:

- **Why** this Rust concept matters for tick-to-trade systems
- **What** problem it solves (latency, jitter, correctness, or all three)
- **How** it works internally (with accurate analogies)
- **Applicability** — how to use it, when **not** to use it, common footguns that create tail latency
- Concrete code examples that feel like real trading components
- Emphasis on **predictable performance under load**

---

## Directory Structure

- `ownership-lifetimes/` — The foundation. Moves, borrows, lifetimes in the context of parsers and books.
- `atomics-memory/` — Deep mastery of atomics, memory ordering, and lock-free patterns.
- `allocation-free/` — How to write code that never calls the allocator on the hot path.
- `zero-copy/` — `bytes`, slices, views, and avoiding copies.
- `performance-tooling/` — How to actually prove your code is fast (and find what isn't).
- `examples/` — Self-contained small projects and single-file demos.

## How to Use

1. Read each topic's `README.md` in order.
2. Study the examples. Run them in `--release`.
3. Try the "experiments" suggested in each section.
4. Come back and re-read after you've built something real and hit a performance mystery.

The goal is deep intuition, not just syntax knowledge.

---

**Core Mindset**:
In HFT, Rust's greatest gift is not speed — it is that it **makes many dangerous things compile errors**. Use the type system to make the fast path the only easy path.
