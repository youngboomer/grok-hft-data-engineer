# Case Study: Skew Handling in PySpark for Daily Order Book Features

## Context
A data team needed to compute daily aggregated features (volume at price profiles, order flow imbalance) from a month's worth of tick data for 300 symbols. The job was dominated by 8-10 hot symbols.

## Solution
- Identified skew via Spark UI and `explain()`.
- Applied salting only to hot symbols (detected via historical volume).
- Used `repartitionByRange` on price levels for the book reconstruction step.
- Combined with vectorized UDFs (Arrow) and Delta Lake for incremental updates.

## Results
- Runtime reduced ~6x.
- Straggler tasks eliminated.
- Feature correctness validated by comparing replay vs live on a hold-out day.

**Key Lesson**: Skew handling is not one-size-fits-all; combine statistical detection with domain knowledge of market data (a few symbols dominate volume).

This pairs extremely well with the systems-performance and vectorized compute material.
