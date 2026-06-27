# Core Questions: System Design & Architecture

## Question: Design a system that can market make 30 symbols on Binance while hedging on a second venue, with a strict p99 tick-to-trade budget of < 150µs in normal conditions.

**Key Areas a Strong Answer Must Cover**:

### Threading Model
- Dedicated feed thread(s) (possibly one per major symbol group).
- Strategy / pricing thread(s).
- OMS / execution thread.
- Background thread or async runtime for everything else.

### Data Flow
- Market data → lock-free publication (atomics or seqlock).
- Strategy reads non-blocking.
- Decisions go through a bounded queue to OMS.
- Fills come back through user data stream → inventory.

### Isolation
- Hot threads pinned to isolated cores.
- Allocator pressure isolated (background threads can allocate freely; hot threads cannot after startup).

### Failure Modes
- One symbol goes crazy → should not affect others.
- Reconnect on one stream → must not corrupt books or create duplicate orders.
- OMS gets slow → strategy should still be able to make decisions (or at least not block feed).

### Observability
- You must be able to answer "what was the book state and inventory at the moment this decision was made?"
- Low-overhead latency histograms on the real path.

**Common Weaknesses**:
- Putting everything in one big Tokio runtime.
- Sharing a big `Arc<Mutex<Book>>` for all symbols.
- No story for what happens when the system falls behind during a spike.

---

## Question: How do you decide what belongs in the hot path vs what can be slower?

**Excellent Answer Framework**:
- Anything that must happen before the next market data update can be used → hot.
- Anything that can be eventually consistent or sampled → cold or warm.
- Reconnect logic is cold.
- Metrics export can be sampled.
- Full book maintenance for deep levels can be "best effort" if the strategy only needs top 10.

**Rule**: If a piece of work can occasionally take 5ms without destroying P&L, it does not belong on the hot path.
