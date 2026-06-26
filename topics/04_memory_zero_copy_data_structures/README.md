# 04 — Memory, Zero-Copy, and Data Structures

## Why This Matters in Ultra-Low-Latency Crypto Trading

Every allocation, every copy of a large structure, and every cache miss on the hot path can add jitter that appears in your p99 tick-to-trade numbers. Under burst load from Binance this becomes the difference between being a profitable market maker and one that occasionally blows out.

## What Problem Does It Solve?

- Allocator contention and pauses when many threads allocate at the same time during volatility.
- Cache pollution from copying large messages that only a small part of is actually needed.
- Unpredictable latency from Vec reallocation or HashMap growth in the middle of a burst.

## How Does It Solve the Problem?

### Bump Allocators and Arenas

Instead of asking the global allocator on every update, you reserve a large block up front (or per batch) and "bump" a pointer. Allocation becomes pointer arithmetic. Freeing everything at once is just resetting the pointer.

Useful for per-symbol order books or temporary working buffers that have clear lifetime.

### bytes::Bytes and Zero-Copy

`bytes::Bytes` is a reference-counted, cheap-to-clone buffer that is perfect for network data. You can slice it (`split_off`, `slice`) without copying the underlying bytes until you need to.

This lets the feed handler hand a view of the raw frame to parsers and book updaters with almost no cost.

### Pre-allocation Is Your Friend

```rust
let mut levels: Vec<Level> = Vec::with_capacity(1024);
```

Do this once at startup for anything the hot path will push to.

### Data Structure Choices for Order Books and Queues

See the comparison table below and the detailed discussion in topic 07.

## Applicability

### How to Apply
- Receive frames as `bytes::Bytes`.
- Parse into Copy types or stack-allocated structures as much as possible.
- Use `SmallVec` / `ArrayVec` for bounded collections you touch on every update.
- Use arena allocators for anything with clear batch or symbol lifetime.

### Gotchas
- `Arc` bumps are not free and can cause cache-line ping-pong.
- Global allocator can still be called by libraries you didn't expect.
- Reusing buffers incorrectly leads to use-after-free or logic bugs that are hard to debug.

## Demo Code

`demo_code/arena-vs-vec/` — a tiny project comparing bump-style pre-allocated storage vs repeated Vec growth for synthetic market data updates.

See the folder for instructions.
