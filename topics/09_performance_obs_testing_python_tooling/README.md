# 09 — Performance, Observability, Testing, and Python Tooling

## Why This Matters

You cannot improve what you do not measure. In this domain you must measure distributions (p50, p95, p99, p99.9) under realistic burst load, not just averages on a quiet machine.

## Cargo Release Tuning

See the recommended `[profile.release]` settings in the main README and topic 02.

## Low-Overhead Metrics in Rust

Use atomics for counters and for simple histograms (atomic buckets). Avoid locks in the hot measurement path.

## Python for Analysis

Python + Polars or pandas + matplotlib/seaborn is excellent for post-processing latency logs, building histograms, and detecting anomalies.

## Golden Wire Format Tests

Serialize a known message once, commit the bytes, and assert that your parser produces exactly the expected structure on every change. This catches protocol drift early.

## Demo

- `demo_code/latency-histogram/` (Rust side that writes a log)
- `demo_code/analyze_latency.py` (Python script that reads the log and prints percentiles + simple histogram)

See the demo folders.
