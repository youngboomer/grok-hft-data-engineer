#!/usr/bin/env python3
"""
Latency distribution analyzer for the Rust harness.

pip install numpy pandas matplotlib

Usage:
    python analyze_latency.py latency.log
"""

import sys
import numpy as np
import pandas as pd

def main(path="latency.log"):
    data = pd.read_csv(path, header=None, names=["latency_ns"])
    ns = data["latency_ns"].values

    p50 = np.percentile(ns, 50)
    p95 = np.percentile(ns, 95)
    p99 = np.percentile(ns, 99)
    p999 = np.percentile(ns, 99.9)
    maxv = ns.max()

    print("=== Latency Analysis (nanoseconds) ===")
    print(f"count : {len(ns)}")
    print(f"p50   : {p50:,.0f} ns")
    print(f"p95   : {p95:,.0f} ns")
    print(f"p99   : {p99:,.0f} ns")
    print(f"p99.9 : {p999:,.0f} ns")
    print(f"max   : {maxv:,.0f} ns")
    print()

    # Very rough ASCII histogram
    bins = np.percentile(ns, [0, 50, 90, 95, 99, 99.9, 100])
    print("Rough distribution (log scale-ish):")
    for i in range(len(bins)-1):
        count = np.sum((ns >= bins[i]) & (ns < bins[i+1]))
        bar = "#" * int(np.log10(count + 1) * 4)
        print(f"  {bins[i]:8.0f} - {bins[i+1]:8.0f} ns : {count:6d} {bar}")

if __name__ == "__main__":
    path = sys.argv[1] if len(sys.argv) > 1 else "latency.log"
    main(path)
