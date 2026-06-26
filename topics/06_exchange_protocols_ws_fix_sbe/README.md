# 06 — Exchange Protocols: WebSocket, FIX, SBE

## Why This Matters

Binance (and most crypto venues) primarily use WebSocket for market data and user data. Understanding the exact message formats, heartbeat requirements, authentication, and reconnect rules is mandatory for a correct and low-latency system.

## Key Binance WS Patterns You Must Master

- Depth streams (`@depth@100ms` or `@depth@1000ms`)
- `lastUpdateId` and how to apply snapshots vs deltas correctly
- aggTrade stream
- User data stream (listenKey, signed, for execution reports and balance updates)
- Heartbeats (ping/pong at protocol level + application level)

## HMAC for Authenticated REST and Signed WS

Never implement HMAC yourself from scratch for orders. Use well-reviewed crates and constant-time comparison where relevant.

## Binary Protocols (SBE / FIX)

Some venues or internal links use Simple Binary Encoding (SBE) or FIX. SBE is particularly interesting because it is designed for zero-copy decoding with predictable latency.

## Demo

`demo_code/binance-depth-parser/` — a small educational parser that shows how you would handle a simplified depth update and apply it to a tiny book while respecting sequence rules.

See the demo README.
