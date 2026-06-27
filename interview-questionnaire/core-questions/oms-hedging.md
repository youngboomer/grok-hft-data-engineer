# Core Questions: OMS, Strategy & Cross-Venue Hedging

## Question: Design an order state machine that makes duplicate orders and lost fills nearly impossible.

### Why Interviewers Ask
The OMS is where real money moves. Bugs here cause immediate P&L loss or compliance issues.

### Strong Answer

Use a Rust enum with exhaustive matching for states:

```rust
enum OrderState {
    New { client_id: u64 },
    Live { exchange_id: Option<String>, filled_qty: u64 },
    PartiallyFilled { exchange_id: String, filled_qty: u64 },
    Filled { exchange_id: String, avg_price: u64 },
    Canceled { exchange_id: Option<String> },
    Rejected { reason: String },
}
```

**Key principles**:
- Client order ID is the source of truth for deduplication.
- Never transition to a "sent" state until you have the response or confirmed via reconciliation.
- On reconnect: query open orders + recent fills, then reconcile local state machine.
- Fills should be applied idempotently (use exchange order ID + trade ID as key).

**Inventory update**:
- Fills should update inventory via a lock-free or short-critical-section path.
- Never block the user data stream consumer on strategy or quoting logic.

### Gotchas
- Partial fills arriving out of order.
- "Order accepted" vs "order on book" distinction.
- Using exchange timestamps vs local timestamps for latency measurement.

---

## Question: How should inventory from one venue flow into quoting and hedging logic on another venue?

### Core Challenges
- Fills are asynchronous.
- You need a consistent-enough view of net exposure for pricing decisions.
- Over-hedging or under-hedging both cost money.
- Different venues have different precision, symbols, fees, and latency.

### Recommended Design

1. **Central Inventory** (or per-asset) updated by all venue feed handlers.
   - Use atomics for simple net position + notional exposure.
   - Or a small seqlock struct if you need more fields (e.g. realized PnL).

2. **Quoting logic** on Binance reads the latest inventory cheaply on every decision cycle and applies skew.

3. **Hedging engine** reacts to inventory changes but lives on a slightly warmer path.
   - It can use a bounded queue or atomic flag to know "inventory just moved significantly".
   - It decides hedge size considering fees, current quotes on the hedge venue, and risk limits.

4. **Safety layers**:
   - Hard position limits that can cancel quotes.
   - "Hedge in flight" tracking to avoid sending multiple hedges for the same move.
   - Reconciliation loop on a cold path that periodically checks reality against both venues.

### Key Trade-off to Discuss
"I accept that for a few hundred microseconds the two venues may see slightly different inventory. I design risk limits and hedge frequency around that, rather than trying to make inventory perfectly consistent at all times (which would require blocking or heavy synchronization)."

---

## Question: Your strategy produces good quotes in quiet markets but gets picked off badly during volatility. What could be wrong with the feedback loop?

Possible issues to explore:
- Inventory updates are too slow or batched.
- The book used for quoting is stale relative to the inventory state.
- Skew logic is too weak or uses the wrong units (base vs quote).
- You are not pulling quotes fast enough when inventory moves against you.
- Adverse selection signal (e.g., one-sided fills) is not being acted on quickly.
- No "toxic flow" detection or temporary quote widening.

Strong candidates will talk about measuring "fill-to-inventory-update latency" and "inventory-to-next-quote latency" separately.
