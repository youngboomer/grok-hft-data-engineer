# 07 — Market Data and Order Book Management

## Why This Matters

Your local order book is the single source of truth for every pricing and risk decision. If it is wrong or slow to update you will quote on stale prices or miss hedges.

## Core Requirements

- Ingest snapshots + deltas correctly
- Handle gaps (sequence number jumps)
- Provide extremely fast top-of-book + imbalance views to the strategy
- Stay correct and fast even when Binance sends thousands of updates per second

## Data Flow (ASCII)

```
WS frame
   │
   ▼
parse (zero-copy where possible)
   │
   ▼
if gap detected → request snapshot (cold path)
   │
   ▼
apply delta to local book (careful with levels)
   │
   ▼
publish new best / imbalance (atomic or lock-free)
```

## Recommended Structures

- Price levels as sorted arrays or maps with good cache behavior for the active range.
- For many symbols: per-symbol book with pre-allocated levels.
- Fast "top of book + seq" view published via atomics for strategy.

## Demo

A mini order book that ingests synthetic Binance-style updates and measures update latency.

See `demo_code/mini-orderbook/`.
