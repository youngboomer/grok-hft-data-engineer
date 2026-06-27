# Gaps, Nice-to-Haves, and Non-Rust/Python Topics

This document captures areas that are important for HFT data management roles but are either:

- Not purely Rust/Python
- Advanced enough that we provide explanation without full implementation
- Nice-to-have for well-rounded knowledge

## Non-Rust/Python Topics (Brief Explanations)

### FPGA / Hardware Acceleration
Field-Programmable Gate Arrays allow you to implement logic directly in hardware.

**Why relevant**:
- Can achieve deterministic sub-microsecond processing for market data normalization and even simple matching.
- Used by top-tier HFT firms for the absolute edge (parsing feeds, basic risk checks).

**How it fits data pipelines**:
- Often sits between the NIC and the CPU pipeline.
- Market data comes in → FPGA normalizes and publishes to CPU via shared memory or DMA.

You do not need to write Verilog for most data engineering roles, but you should be able to discuss:
- When software (Rust) is "fast enough"
- The trade-off between flexibility (software) and determinism/speed (FPGA)
- Co-design: CPU does complex logic, FPGA does the hot path

### DPDK / Kernel Bypass
DPDK (Data Plane Development Kit) lets user-space applications talk directly to the NIC, bypassing the Linux kernel network stack.

**Relevance**:
- Removes syscall and kernel copy overhead for market data reception.
- Common in serious HFT feed handlers.

In Rust you can use `dpdk-sys` bindings or higher-level crates, but many teams wrap DPDK in C and expose a thin Rust interface.

### Aeron (Messaging)
Aeron is a high-performance transport (UDP + shared memory) designed for low-latency, high-throughput messaging.

**Why it appears in HFT**:
- Used for inter-process or inter-machine communication of market data and orders.
- Very low and predictable latency compared to most message brokers.

Rust has client libraries. You will often see it mentioned alongside Disruptor for the full stack (in-process + out-of-process).

### Simple Binary Encoding (SBE)
SBE is a code-generation based binary protocol designed for zero-copy decoding.

Very common in traditional finance (CME, Eurex, etc.). Even if your venue uses JSON/WebSocket (Binance), understanding SBE teaches you how efficient binary protocols work.

### kdb+/q
Still widely used for tick data storage and analytics in many (especially traditional) HFT and quant shops.

It is extremely fast for time-series but has a steep learning curve and is not free.

In a Rust/Python world you will more often see ClickHouse + Arrow or custom solutions.

## Nice-to-Have Topics for Data Roles

- Custom binary storage formats for tick data (append-only + index).
- Deterministic replay engines that can run faster than real-time.
- Integration of data pipelines with risk engines (position keeping at low latency).
- Exactly-once processing patterns that still respect latency budgets.
- eBPF for observability and some packet processing.
- NUMA and hardware topology awareness when scaling across sockets.
- Backtesting data engines that can feed strategies at production speeds.

## How We Handle These in This Branch

- Deep, production-oriented implementations in **Rust** (with Python where it makes sense for higher layers or comparison).
- Clear explanations (with analogies) for everything else.
- References to real crates and papers.
- Focus on concepts you can actually use in interviews and real data pipeline work.

This balance gives a newcomer enough scope to build real understanding while respecting the constraints of the domain.
