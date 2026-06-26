# 08 — OMS, Strategy, and Hedging Engines

## Why This Matters

The OMS is responsible for the actual money-moving actions. It must be correct above all else, fast enough not to be the bottleneck, and able to survive disconnects and rate limits without creating duplicates or losing fills.

Strategy must turn book state + inventory into good quotes, and hedging logic must keep overall risk under control across venues.

## Core Concepts

- Order lifecycle as a Rust enum + exhaustive match (New, PartiallyFilled, Filled, Canceled, Rejected...).
- Client order ID generation that is unique and allows idempotent resends.
- Rate limiting (token bucket is common).
- Inventory as the source of truth that feeds back into quoting.
- Cross-venue hedge signals generated from net exposure.

## Feedback Loop (Critical)

Fills arrive on the private user data stream → update inventory atomically or via short critical section → next strategy decision sees updated inventory and skews or cancels quotes accordingly.

## Demo

`demo_code/oms-state-machine/` — a clean enum-based order state machine with simulated fills and a tiny inventory feedback example.

See demo folder.
