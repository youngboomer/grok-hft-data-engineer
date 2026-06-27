# Zero-Copy Techniques

## Core Idea

Every time you copy data you pay in latency and cache pressure. The goal is to touch the original network bytes as few times as possible, and copy out only the fields you actually need to keep.

## bytes::Bytes

This is one of the most important crates for network HFT code.

```rust
use bytes::Bytes;

fn handle_frame(frame: Bytes) {
    // frame is cheap to clone (just refcount)
    let slice = frame.slice(10..30);   // also cheap, no copy of payload
    // Later when you need owned data:
    let owned: Vec<u8> = slice.to_vec(); // only now you pay
}
```

`Bytes` + slicing lets parsers return views without copying the entire message.

## Parsing Pattern

```rust
struct DepthDelta<'a> {
    last_update_id: u64,
    bids: &'a [(u64, u64)],
    // ...
}

fn parse_delta<'a>(raw: &'a [u8]) -> DepthDelta<'a> {
    // In real code you would do careful offset math or use a fast parser.
    // The key point is the returned struct borrows from raw.
}
```

You then immediately extract the few `u64` values you care about into your book. The original frame can be dropped.

## When Zero-Copy Is Not Worth It

- If you need the data to live longer than the buffer, you have to copy anyway.
- For very small messages, the copy cost may be smaller than the complexity of managing lifetimes.
- When the data needs to be mutated, ownership is often simpler.

## Golden Rule

"Copy late, copy little, copy explicitly."

If you see a `.clone()` or `.to_vec()` inside a hot loop without a very good comment, it is probably a bug.
