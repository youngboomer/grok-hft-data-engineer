# Interview Questions by Topic

## 01 System Architecture

- Walk through the complete tick-to-trade path from Binance WS frame to order on the wire in your system.
- How do you size and isolate threads for a 20-symbol market maker?
- Design a feedback loop from fills to quoting that cannot introduce jitter into the hot path.

## 02 Rust for Low Latency

- Explain ownership, moves, and borrowing using an example from order book or market data handling.
- How do you guarantee (or come close to guaranteeing) zero allocations on the hot path?
- What release profile flags do you set and why?

## 03 Concurrency & Lock-Free

- Explain Acquire/Release memory ordering with a concrete publish of best bid/ask.
- Design a lock-free way for N readers to see the latest top-of-book without ever blocking the writer.
- When would you choose a dedicated thread model over a single async runtime for market data?

## 04 Memory & Zero-Copy

- Compare SmallVec vs Vec<Level> for maintaining the top 20 levels of a book.
- How would you design an arena for per-symbol temporary working data during a burst?

## 05 Linux Networking

- Which socket options are essential for low latency to Binance and why?
- Describe a reconnect strategy that never loses fills and never sends duplicate orders.

## 06 Exchange Protocols

- Explain exactly how Binance depth snapshot + delta sequencing works.
- How do you authenticate and maintain a user data stream for execution reports?

## 07 Order Book Management

- Implement (on a whiteboard) a minimal book that correctly handles snapshots and deltas.
- How do you detect and recover from gaps without introducing long periods of bad quotes?

## 08 OMS, Strategy, Hedging

- Model an order state machine that makes duplicate orders or lost fills impossible.
- How does inventory from one venue affect quoting and hedging on another?

## 09 Perf & Tooling

- How do you measure and report p99.9 tick-to-trade latency under burst load?
- Describe a golden wire-format test you would write for a depth parser.

## Cross-Topic / System Design

- Design the full concurrency and data ownership model for a crypto MM on Binance + one other venue.
- Your p99 latency just tripled during a large liquidation event. Walk through your debugging process.
- How do you structure a codebase so that the hot path stays obviously fast even as the system grows?
