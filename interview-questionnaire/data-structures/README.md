# Data Structures for HFT Interviews

Data structure choice is one of the highest-leverage decisions in an ultra-low-latency trading system.

Interviewers love this area because:
- Poor choices create both latency **and** correctness problems.
- Good choices demonstrate you understand cache behavior, contention, allocation, and hot-path constraints.

Every structure below includes:
- Why it matters
- Trade-off table
- When to use / when to avoid
- HFT-specific gotchas
- Simple code sketch where useful

---

## 1. Common Data Structures in Trading Systems

### Vec<T>

**Role**: The workhorse for anything with a small-to-medium known upper bound (top levels, recent trades, pending intents).

**HFT Considerations**:
- `Vec::with_capacity()` at startup is mandatory.
- `.clone()` on hot path of a large Vec is one of the fastest ways to create tail latency.
- Reallocation during a burst is catastrophic for predictability.

**Strong Pattern**:
```rust
let mut top_levels: Vec<Level> = Vec::with_capacity(64);
```

### HashMap vs BTreeMap

| Structure   | Lookup | Ordered? | Allocation | Cache Friendliness | HFT Use Case                     | Recommendation |
|-------------|--------|----------|------------|--------------------|----------------------------------|----------------|
| HashMap     | O(1) avg | No     | Yes        | Poor               | Symbol -> Book lookup            | Good for cold lookups |
| BTreeMap    | O(log n) | Yes    | Yes        | Moderate           | Price levels when you need order | Rarely on hot path |
| Sorted Vec  | O(log n) binary search | Yes | No (if pre-sized) | Excellent | Top-N price levels               | Preferred for small hot sets |

**Rule of thumb**: If N < 128 and you need ordering, use a sorted `Vec` or `SmallVec` + binary search. HashMap for symbol routing only.

### SmallVec / ArrayVec

These are critical in HFT.

- `SmallVec<[T; N]>`: Stores up to N elements on the stack. Falls back to heap only when exceeded.
- `ArrayVec<[T; N]>`: Fixed capacity, never allocates.

**Perfect for**:
- Top 16–64 price levels
- Recent trades buffer
- Small order ID lists

**Interview gold**: "I use `SmallVec` because 99% of the time we only care about the top 20 levels. The common case stays on the stack and in cache."

---

## 2. HFT-Specific Data Structures

### Atomic Snapshot for Top-of-Book

**Structure**:
```rust
struct AtomicTopOfBook {
    bid: AtomicU64,
    ask: AtomicU64,
    seq: AtomicU64,
}
```

**Publication pattern** (Release on write, Acquire on read + sequence check).

**Why interviewers ask**: This is the simplest and most common lock-free publication mechanism. You must be able to explain memory ordering here.

### Seqlock for Small Structs

Used when you want to publish a slightly larger struct (e.g. best bid + ask + last trade price + imbalance) without locks.

Readers spin briefly if writer is in the middle of an update.

### Lock-Free SPSC / MPMC Ring Buffers

**Use cases**:
- Strategy → OMS intent queue
- Feed → multiple consumers (sometimes)
- Fill notifications

Popular crates: `crossbeam-channel`, `rtrb`, or hand-rolled with atomics.

**Key properties to discuss**:
- Bounded vs unbounded (bounded is almost always safer)
- What happens on full queue (drop, block, overwrite?)
- Cache line behavior of head/tail indices

### Order Book Representations

Common options:

| Representation              | Pros                              | Cons                                      | Hot Path Suitability | Typical Use |
|-----------------------------|-----------------------------------|-------------------------------------------|----------------------|-------------|
| `BTreeMap<Price, Qty>`      | Easy ordered iteration            | Allocations + pointer chasing             | Poor                 | Rarely hot  |
| Sorted `Vec<PriceLevel>`    | Excellent cache locality          | Insert/delete in middle is O(n)           | Good for small depth | Top-N books |
| Array of buckets (price bins) | O(1) access if price range known | Wastes memory, tricky with floating prices | Excellent for fixed tick | Some futures |
| HashMap + separate sorted list | Fast lookup + order               | Complex, two structures to keep in sync   | Medium               | Full depth  |
| Hierarchical (price buckets + lists) | Good balance for very deep books | Complexity                                | Medium               | Very deep books |

**For most Binance spot market making**: A sorted `SmallVec` or `Vec` for the active top levels + a map (or second structure) for full depth is common.

### PriceLevel Struct Design

```rust
#[repr(C)]
#[derive(Clone, Copy)]
struct PriceLevel {
    price: u64,   // scaled integer
    qty: u64,
}
```

**Important considerations**:
- Keep it `Copy` when possible.
- Consider padding to avoid false sharing if you have arrays of these written by different threads.
- Use scaled integers (not f64) to avoid floating point comparison nightmares.

---

## 3. Comparison Table — Hot Path Suitability

| Structure                  | Allocation on Hot Path | Contention | Cache Behavior | Predictability | HFT Recommendation |
|----------------------------|------------------------|------------|----------------|----------------|--------------------|
| `Mutex<T>`                 | No (after init)        | High       | Bad            | Poor           | Almost never on hot path |
| `AtomicU64`                | No                     | None       | Excellent      | Excellent      | Best for single values |
| `SmallVec<[T; 16]>`        | Only on overflow       | None       | Excellent      | Very good      | Top levels, small buffers |
| `crossbeam::channel` (bounded) | On send (if grows)  | Low        | Good           | Good           | Intent / fill queues |
| `BTreeMap`                 | Yes                    | High       | Poor           | Poor           | Cold path only |
| Custom arena / bump        | No (after reserve)     | None       | Excellent      | Excellent      | Batch temporary objects |
| `Arc<Mutex<Book>>`         | Yes                    | Very high  | Terrible       | Terrible       | Never for market data |

---

## Interview Tips for Data Structures Questions

When asked "What data structure would you use for X?":

**Strong answer structure**:
1. Clarify the access patterns (read-mostly? write rate? ordering needed? size distribution?).
2. State the constraints (allocation forbidden on hot path, multiple readers, single writer, etc.).
3. Propose 1-2 options with trade-offs.
4. Mention a specific gotcha (false sharing, reallocation during burst, sequence handling).
5. Mention how you would measure that the choice was good.

**Weak answer**: "I'd use a HashMap because it's fast."

Always tie the choice back to **predictable latency** and **correctness under burst load**.
