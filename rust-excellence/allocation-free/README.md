# Allocation-Free Code on the Hot Path

## Why This Matters

Every allocation can:
- Contend with other threads on the global allocator
- Cause memory to be zeroed or fetched into cache
- Occasionally trigger a page fault or allocator internal work

Under burst load these occasional costs become visible in p99 and p99.9.

## Techniques

### 1. Pre-allocation at Startup

```rust
let mut levels: Vec<Level> = Vec::with_capacity(1024);
let mut recent_trades: Vec<Trade> = Vec::with_capacity(256);
```

Do this once. Never let these grow on the hot path.

### 2. SmallVec / ArrayVec for Bounded Small Collections

```rust
use smallvec::SmallVec;
let mut top: SmallVec<[Level; 16]> = SmallVec::new();
```

Stays on the stack until it exceeds the inline capacity.

### 3. Bump Allocators / Arenas

When you have many temporary objects with a clear lifetime (e.g. per-symbol working buffers during a burst):

```rust
// Pseudocode
let arena = Bump::new();
let levels = arena.alloc_slice(...);
// Use levels...
// At end of batch: arena.reset();
```

No individual frees. Extremely predictable.

### 4. Object Pools (carefully)

For larger objects that you truly reuse, a pool can work. But pools themselves can become sources of complexity and false sharing.

**Rule**: Prefer pre-sized vectors and stack-friendly structures first.

### 5. Avoid These on Hot Path

- `String`, `format!`, `to_string()`
- `Vec::new()` without capacity (or growing)
- `HashMap` insertion that may rehash
- `Arc::new` or `Box::new` in the per-message path
- Error types that allocate on the happy path

## Diagnostic

Add this temporarily to a hot function:

```rust
#[cfg(debug_assertions)]
{
    let before = allocation_counter::count();
    // ... hot work ...
    let after = allocation_counter::count();
    assert_eq!(before, after, "allocation in hot path!");
}
```

Or use `dhat` / `heaptrack` / `perf` to find unexpected allocations in release builds.

## Interview Framing

When asked "How do you avoid allocations on the hot path?":

A strong answer mentions:
- Pre-allocation + capacity
- SmallVec / ArrayVec for common cases
- Extracting only `Copy` data you need
- Using `bytes::Bytes` + borrowing
- Measuring (not just hoping)
