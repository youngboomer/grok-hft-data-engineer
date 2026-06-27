# Interview Questionnaire — Deep Dive

**Purpose**: Turn knowledge into interview-ready mastery and real engineering intuition.

This is **not** a list of flashcards. Every topic is explored using the same learning philosophy used throughout this repository:

1. **Why it matters** in real ultra-low-latency crypto market making / hedging
2. **What** problem it solves
3. **How** it solves it (mechanisms, internals, data flow)
4. **Applicability**
   - How to apply it effectively
   - Key gotchas / pitfalls (especially tail latency, jitter, correctness under load)
   - When to use vs when NOT to use
5. Clear **analogies** (explained like to a very smart 10-year-old)
6. **Data structure comparisons**, ASCII diagrams, and concrete Binance-relevant examples
7. **Interview angle**: What a strong answer demonstrates vs weak answers

Use this material to:
- Prepare for system design + deep technical rounds
- Build genuine understanding (not just memorization)
- Practice explaining concepts out loud

---

## How to Use This Material

1. Read the **HFT Terminology Glossary** first — language is power.
2. Study **Data Structures** deeply (this is where many candidates lose points).
3. Go through **Core Questions** and **Tricky Questions**. 
   - Try to answer out loud or on a whiteboard **before** reading the model answer.
4. Do the **Practice Scenarios**.
5. Use the **Quick Reference** the night before an interview.

Everything here assumes you have already worked through the main 9 topics in the repository.

## Structure

- `hft-terminology/glossary.md` — 30+ critical terms with meaning + system impact
- `data-structures/` — Common + HFT-specific structures with trade-off tables
- `core-questions/` — In-depth answers for frequently asked areas
- `tricky-questions/` — The questions that separate strong from average candidates
- `practice-scenarios/` — Whiteboard / live-coding style exercises
- `quick-reference/` — Cheat sheets for rapid review

---

**Remember the job reality**: The interviewer is not testing whether you can recite definitions. They are testing whether you can design and debug a system that stays correct and fast when Binance is sending 10k+ updates/sec during a liquidation cascade, while your inventory is moving, and one of your connections just dropped.

Let's get you ready.
