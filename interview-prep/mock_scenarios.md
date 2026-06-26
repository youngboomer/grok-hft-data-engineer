# Mock Interview Scenarios

## Scenario 1: Concurrency Model for 20-Symbol Binance Market Maker

You are building a market maker on Binance Spot for 20 liquid symbols.
You also hedge on a second venue.

Requirements:
- Must handle 5k–15k depth updates/sec per symbol during spikes.
- Must never block market data ingestion.
- Must react to own fills within a few hundred microseconds for inventory skew.
- Must survive WS disconnects without creating duplicate orders or bad inventory.

**Tasks for the candidate**:
- Draw the thread / runtime layout.
- Choose publication mechanisms between components.
- Explain exactly how fills flow back into quoting.
- Describe your reconnect + snapshot strategy in detail.

## Scenario 2: Debug High Tail Latency

Production p99 tick-to-trade is 80µs most of the day.
During high volatility it jumps to 1.4ms for minutes at a time.

You have flamegraphs and some atomic histograms.

**What do you look for first?**
What experiments do you run? How do you reproduce the problem in a controlled way?

## Scenario 3: Reconnect + Deduplication That Never Loses Fills

Design (and sketch the code for) the logic that:
- Detects a dropped depth stream.
- Fetches a snapshot on REST without blocking the main feed.
- Replays buffered deltas safely.
- Correlates execution reports from the user data stream to local order state so that a reconnect never causes duplicate orders or missed position updates.

## Scenario 4: Cross-Venue Hedge Signal Design

You are long on Binance and the market is moving against you.
You need to send a hedge order on Deribit within low microseconds of the fill arriving on Binance.

- How is the fill published?
- How does the hedge decision get made without polluting the Binance hot path?
- What safety mechanisms prevent over-hedging?
- How do you handle the case where the hedge venue is slower or rate-limited?
