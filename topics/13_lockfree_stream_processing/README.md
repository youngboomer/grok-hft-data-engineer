# 13 — Lock-Free Stream Processing for HFT Data Pipelines

## Why This Matters

The heart of an HFT data pipeline is moving events from producers (feeds) to consumers (strategies, risk) with minimal latency and zero contention on the hot path.

General purpose channels and queues introduce hidden costs: allocations, locks, context switches.

## What Problem Does It Solve?

Need to fan-out one high-rate stream to many readers without the producer ever blocking or slowing down.

## How It Solves It

### The Disruptor Pattern (LMAX)

Classic design:
- Single ring buffer pre-allocated
- Producers claim slots
- Consumers have their own sequence cursors
- Wait strategies (busy spin, yield, park) for different latency/CPU tradeoffs

### Rust Implementation Approach

Reference:
- `disruptor-rs`
- Custom implementations in the ecosystem

From-scratch version in demo:
- Multi-consumer support
- Different wait strategies
- Batch claiming for producers

## Applicability

Use in:
- Feed handler publishing to strategy + risk + logging
- Normalized data fan-out inside the trading process

## Demo

See `demo_code/` for a working multi-consumer Disruptor-style processor in Rust, with Python consumer simulation.

## Gaps

- Full Aeron integration (briefly explained)
- Hardware offload
- Production wait strategy tuning
