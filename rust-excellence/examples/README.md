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
(when placed in an examples/ directory of a Cargo project)

Experiment ideas:
- Change N and ITERS.
- Add actual work between iterations.
- Measure with perf.
