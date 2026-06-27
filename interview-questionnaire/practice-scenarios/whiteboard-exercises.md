# Practice Scenarios & Whiteboard Exercises

Do these out loud or on paper. Time yourself.

## Scenario 1: 15-Symbol Binance Market Maker Concurrency Model

**Prompt**:
Design the threading, data ownership, and communication model for a market maker running 15 liquid symbols on Binance Spot. You also hedge inventory moves on Deribit.

**Deliverables**:
- Draw the threads / async runtimes and which cores they should prefer.
- Show how market data reaches pricing.
- Show exactly how a fill on Binance updates inventory and affects both Binance quoting and Deribit hedging.
- Explain what happens on a depth stream disconnect for one symbol.
- Choose your publication mechanisms (atomics? channels? both?) and justify.

**Evaluation Criteria** (what a good answer covers):
- Clear hot/cold separation
- Non-blocking publication of market data
- Fast but safe feedback loop for inventory
- Reconnect strategy that doesn't create duplicates or lose fills
- Awareness of rate limits and backpressure

---

## Scenario 2: Debug 10x Tail Latency Spike

**Prompt**:
p99 tick-to-trade is normally ~70µs. During the first 30 seconds of any major liquidation event it jumps to 700-900µs. CPU is not saturated. What do you do?

**Approach**:
1. What metrics / histograms would you already have in production?
2. What would you add temporarily to debug?
3. Walk through the most likely 3-4 root causes in order of probability.
4. How would you reproduce this in a controlled environment?

---

## Scenario 3: Design an Allocation-Free Top-N Order Book Updater

**Prompt**:
Implement (in pseudocode or real Rust) the hot path of a component that receives Binance-style depth deltas and maintains only the top 20 bids and top 20 asks.

Constraints:
- No allocations on the update path after initialization.
- Must publish new best bid/ask + a simple imbalance metric.
- Must be safe for multiple readers.

**Things to discuss**:
- Data structure choice
- How you apply an update to a sorted list efficiently
- How you publish the result
- What you do when the book changes at level 21 (do you care?)

---

## Scenario 4: Cross-Venue Inventory Consistency

You are long 2.3 BTC on Binance after a series of fills. You want to send a hedge on Deribit within 200µs of the last fill.

- How is the fill information propagated?
- How do you avoid over-hedging if you receive multiple partial fills quickly?
- What happens if the Deribit connection is slow or rate limited?
- How do you handle the case where Binance and Deribit use different precisions and symbols?

---

## Scenario 5: "Make this code fast and safe"

You are given a piece of code that uses `Arc<RwLock<OrderBook>>` shared between the feed thread and three strategy threads.

**Tasks**:
- Identify all the latency and correctness problems.
- Redesign the publication mechanism.
- Show how the new design affects the failure modes (one slow strategy, reconnect, etc.).

This is a classic "find the bad abstraction" interview question.
