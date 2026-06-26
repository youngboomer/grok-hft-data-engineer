# Performance & Tooling Demos

## latency-harness (Rust)

```bash
cd latency-harness
cargo run --release
```

Produces `latency.log` (one latency in nanoseconds per line).

## analyze_latency.py (Python)

```bash
pip install numpy pandas
python analyze_latency.py ../latency-harness/latency.log
# or from the harness dir after running it:
python ../analyze_latency.py latency.log
```

This is the kind of lightweight but extremely valuable tooling you should have for every system you build.
