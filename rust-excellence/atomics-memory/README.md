# Atomics & Memory Ordering Mastery

## Why This Matters in HFT

This is the primary tool for safe, lock-free communication between the feed thread and everyone else.

If you get memory ordering wrong, you get:
- Stale prices being used for quoting
- Torn reads (bid from one update, ask from another)
- Extremely hard-to-reproduce bugs that only appear under high load or on different CPU architectures

## The Traffic Light Analogy

Think of memory ordering as traffic rules between threads (cores):

- **Relaxed**: No rules. Cars go whenever they want. Fastest, but crashes possible.
- **Release**: The writer puts up a "new information available" sign (and everything before the sign happened).
- **Acquire**: The reader waits until they see the sign, then trusts that everything the writer did before the sign is visible.
- **SeqCst**: Everyone agrees on a total global order of events. Very strong, quite slow.

For almost all HFT publication of market data, **Acquire/Release** is the sweet spot.

## Publishing Top-of-Book Correctly

```rust
use std::sync::atomic::{AtomicU64, Ordering};

struct TopOfBook {
    bid: AtomicU64,
    ask: AtomicU64,
    seq: AtomicU64,
}

impl TopOfBook {
    // Feed thread only
    #[inline]
    fn publish(&self, bid: u64, ask: u64) {
        let new_seq = self.seq.load(Ordering::Relaxed) + 1;
        self.bid.store(bid, Ordering::Release);
        self.ask.store(ask, Ordering::Release);
        self.seq.store(new_seq, Ordering::Release);
    }

    // Strategy threads
    #[inline]
    fn read(&self) -> (u64, u64, u64) {
        loop {
            let s1 = self.seq.load(Ordering::Acquire);
            let bid = self.bid.load(Ordering::Acquire);
            let ask = self.ask.load(Ordering::Acquire);
            let s2 = self.seq.load(Ordering::Acquire);

            if s1 == s2 {
                return (bid, ask, s2);
            }
            // Very rare — writer was in the middle. Retry.
            std::hint::spin_loop();
        }
    }
}
```

## Important Details

- We load the sequence first and after — this is the lightweight seqlock pattern.
- Using `Release` on the writer guarantees that if a reader sees the new sequence number, the bid and ask stores that happened before it are also visible.
- On x86 this is relatively cheap. On ARM/POWER the barriers are real.

## Common Mistakes

1. Using `Relaxed` for the sequence number → readers can see new seq but old (or torn) prices.
2. Using `SeqCst` "to be safe" everywhere → unnecessary cost.
3. Forgetting that `fetch_add` etc. also have ordering parameters.

## When Atomics Are Not Enough

- If the data you want to publish is larger than a few words, consider a proper seqlock or a lock-free ring buffer.
- If you need to transfer ownership of data (not just a snapshot), use a bounded SPSC channel (crossbeam, rtrb, etc.).

## Exercises

1. Change the above code to use only `Relaxed`. Write a test that spawns a writer and multiple readers and occasionally prints "torn read detected".
2. Benchmark the difference between Acquire/Release version and a `Mutex` protected version under high contention.
3. Research `crossbeam_utils::atomic::AtomicCell` and `seqlock` crates.
