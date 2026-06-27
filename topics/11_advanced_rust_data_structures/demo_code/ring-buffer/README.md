# Ring Buffer Demo (From-Scratch SPSC)

This demo implements a Single-Producer Single-Consumer ring buffer from scratch in Rust.

## Learning Goals
- Understand the mental model of ring buffers (circular train track).
- See why cache-line padding matters.
- Experience zero-copy reads (`peek` + `advance`).
- Compare against real crates (`rtrb`, `ringbuf`).

## How to Run
```bash
cd demo_code/ring-buffer
cargo run --release
```

## Experiments to Try
1. Increase the number of pushes dramatically and time it.
2. Add a second consumer thread (you will see why we need MPMC or careful design).
3. Remove the padding and measure any difference in a contended scenario.
4. Replace the implementation with `rtrb::RingBuffer` and compare API + performance.

There's also a `python_comparison.py` showing the conceptual difference (never use the Python version for real work).

## References
- `rtrb` crate — production-quality realtime ring buffer.
- LMAX Disruptor paper (the spiritual ancestor of many of these designs).
- "Mechanical Sympathy" by Martin Thompson.
