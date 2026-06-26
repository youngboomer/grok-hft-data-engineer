# Lock-Free Publication Demo

Feed thread publishes best bid/ask using atomics + sequence.

Two consumer threads read without ever blocking the publisher.

This is the fundamental pattern behind safe, low-jitter market data distribution.

Run:

```bash
cargo run --release
```
