# Curated Reading & Resources

High-signal resources only. Prioritize things that have directly helped people build real low-latency trading systems.

## Rust Performance & Low-Level

- Rust Performance Book — https://nnethercote.github.io/perf-book/
- "Rust for Rustaceans" (Jon Gjengset) — especially ownership, unsafe, concurrency chapters
- `smallvec`, `bytes`, `crossbeam` crate documentation + source
- "Zero-cost abstractions" talks (various Rust team members)

## Concurrency & Systems

- LMAX Disruptor paper and blogs
- "The Tail at Scale" (Jeff Dean, Google)
- "Designing Data-Intensive Applications" — streaming, consistency, and replication chapters
- Linux `perf`, `bpftrace`, `htop` + `taskset` / `isolcpus` usage guides

## Networking & Linux

- "TCP/IP Illustrated"
- `man 7 socket`, `man 7 tcp`
- Articles on `TCP_NODELAY`, `SO_BUSY_POLL`, kernel bypass considerations
- `ethtool` and NIC ring buffer tuning guides

## Market Data & Exchange Specific

- Official Binance Spot WebSocket API documentation (depth streams, user data stream, listenKey, snapshot rules)
- Binance API error codes and rate limit documentation
- "Order book" implementations in various open source market making bots (study the good ones, not the toy ones)
- FIX and SBE protocol specifications (for when you encounter binary protocols)

## Observability & Measurement

- `hdrhistogram` concepts (even if you implement a simple atomic version)
- `dhat`, `heaptrack`, `perf` + flamegraph workflows
- Python Polars + matplotlib for latency analysis pipelines

## General Engineering Discipline

- "The Checklist Manifesto"
- Postmortems from trading firms and HFT shops (public ones)
- "Site Reliability Engineering" (Google) — error budgets and toil reduction thinking applies directly to trading systems

Add anything you discover that gave you a genuine "aha" moment while building or debugging these systems.
