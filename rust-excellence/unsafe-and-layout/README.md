# Unsafe Rust, Memory Layout, and SIMD for HFT (When and How)

## Philosophy

In well-written HFT Rust, `unsafe` should be rare, small, and heavily documented. The goal is to use it only to remove bounds checks or enable better layouts where the safe version has measurable cost, and then immediately wrap it in safe abstractions.

## When Unsafe Is Justified in Trading Systems

- Removing bounds checks in a hot loop where you have already proven the indices are valid (e.g., iterating a pre-sized vector with known length).
- Implementing a custom lock-free structure where the safe equivalent would allocate or use locks.
- Precise control over memory layout (`repr(C)`, `repr(align(64))`) to avoid false sharing or improve cache behavior.
- Very carefully, when interacting with kernel or NIC APIs that require raw pointers.

**Never** use `unsafe` for "performance" without measurement and a safe fallback.

## Safe Abstractions Around Unsafe

```rust
// Example: a very small unsafe access that is wrapped
pub struct FastVec<T> {
    data: *mut T,
    len: usize,
    cap: usize,
}

impl<T> FastVec<T> {
    #[inline]
    pub unsafe fn get_unchecked(&self, index: usize) -> &T {
        debug_assert!(index < self.len);
        &*self.data.add(index)
    }
}
```

Even better: use crates like `bytemuck`, `zerocopy`, or `slice::from_raw_parts` only after validation.

## Memory Layout Control

```rust
#[repr(C, align(64))]
struct CacheAlignedCounter {
    value: AtomicU64,
    _pad: [u8; 56],
}
```

Use this for hot atomics that are written from different threads.

`repr(transparent)` is useful when you want a newtype that has exactly the same layout as the inner type.

## SIMD in Rust for HFT

For most market making the hot path is not CPU-bound in a way that benefits from manual SIMD. However, certain things can benefit:

- Computing weighted mid prices or imbalance across many levels.
- Certain risk calculations or aggregations during bursts.
- Parsing binary protocols.

Use `std::simd` (nightly) or `packed_simd` / `core::arch` with care.

**Rule of thumb**: Profile first. Only add SIMD when you have a clear, measurable hotspot and the safe vectorized version (e.g. using iterators + autovectorization) is not sufficient.

## repr(C) vs repr(Rust)

- Use `repr(C)` when you need a stable layout for FFI, serialization, or very precise padding control.
- Default `repr(Rust)` allows the compiler to reorder fields for better packing. This is usually fine and even better for cache usage.

Always document why you chose a specific repr.
