# Performance Tooling for Rust in HFT

You cannot optimize what you cannot measure.

## Essential Tools

### 1. Always Benchmark in Release

```bash
cargo run --release
cargo bench
```

Debug builds have completely different characteristics (no inlining, different codegen, bounds checks everywhere).

### 2. cargo expand, cargo asm

```bash
cargo install cargo-expand cargo-asm
cargo expand
cargo asm --rust my_hot_function
```

Seeing the actual generated assembly for your hot path is incredibly educational.

### 3. Linux perf + flamegraphs

```bash
perf record -F 9999 --call-graph dwarf ./target/release/my_binary
perf script | stackcollapse-perf.pl | flamegraph.pl > flame.svg
```

### 4. Allocation Tracking

- `dhat` (Rust allocator profiler)
- `heaptrack`
- Custom counters around hot sections

### 5. Low-Overhead Histograms in Production

Use atomics to maintain simple histograms of tick-to-trade, decision latency, etc. Export them periodically on a cold path.

Never use a heavy profiling framework on your actual hot path in production.

## What Good Measurement Looks Like

- You have p50 / p95 / p99 / p99.9 numbers under realistic burst load.
- You can attribute latency to specific stages (parse, book update, decision, OMS send).
- You can reproduce tail latency in a controlled synthetic load test.

## Exercise

Take one of the small demos from the main repository and:
1. Run it under `perf`.
2. Generate a flamegraph.
3. Identify the single most expensive thing in the hot path.
4. Try to improve it and measure again.
