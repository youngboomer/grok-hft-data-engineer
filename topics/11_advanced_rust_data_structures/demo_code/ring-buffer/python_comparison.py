"""
Python side-by-side comparison for the ring buffer concept.

In real HFT data pipelines, the hot path is almost always in Rust/C++.
Python (or Polars) is used for:
- Analytical / post-trade processing
- Backtesting engines
- Data validation and transformation layers

This file shows a *very* naive ring buffer in Python so you can feel the difference
in mental model and performance characteristics.

Real production analytical pipelines would use:
- Polars (lazy) + Arrow
- DataFusion (Rust) exposed to Python
"""

from collections import deque
from typing import Optional

class SimpleRingBuffer:
    """Naive ring buffer using collections.deque for illustration only.
    Do NOT use this for anything performance sensitive.
    """
    def __init__(self, capacity: int):
        self.capacity = capacity
        self.buffer = deque(maxlen=capacity)

    def push(self, value):
        self.buffer.append(value)

    def pop(self) -> Optional[int]:
        if self.buffer:
            return self.buffer.popleft()
        return None

    def __len__(self):
        return len(self.buffer)


if __name__ == "__main__":
    rb = SimpleRingBuffer(8)
    for i in range(10):
        rb.push(i * 100)
        print(f"pushed {i*100}, len={len(rb)}")

    while (val := rb.pop()) is not None:
        print(f"popped {val}")

    print("\nKey difference: In Python you pay for reference counting, GIL, and object overhead.")
    print("In Rust we can have true zero-copy + cache-friendly layouts with no allocations on the hot path.")
