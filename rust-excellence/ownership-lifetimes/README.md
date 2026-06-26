# Ownership, Borrowing & Lifetimes for HFT

## Why This Matters

Every microsecond of jitter and every correctness bug in a market maker eventually traces back to how data is owned and moved between components.

In a trading system you constantly ask:
- Who owns this parsed depth frame?
- Can I store a reference to it, or must I copy data out?
- If two threads need to see data, who is allowed to mutate it?
- What happens to my latency if I accidentally clone a Vec on every update?

Rust forces you to answer these questions at compile time.

## The Library Book Analogy (Revisited)

- **Ownership** (`let book = OrderBook::new(); let b2 = book;`): Only one person can have the book checked out. Moving it transfers responsibility.
- **Borrowing** (`&book` or `&mut book`): You can look at the book (many readers) or write in it (one writer), but the librarian (compiler) makes sure the rules are followed while you have it.
- **Lifetime**: "This reference is only valid as long as the book is still on the shelf where I left it."

These rules eliminate:
- Use-after-free (you can't use data after it moved or was dropped)
- Data races at compile time
- Many accidental expensive clones (you have to write `.clone()` explicitly)

## Hot Path Implications

### Rule 1: Move is free, clone is not

```rust
let frame = bytes::Bytes::from(...);           // owns the data
let frame2 = frame;                            // move — cheap pointer copy
let frame3 = frame2.clone();                   // clone — reference count bump + potential cost later
```

On the hot path you want **moves** or **borrows**. Clones should be suspicious.

### Rule 2: Lifetimes in Parsers

When parsing a Binance message, you often want to borrow from the original buffer as long as possible:

```rust
fn parse_depth<'a>(raw: &'a [u8]) -> DepthView<'a> {
    // returns a view that borrows from raw
}
```

You **cannot** store a `DepthView<'a>` in a long-lived struct unless the original `raw` buffer also lives that long.

This forces good design:
- Either extract the `Copy` data you need (prices, qtys, ids) immediately.
- Or keep the original `Bytes` alive alongside the view.

### Rule 3: Making Dangerous Things Impossible

Good HFT Rust code makes it hard to do the slow or wrong thing.

Example: Instead of returning an `Arc<Mutex<Book>>`, return a cheap snapshot type or force consumers to go through a non-blocking reader.

## Common Expensive Mistakes

1. Accidentally cloning a `Vec<Level>` or `String` inside the per-message loop.
2. Storing `&str` or `&[u8]` in structs that outlive the original buffer (leads to lifetime hell or forced clones).
3. Using `Arc<T>` "just to share" when the data is actually small and `Copy` would have been better.
4. Returning `Result<..., String>` from hot functions (the `String` allocation on the error path can still surprise you).

## Recommended Patterns for Trading Code

```rust
// Good: Copy types for prices
#[derive(Clone, Copy)]
struct Price(u64);

// Good: Small fixed data on stack
use smallvec::SmallVec;
type TopLevels = SmallVec<[PriceLevel; 20]>;

// Good: Take ownership of what you need, drop the rest quickly
fn process_frame(frame: bytes::Bytes) {
    let view = parse(&frame);
    book.apply(view);   // extracts only what it needs
    // frame can be dropped here
}
```

## Experiments

1. Take a hot function and add `#[inline(never)]` + use `cargo asm` or `cargo expand` to see what it actually does.
2. Write a small loop that clones a 50-element Vec on every iteration vs one that moves or borrows. Measure the difference in `--release`.
3. Try to store a borrowed depth view in a long-lived struct. Enjoy the compiler yelling at you — then fix it the right way.

Mastering ownership is the difference between "my Rust code is fast in benchmarks" and "my Rust code stays fast and correct when Binance is on fire."
