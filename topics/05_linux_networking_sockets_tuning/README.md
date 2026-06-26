# 05 — Linux Networking, Sockets, and Tuning

## Why This Matters in Ultra-Low-Latency Crypto Trading

Almost all interaction with Binance (and other venues) goes over TCP (WebSocket or REST). Every extra syscall, every extra copy in the kernel, every delayed ACK, and every context switch adds latency and jitter to your tick-to-trade path.

## What Problem Does It Solve?

- Default Linux TCP settings are tuned for throughput and fairness, not for minimal latency on a single connection.
- The kernel is not your enemy, but you must understand where it costs you time.
- Reconnects and message loss are normal; your networking layer must recover without adding tail latency or duplicates.

## How Does It Solve the Problem?

### Essential Socket Options for Trading

- `TCP_NODELAY` — disable Nagle. Critical. Send small packets immediately.
- `SO_REUSEADDR` / `SO_REUSEPORT` — for fast restart.
- Increase receive buffer sizes (`SO_RCVBUF`) to absorb bursts.
- Use `recvmsg` / `sendmsg` with `MSG_DONTWAIT` when you want non-blocking behavior.
- Consider `SO_BUSY_POLL` on some kernels (advanced).

### Userspace vs Kernel

Move as much processing as possible into userspace after the data has arrived. Use `bytes` or `mmap` techniques where practical. The goal is to touch the data as few times as possible.

### Reconnect Strategy That Does Not Lose Messages

1. Detect gap via sequence numbers (depth stream `lastUpdateId`).
2. Do not throw away the current book on disconnect — attempt to resume.
3. On hard gap or long disconnect, fetch snapshot on a separate REST path (cold) and then apply buffered deltas.
4. Never replay the same fill twice — use client order ID + exchange order ID correlation.

## Demo Code

`demo_code/socket-options/` — a small example showing how to set common options on a TCP stream (educational; real Binance client would use a proper WebSocket library + these tunings underneath).

See README inside the demo folder.
