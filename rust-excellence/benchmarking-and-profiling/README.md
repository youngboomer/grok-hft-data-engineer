# Benchmarking, Microbenchmarks, and Production Profiling

## Microbenchmarks Are Necessary But Dangerous

`criterion` or `divan` are great for comparing small functions.

**Problems**:
- They often run in a very different environment than your real system (different allocator pressure, different cache state, no other threads).
- They can make you optimize the wrong thing.

**Rule**: Use microbenchmarks to guide local improvements. Use end-to-end realistic load tests to decide if the improvement actually matters.

## Good Practices for HFT Benchmarks

- Always `--release`.
- Warm up properly.
- Measure under load that mimics bursts (not steady state).
- Include the full realistic work (parsing + book update + decision + simulated send), not just one function.
- Use `black_box` correctly.

## Production-Grade Measurement

The numbers that matter are taken from the real running system under real market conditions.

Techniques:
- High-resolution timestamps around real work (using `Instant` or `std::time` with care about monotonicity).
- Atomic histograms or a dedicated low-overhead sample ring.
- Export on a separate thread or via a side channel.
- Tag samples with context (symbol, burst flag, time since reconnect, etc.).

## Tools Worth Mastering

- `perf` + flamegraphs
- `cargo asm` / `cargo objdump`
- `dhat` for allocation profiling
- `bpftrace` for production tracing without stopping the process
- `hdrhistogram` (used carefully)
- Custom atomic histograms

## Example: What a Good Benchmark Story Sounds Like

"We saw that our book update function was showing up in flamegraphs during bursts. We wrote a criterion benchmark that replayed recorded depth bursts. We then measured the full pipeline (receive → parse → update → publish) in a multi-threaded harness that simulated real load. After switching from BTreeMap to a sorted pre-sized Vec for the top levels, the p99 of the synthetic harness dropped 18%. We then deployed and confirmed similar improvement in the real p99 numbers during volatility."

This level of rigor is what strong candidates demonstrate.
