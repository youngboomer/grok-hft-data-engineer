# Zero-Copy + SmallVec Demo

Demonstrates the dramatic difference between a hot-path friendly pattern and a "works but will bite you" pattern when ingesting market data.

## What It Shows

- Using `bytes::Bytes` + borrowing to avoid copying the entire incoming frame.
- `SmallVec<[T; N]>` so that the common case (top 16 levels) never allocates.
- The cost of a naive `to_vec()` clone on every update.

## Run

```bash
cd topics/02_rust_for_ultra_low_latency/demo_code/zero-copy-smallvec
cargo run --release
```

## Key Takeaway for Binance Work

Binance depth streams regularly send 100–1000+ levels. If your per-update code path allocates or copies the whole thing, you will see it in the tail latency numbers the moment real money starts trading.

The fast version in this demo does almost zero work per update beyond what is strictly required.
