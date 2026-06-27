# Object Pool Demo

Pre-allocated object pool to avoid allocator pressure in hot data paths.

Reference crates: `bumpalo`, custom slab implementations.

This is a very common pattern in high-performance Rust for trading systems.
