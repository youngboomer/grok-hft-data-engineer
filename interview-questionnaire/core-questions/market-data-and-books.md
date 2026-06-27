# Core Questions: Market Data & Order Books

## Question: Walk me through exactly how you would maintain a correct Binance depth book from scratch.

### Strong Answer Structure

#### 1. Connection & Data Sources
- Connect to the depth WebSocket stream (`btcusdt@depth@100ms` or `@1000ms`).
- Separately maintain a user data stream for fills (different connection).
- Have a background path to fetch snapshots via REST.

#### 2. Snapshot + Delta Rules (This is where most people fail)
Binance rules (you must know these):
- Every depth message has `lastUpdateId`.
- Snapshot response has `lastUpdateId`.
- You must:
  1. Buffer deltas while fetching snapshot.
  2. Choose a snapshot whose `lastUpdateId` is >= the last buffered delta's `firstUpdateId - 1`.
  3. Drop any buffered deltas older than the snapshot.
  4. Apply remaining deltas in order.
  5. Ignore any message with `lastUpdateId <=` what you have already applied.

#### 3. Local Data Structure
For most market making you only need fast access to top levels + the ability to apply updates correctly.

Recommended starting point:
- A sorted `Vec<Level>` or `SmallVec` for the active top-N levels (very cache friendly).
- Possibly a `BTreeMap` or `HashMap` for full depth if you need deep levels for imbalance calculations.

#### 4. Publication to Strategy
After applying a delta:
- Update your internal book.
- Publish new best bid/ask (and optionally imbalance) using atomic + sequence or seqlock.
- Do **not** publish the entire book on every update unless the strategy actually needs it.

#### 5. Gap Detection & Recovery
- If you receive a message with `lastUpdateId` that jumps forward, you have a gap.
- Stop publishing new top-of-book or mark the book as "stale".
- Trigger snapshot fetch on cold path.
- Resume only after clean recovery.

#### Common Weak Answers
- "I'll just keep applying every delta I receive."
- "I'll store everything in a HashMap and sort when needed."
- Forgetting that you can receive messages out of order during high load.

---

## Question: How do you calculate meaningful imbalance without scanning the entire book on every update?

**Good Approaches**:
- Maintain running sums for top 5 / top 10 levels only.
- Update the running sums incrementally when a level changes.
- Publish the pre-computed imbalance value together with the top-of-book in your atomic snapshot.

**Why this matters**: Scanning 100+ levels on every market data update adds latency and cache pressure.

---

## Question: Your book looks correct in testing but during a real volatility spike you start quoting on the wrong side. What could be wrong?

Possible root causes to discuss (in order of likelihood):
1. Sequence handling bug (applying deltas out of order or twice).
2. Stale snapshot being used.
3. Race between feed thread updating the book and strategy reading a partially updated view.
4. Using floating point prices and getting comparison errors.
5. Not handling `U` (delete) vs `u` (update) correctly in some exchanges (Binance depth uses different logic).

**Strong candidates** will talk about adding sequence numbers to their internal book updates and logging "book state at decision time".
