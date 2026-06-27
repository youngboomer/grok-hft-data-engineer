# Core Questions: Networking, Sockets & Exchange Protocols

## Question: How do you tune a TCP connection to Binance for minimal and predictable latency?

### Why This Matters
Almost all interaction with Binance happens over TCP (WebSocket for data, REST for orders). Default Linux socket settings are optimized for throughput and fairness, not sub-100µs tail latency.

### Strong Answer

**Key tunings (in rough order of impact):**

1. **TCP_NODELAY** — Disable Nagle's algorithm. Critical. Without it, small packets can be delayed up to 40ms.
   ```c
   int one = 1;
   setsockopt(fd, IPPROTO_TCP, TCP_NODELAY, &one, sizeof(one));
   ```

2. **Increase socket buffers** (`SO_RCVBUF`, `SO_SNDBUF`) — But not excessively. Too large increases memory pressure and can hurt latency in some cases. Start with 1-4MB for receive during bursts.

3. **SO_REUSEADDR / SO_REUSEPORT** — For fast restarts after crashes.

4. **Busy polling** (advanced, when you control the NIC):
   - `SO_BUSY_POLL`
   - `ethtool` rx/tx ring tuning
   - `isolcpus` + thread pinning

5. **Userspace processing** — Use `recvmsg` with `MSG_DONTWAIT` or `epoll` with edge-triggered mode. Minimize syscalls on the hot path.

**Reconnect strategy** (very important):
- Detect failures via sequence gaps or heartbeat timeouts.
- Do NOT block the hot feed thread during reconnect.
- Have a dedicated cold-path reconnect manager.
- On reconnect for user data stream: re-auth, re-subscribe, reconcile state via REST.

### Gotchas
- Many WebSocket libraries hide the underlying socket. You must configure options before the TLS handshake or through the library's socket accessor.
- Cloud environments (noisy neighbors) make some tunings less effective — you still need them.
- Over-buffering can mask real problems until a big burst.

### When to go further
- Kernel bypass (DPDK + userspace TCP) or FPGA only when you have proven that kernel networking is the dominant cost (very rare for most Binance market making).

---

## Question: Design a robust Binance WebSocket client that survives disconnects without losing messages or creating duplicate orders.

### Key Requirements
- Market data stream (public)
- User data stream (private, authenticated via listenKey)
- Order placement via signed REST (or WebSocket API if available)

### Strong Architecture
- Separate connections for market data and user data.
- Heartbeat handling (ping/pong at protocol level + application level).
- Sequence number validation on depth streams.
- Background task that refreshes listenKey every ~30-45 min.
- On disconnect:
  - Market data: gap detection + snapshot recovery (see market data questions).
  - User data: re-establish listenKey, then reconcile open orders + balances via REST.
- Idempotent order placement using client order IDs.

**Important Binance specifics**:
- Depth stream: `lastUpdateId` rules are strict.
- User data stream can drop messages; always cross-check with REST after reconnect.
- Rate limits apply to REST even during recovery.

### Common Failure Modes
- Treating the WebSocket as a reliable ordered channel.
- Creating new client order IDs on every reconnect attempt.
- Not buffering deltas while fetching snapshots.

---

## Question: How do you handle rate limits without introducing jitter into order placement?

**Good approaches**:
- Token bucket implemented with atomics or a very short critical section.
- Pre-compute "earliest allowed send time".
- Separate rate limit enforcement from the actual send path.
- For bursty hedging: queue and drain at allowed rate rather than sleeping in the hot path.

**Bad**:
- `std::thread::sleep` in the OMS hot path.
- Global lock around every order send.
- Ignoring rate limits until you get 429s.

Use a combination of client-side token bucket + respect of `retry-after` headers.
