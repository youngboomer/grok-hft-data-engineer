# Core Questions: Concurrency & Memory

## Question 1: Explain Acquire/Release memory ordering using a real trading example.

**Why interviewers ask this**:
They want to know if you understand when data published by one thread becomes visible to another, and the performance vs safety trade-offs.

**Strong Answer Structure**:

### Why This Matters
In a market maker the feed thread is the only writer of the latest best bid/ask. The strategy thread(s) are readers. If the strategy sees a new sequence number but an old price, it can make a pricing decision on stale data → adverse selection or bad hedge.

### What Problem Does It Solve?
We need a way to publish a small piece of data so that:
- Readers never block the writer
- When a reader sees the "new" version, it is guaranteed to also see the data that belongs to that version
- We avoid expensive locks on the hot path

### How It Works (with Analogy)

**Analogy**: The feed thread is the librarian putting a new edition of the price newspaper on the stand. The sequence number is the edition number printed on the front page.

1. Feed thread updates prices.
2. Feed does `Release` store on the prices.
3. Feed does `Release` store on the sequence number (bumps the edition).
4. Strategy thread does `Acquire` load on the sequence.
5. If the edition number is newer, it does `Acquire` loads on the prices.
6. If it cares about consistency, it can double-check the sequence after reading prices (classic seqlock pattern).

**Code Sketch**:
```rust
// Writer (feed thread)
self.bid.store(new_bid, Ordering::Release);
self.ask.store(new_ask, Ordering::Release);
self.seq.store(new_seq, Ordering::Release);

// Reader (strategy thread)
let s1 = self.seq.load(Ordering::Acquire);
let bid = self.bid.load(Ordering::Acquire);
let ask = self.ask.load(Ordering::Acquire);
let s2 = self.seq.load(Ordering::Acquire);

if s1 == s2 && s2 > last_seen {
    // safe to use bid/ask
}
```

### Gotchas
- Using `Relaxed` everywhere → readers can see torn or reordered values.
- Forgetting to use `Acquire` on the reader side → compiler/CPU can reorder loads.
- Assuming that "it worked on my laptop" means the ordering is correct (x86 is very forgiving; ARM is not).

### When to Use
- Use Acquire/Release when publishing small immutable snapshots of data.
- Use SeqCst only when you need a total order across multiple atomic variables (rare and expensive).
- Consider a proper seqlock crate or your own carefully reviewed implementation for slightly larger structs.

---

## Question 2: How would you publish best bid/ask to multiple strategy threads without any of them blocking the feed thread?

**Strong Answer**:
- Single writer (feed) → multiple readers (strategies).
- Use atomics + sequence number (or seqlock).
- Readers must be non-blocking. If they are slow, they miss updates — that's acceptable. The feed must never wait.

**Alternatives and Trade-offs**:
- `RwLock`: Bad — slow writers when readers exist, and readers can still block writers briefly.
- `crossbeam::channel` (broadcast style): Possible, but adds queuing and potential allocation.
- Atomic snapshot (as above): Best starting point for top-of-book.

**Key Point to Emphasize**:
"The feed thread's only job is to publish the latest truth as fast and predictably as possible. Consumers are responsible for keeping up."

---

## Question 3: What is false sharing and how would you detect and fix it in a trading system?

**Definition** (recap): Two unrelated hot variables living on the same 64-byte cache line. Writes from one core invalidate the line for other cores.

**HFT Example**:
```rust
struct BadMetrics {
    updates_processed: AtomicU64,   // written by feed
    orders_sent: AtomicU64,         // written by OMS
    // both on same cache line → disaster
}
```

**Fix**:
```rust
#[repr(align(64))]
struct PaddedAtomic {
    value: AtomicU64,
    _pad: [u8; 56],
}
```

Or use `crossbeam_utils::CachePadded`.

**How to detect**:
- `perf c2c` (cache-to-cache) on Linux.
- Measure p99 before/after padding.
- Look at CPU performance counters for "cache coherency traffic".

**Interview Tip**: Mention that you would measure, not just theorize.

---

## Question 4: Compare Mutex, crossbeam channel, and atomic + seq for publishing market data.

Use the table from the data-structures section and expand on it.

**Strong candidates** will say:
- Mutex is simple but introduces unpredictable latency under contention.
- Channels are great for ownership transfer of larger messages.
- Atomics + sequence is king for tiny, frequently updated, read-mostly data like top of book.
