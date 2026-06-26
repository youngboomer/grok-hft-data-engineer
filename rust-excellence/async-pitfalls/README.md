# Async Rust Pitfalls in High-Frequency Trading

## The Core Tension

Async (Tokio, async-std, etc.) is excellent for high-concurrency I/O-bound workloads with thousands of connections. It is **often not** the right tool for the absolute lowest tail latency hot path in a market maker.

## Common Problems

### 1. Task Scheduling Jitter
The executor decides when your task runs. Even with `tokio::task::yield_now()` or priorities, there can be surprising delays.

### 2. Waker and Allocation Overhead
Many async primitives allocate on first poll or when creating wakers. This can happen at inconvenient times.

### 3. Mixing Hot and Cold Work
It's very easy to accidentally put market data processing on the same runtime as reconnect logic, metrics, or admin HTTP.

### 4. `async fn` in Hot Paths
Every `await` point is a potential scheduling decision. A hot path with several awaits can have much worse tail latency than an equivalent synchronous version on a dedicated thread.

## When Async Is Fine or Even Good

- Cold path work: REST calls, snapshot fetching, listenKey management, control plane, logging.
- High connection count scenarios (many symbols + many users in a different architecture).
- Glue code between components.

## Recommended Pattern for Most Binance Market Makers

- Dedicated OS threads (pinned) for the true hot path: feed handler, main strategy loop, OMS send path.
- Use a normal async runtime (Tokio) for everything else.
- Communicate between the two worlds with lock-free queues or atomics.

## How to Talk About This in Interviews

A strong answer:
"I use async runtimes heavily for the cold and warm paths because they make I/O and concurrency much easier to reason about. For the market data hot path I deliberately use plain threads + atomics + bounded lock-free queues because I can control scheduling, avoid waker overhead, and pin to cores. The two worlds communicate through very narrow, carefully designed interfaces."

This shows you understand the trade-offs instead of cargo-culting "async is always faster."
