# Systems & Performance Engineering

This section covers claiming deep expertise in building high-performance, low-latency systems, especially using Rust for performance-critical components, zero-copy techniques, and thoughtful system design.

## Rust with PyO3 / Maturin (Python + Rust Interop)

### Why This Matters
Python is great for productivity and the data ecosystem (Pandas, PySpark, etc.). Rust is great for performance, memory safety, and zero-cost abstractions. PyO3 + Maturin lets you write the hot path or performance-sensitive logic in Rust while keeping the high-level orchestration in Python. This is extremely powerful for data pipelines, low-latency services, and replacing slow Python loops or C extensions.

### What Problem Does It Solve?
- Python GIL and interpreter overhead in hot loops
- Need for high-performance parsing, computation, or low-latency networking while staying in the Python ecosystem
- Writing safe, fast extensions without dealing with CPython C API complexity
- Gradual migration or hybrid systems (Python for glue + Rust for speed)

### How It Works (Internals)
- **PyO3**: Rust crate that provides safe bindings to the Python C API. Uses Rust's ownership model to prevent common C extension bugs (use-after-free, GIL issues).
- **Maturin**: Build tool that compiles Rust into Python wheels. Handles cross-compilation, manylinux wheels, and publishing to PyPI.
- You define `#[pyfunction]` and `#[pymodule]`.
- Data exchange: PyO3 can accept Python objects, convert to Rust types (with zero-copy where possible using `PyBytes`, numpy arrays via `numpy` crate, or Arrow via `pyarrow`).
- GIL management: `Python::with_gil` or release the GIL for long-running Rust work with `Python::allow_threads`.

**Simple Example Structure**:
```rust
// lib.rs
use pyo3::prelude::*;

#[pyfunction]
fn fast_parse(data: &[u8]) -> PyResult<Vec<u64>> {
    // zero-copy friendly processing here
    Ok(process(data))
}

#[pymodule]
fn my_fast_module(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(fast_parse, m)?)?;
    Ok(())
}
```

Build with `maturin develop` or `maturin build --release`.

### Applicability & Best Practices
- Use for **hot paths**: parsing, aggregation, complex math, low-latency protocol handling.
- Keep Python for orchestration, I/O, and data science libraries.
- For data interchange:
  - `bytes` / `PyBytes` for zero-copy when possible.
  - Arrow (via `pyarrow` + `arrow` crate) for columnar data — excellent with Pandas/Polars.
  - NumPy arrays via the `numpy` crate (zero-copy views).
- Release the GIL for CPU-bound work so Python threads can run.
- Error handling: Convert Rust errors to Python exceptions cleanly.

**Gotchas & Pitfalls**:
- Holding the GIL during long computations → other Python threads blocked.
- Unnecessary data copies on the boundary (biggest performance killer).
- Using Python objects inside hot Rust loops (attribute access is slow).
- Version mismatches between Rust Python and target Python.
- Debugging mixed stack traces can be painful — use logging from both sides.

**When to Use vs Not**:
- Use: When Python profiling shows a clear hot function that is 10x+ slower than it should be.
- Not: For simple glue code or when development speed matters more than runtime (pure Python or Cython may be enough).
- Alternative: Cython (faster to write for some), or pure Rust binary + subprocess/FFI, or Polars/Pandas extensions.

### Interview Talking Points & Questions
**Strong answer structure**:
- Context of the project (e.g., "We had a Python ETL pipeline processing 10M+ messages/sec where parsing was the bottleneck").
- Why Rust: memory safety + performance + easy to expose via PyO3.
- Key decisions: zero-copy with Arrow, releasing GIL, careful boundary design.
- Results: "Reduced parsing latency from 2ms to 40µs per batch, enabling higher throughput".
- Tradeoffs discussed: development time vs runtime, testing complexity.

**Common questions**:
- "How do you avoid copying data when passing large arrays from Python to Rust?"
- "What is the GIL and how do you handle it in PyO3?"
- "Walk through how you would expose a high-performance order book or parser from Rust to a Python strategy layer."
- "Compare PyO3/Maturin vs Cython vs writing a separate service."

**Resume claim examples**:
- "Built performance-critical parsing and stateful processing modules in Rust exposed via PyO3/Maturin, achieving 50x speedup over pure Python equivalents while maintaining seamless integration with existing Pandas/Polars pipelines."

## Zero-Copy Parsing

### Why This Matters
In high-throughput or low-latency systems, every memcpy hurts. Zero-copy means operating on the original buffer (network packet, file mmap, Arrow buffer) without creating intermediate copies.

### What Problem Does It Solve?
Copying large payloads (market data, logs, events, Parquet chunks) creates CPU, memory bandwidth, and cache pressure. It also increases latency and GC/allocation jitter.

### How It Works
- Use memory-mapped files (`mmap`) or direct buffer views.
- In Rust: `&[u8]` slices, `bytes::Bytes`, `std::io::Cursor`.
- Libraries: `nom`, `winnow`, or manual offset-based parsing for binary protocols.
- For Python: `memoryview`, `numpy.frombuffer`, Arrow `RecordBatch` from buffers.
- PyO3 can accept `&[u8]` or `PyBytes` and return views or small extracted `Copy` types.

**Example Mental Model**:
Think of a library book. Zero-copy is reading the book while it's still on the shelf (taking notes on specific pages). Copying is photocopying the whole book first.

### Applicability
- Network protocols (WebSocket frames, FIX, custom binary)
- File formats (Parquet, CSV with chunked reading, binary logs)
- Message queues (consume from ZMQ/Redis without extra copies)

**Gotchas**:
- Lifetime management (the buffer must outlive the view).
- Alignment and endianness issues in binary parsing.
- Python's `bytes` are immutable but `memoryview` helps.
- In distributed systems, zero-copy across network (RDMA, DPDK) is next level.

**Interview Questions**:
- "Explain zero-copy parsing and give an example of where you used it to reduce latency."
- "How do you handle a partially received message in a zero-copy parser?"

## Performance Profiling, Memory & Latency Optimization

### Key Tools & Techniques
- **Rust**: `perf`, `flamegraph`, `cargo flamegraph`, `dhat` (heap), `criterion` for microbenchmarks, `cargo asm`.
- **Python**: `cProfile`, `py-spy`, `scalene`, `memory_profiler`, `line_profiler`.
- **System**: `perf stat`, `bpftrace`, `strace`, `valgrind` (cachegrind), `htop` + `pidstat`.
- **Low-latency mindset**: Measure p99/p99.9 under burst load, not averages. Distinguish allocation, cache misses, branch mispredictions, syscalls, lock contention.

**Techniques**:
- Pre-allocation and object pooling / arenas
- SmallVec / stack allocation
- False sharing avoidance (cache line padding)
- Lock-free structures where possible
- SIMD where it actually helps
- Reducing syscalls (batching, io_uring where available)
- Memory mapping for large datasets

**Common Interview Deep Dive**:
Be ready to draw a simple flamegraph or explain "I saw X function taking 40% in a burst, replaced Y with Z, got 3x improvement because..."

## Low-Latency Data Streams & System Design

Connects to previous HFT work:
- Dedicated threads for hot paths vs async for cold paths
- Lock-free publication (atomics, seqlocks, SPSC rings)
- Backpressure handling
- Zero-copy + batching tradeoffs
- End-to-end measurement (timestamp on receive → decision → send)

**System Design Approach**:
Always separate hot path (predictable, minimal work) from cold path. Use diagrams for thread boundaries and data ownership.

Example ASCII for a hybrid system:
```
[Network / ZMQ] → [Rust Zero-Copy Parser (hot thread)] → Atomic Snapshot
                                                      ↓
[Python Strategy (Polars/Pandas)] ← reads latest state
                                                      ↓
[FastAPI / Rust OMS] → [Redis / Kafka] for durability
```

### Interview Focus
- "Design a low-latency data ingestion pipeline that feeds both real-time decisions and a batch analytics layer."
- "How would you profile and optimize a system that's fast in tests but has bad p99 in production under load?"
- Be ready to discuss tradeoffs between Rust, C++, Go, Python for different parts of the system.

## Summary for Resume Claims
You should be able to say:
"I used Rust via PyO3/Maturin to implement zero-copy parsing and state machines for low-latency streams. This reduced end-to-end latency by >10x compared to pure Python while integrating cleanly with our PySpark and Polars analytics layers. I profiled using perf and py-spy, identified allocation and GIL contention, and applied pre-allocation + GIL release patterns."
