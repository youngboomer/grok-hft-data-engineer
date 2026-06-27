//! Arrow + Polars analytical pipeline demo.
//!
//! Simulates hot path producing Arrow data, then analytical layer
//! (Polars lazy) doing aggregations, window functions, etc.

use arrow::array::{Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use polars::prelude::*;
use std::sync::Arc;

fn main() {
    println!("=== Arrow / Polars Analytical Pipeline Demo ===\n");

    // Simulate hot path producing batches (e.g. from ring buffer)
    let schema = Arc::new(Schema::new(vec![
        Field::new("seq", DataType::Int64, false),
        Field::new("symbol", DataType::Utf8, false),
        Field::new("price", DataType::Int64, false),
        Field::new("qty", DataType::Int64, false),
    ]));

    let batch = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(Int64Array::from(vec![1, 2, 3, 4])),
            Arc::new(StringArray::from(vec!["BTC", "BTC", "ETH", "ETH"])),
            Arc::new(Int64Array::from(vec![65000, 65010, 3000, 3010])),
            Arc::new(Int64Array::from(vec![10, 5, 20, 15])),
        ],
    ).unwrap();

    // Convert to Polars for analytics (zero-copy where possible)
    let df = DataFrame::try_from(batch).unwrap();
    println!("Raw batch as Polars:\n{}", df);

    let lazy = df.lazy()
        .group_by([col("symbol")])
        .agg([
            col("price").mean().alias("avg_price"),
            col("qty").sum().alias("total_qty"),
        ]);

    let result = lazy.collect().unwrap();
    println!("\nAggregated:\n{}", result);

    println!("\nThis is how real pipelines hand off from Rust hot path to analytical layer.");
}
