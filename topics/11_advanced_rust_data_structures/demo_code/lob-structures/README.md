# LOB Structures Demo

This demo shows practical data structures for maintaining limit order books in data pipelines.

## What it demonstrates
- Flat array representation (cache-friendly) vs hash-based.
- Snapshot + delta application with sequence checking.
- Publishing top-of-book with atomic seqlock.
- Use of `SmallVec` for top levels.

## Run
```bash
cargo run --release
```

## References & Gaps
Reference: `orderbook-rs`, `rust-order-book`.

Nice-to-have / gaps covered later:
- Full matching engine logic
- Concurrent writers (rare but happens in some risk engines)
- Integration with Arrow for downstream analytics
- Hardware offload concepts (FPGA for book updates — explained in later topics without code)
