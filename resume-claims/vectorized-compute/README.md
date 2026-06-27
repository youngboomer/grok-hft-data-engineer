# Vectorized Compute: Pandas, Polars, NumPy

## Why Vectorization Matters
Interpreted Python loops are slow. Vectorized libraries push work down to optimized C/Fortran/Rust kernels that operate on whole arrays at once. This gives orders-of-magnitude speedups and better cache utilization.

## NumPy Fundamentals (Foundation)

**Key Concepts**:
- `ndarray`: homogeneous, contiguous memory, fixed dtype.
- Broadcasting rules.
- Strides and memory layout (`C` vs `F` order).
- Views vs copies (`arr.view()`, slicing often creates views).
- Universal functions (ufuncs) — elementwise operations implemented in C.

**Performance Rules**:
- Avoid Python loops over arrays (`for i in range(len(arr))`).
- Use boolean indexing and advanced indexing carefully (can create copies).
- Prefer in-place operations when possible (`arr += 1` vs `arr = arr + 1`).
- Use `np.einsum` or specialized routines for complex linear algebra.

**Interview Deep Question**:
"Why is `arr[arr > 0]` sometimes much slower than you expect, and how does memory layout affect performance?"

## Pandas

**Strengths**: Extremely rich API, great for interactive analysis, integrates with everything.

**Weaknesses**: Often creates many intermediate copies, single-threaded by default (except some operations), memory hungry because of object dtype and indexes.

**Optimization Techniques**:
- Use `category` dtype for low-cardinality strings.
- Select only needed columns early.
- `query()` and `eval()` for some expressions (numexpr).
- `pd.to_numeric(..., downcast=...)`.
- Avoid `apply(axis=1)` — use vectorized or `apply` with raw=True + numba/cython.
- `groupby` with `observed=True` for categoricals.
- Chunked reading for large files (`read_csv(chunksize=...)`).

**Common Interview Claim**:
"I optimized several Pandas-heavy ETL steps by switching dtypes, eliminating `apply` loops, and using `merge` with proper indexes, reducing memory usage by 70% and runtime by 4x."

## Polars (Especially Lazy Evaluation)

**Why Polars is a Game Changer**:
- Written in Rust.
- Query optimizer (similar to Catalyst).
- True lazy evaluation — builds a plan, optimizes it, then executes.
- Multi-threaded by default.
- Arrow columnar format under the hood (excellent interoperability).
- Strict schema, no object dtype surprises.

**Lazy API Pattern**:
```python
import polars as pl

df = (
    pl.scan_parquet("data/*.parquet")
    .filter(pl.col("amount") > 100)
    .group_by("user_id")
    .agg(pl.col("amount").sum().alias("total"))
    .collect()   # execution happens here
)
```

**Key Advantages to Discuss**:
- Predicate and projection pushdown.
- Common subplan elimination.
- Streaming execution for larger-than-memory datasets.
- Much lower memory overhead than Pandas for many operations.
- Easy to switch between eager (`pl.DataFrame`) and lazy (`pl.LazyFrame`).

**When to Choose**:
- New pipelines or performance-sensitive work → Polars.
- Heavy interactive exploration or when you need Pandas ecosystem compatibility → Pandas (or use `polars` + `.to_pandas()` when needed).
- Very large data → Polars lazy + streaming or PySpark.

**Gotchas**:
- Some Pandas APIs don't have direct 1:1 equivalents yet (though gap is closing fast).
- Lazy plans can be surprising if you don't `explain()` them.
- String handling and regex can still be bottlenecks (use specialized kernels).

**Strong Interview Answer**:
"I rewrote a critical daily aggregation pipeline from Pandas to Polars LazyFrame. The optimizer pushed filters and projections, execution became multi-threaded, and end-to-end runtime dropped from 45 minutes to under 8 minutes with significantly lower peak memory."

## Interoperability & Modern Stack

- Arrow: The common language (zero-copy between Polars, Pandas 2.0+, PyArrow, Spark, Rust, etc.).
- Use `pl.from_arrow()` / `to_arrow()` or the interchange protocol.
- In performance pipelines: Ingest → Polars/PyArrow for cleaning → Arrow buffer to Rust (PyO3) for hot compute → back to Polars/Spark.

## NumPy / Pandas / Polars Comparison Table

| Aspect                  | NumPy                  | Pandas                      | Polars (Lazy)                  |
|-------------------------|------------------------|-----------------------------|--------------------------------|
| Primary Strength        | Numerical arrays       | Rich tabular analysis       | High-performance lazy queries  |
| Memory Model            | Contiguous, typed      | Block manager + object      | Arrow columnar                 |
| Parallelism             | Limited (ufuncs)       | Mostly single-threaded      | Excellent multi-threaded       |
| Optimizer               | None                   | Limited                     | Full query optimizer           |
| Large-than-RAM          | Manual                 | Limited                     | Streaming mode                 |
| Ecosystem Integration   | Excellent              | Best                        | Growing rapidly (Arrow)        |
| Best For                | Math / signals         | Exploration & small-medium  | Production pipelines           |

## Interview Prep Questions

1. "Your Pandas pipeline is using 80GB of RAM on a 20GB dataset. What do you do?"
2. "Walk through how Polars lazy evaluation would optimize this query: [give a query with filters, joins, groupby]."
3. "When would you still choose Pandas over Polars in 2025?"
4. "Explain the difference between a view and a copy in NumPy and how it affects performance and correctness."
5. "How do you achieve zero-copy data transfer between a Polars DataFrame and a high-performance Rust function exposed via PyO3?"

Prepare concrete before/after numbers from real work if possible.
