# Realistic "War Story" Style Questions

These simulate real production incidents or design reviews.

## Scenario: The Mysterious 2ms Spikes

**Prompt**:
Your p99 tick-to-trade is usually excellent (<100µs). Every 30-60 seconds you see a 1.5–3ms spike on one or two symbols. The spikes do not correlate with market volatility. CPU usage is moderate.

**What an interviewer wants you to explore**:
- Is it a specific symbol or random?
- Is it correlated with other activity (logging rotation, metrics export, background reconciliation, listenKey refresh)?
- Is one of your cold paths occasionally doing something expensive that shares resources (allocator, network, CPU)?
- NUMA or core migration?
- Page faults or memory compaction?
- TCP retransmits or NIC buffer issues?
- Your own code doing a rare but expensive operation (e.g. growing a HashMap, cloning something big on first use of a symbol).

**Good response**:
Describe how you would add more granular per-stage histograms + context (symbol, time since last reconnect, etc.) and then go look at what else is running on the same machine or in the same process at that cadence.

## Scenario: The Duplicate Hedge

You sent a hedge on Deribit, but your inventory still shows you are exposed. Later you discover two hedge orders were sent for what should have been one move.

**Root cause categories to discuss**:
- Inventory update happened after the hedging decision was made.
- Two different fills arrived close together and both triggered hedging logic before either hedge was acknowledged.
- Reconnect logic created a new hedge order while the first one was in flight.
- Missing "hedge in progress" tracking with proper state machine.

## Scenario: Book Looks Fine But Quotes Are Terrible

Your local depth book passes all consistency checks and looks correct in your internal UI, but your quotes are consistently on the wrong side of the market or get immediately picked off.

**Things to investigate**:
- Are you publishing top-of-book from a slightly different view than the one the strategy uses?
- Timing: strategy reads the book before or after certain updates?
- Are you applying deltas in the wrong order for one side (bids vs asks)?
- Floating point precision or price scaling bugs that only manifest on certain price levels.
- You are using the book's "best" but the strategy is using mid or a weighted price that is miscalculated.
- The book is correct but your risk/inventory skew is inverted or using stale inventory.

Strong candidates will say: "I would add the ability to snapshot the exact book state + inventory + decision at the moment a quote was generated, keyed by sequence number."
