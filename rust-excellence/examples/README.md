# Runnable Examples

These are small, focused examples you can compile and experiment with.

## atomic_top_of_book.rs

Demonstrates correct Acquire/Release publication of top-of-book.

Run:
```bash
rustc --edition=2021 -O atomic_top_of_book.rs -o atomic_top && ./atomic_top
```
(or put it in a small Cargo project).

## smallvec_vs_vec.rs

Shows the cost difference of cloning a Vec vs using SmallVec in a tight loop.

```bash
cargo run --release --example smallvec_vs_vec
```

## preallocated_book.rs

Shows a pre-sized SmallVec used for maintaining top levels with incremental updates (no allocations in the update path after init).

## unsafe_indexing.rs

Compares safe indexing (with bounds checks) vs `get_unchecked` in a very hot summation loop. Demonstrates when (and why) you might reach for a small amount of unsafe after proving safety.

**Important**: Only do this after measurement shows the bounds checks matter, and wrap it safely.

Experiment ideas:
- Change N and ITERS.
- Add actual work between iterations.
- Measure with perf.
- Run under `cargo asm` to see the difference in generated code.
