//! Basic tick data storage using Arrow + Parquet + memmap concepts.
//! Educational: append events, write to Parquet, read back.

use arrow::array::{Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use parquet::arrow::arrow_writer::ArrowWriter;
use std::fs::File;
use std::sync::Arc;

fn main() {
    println!("=== Tick Data Storage Demo (Arrow + Parquet) ===\n");

    let schema = Arc::new(Schema::new(vec![
        Field::new("ts", DataType::Int64, false),
        Field::new("symbol", DataType::Utf8, false),
        Field::new("price", DataType::Int64, false),
    ]));

    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(Int64Array::from(vec![1000, 1001, 1002])),
            Arc::new(StringArray::from(vec!["BTC", "BTC", "ETH"])),
            Arc::new(Int64Array::from(vec![65000, 65010, 3005])),
        ],
    ).unwrap();

    // Write to Parquet
    let file = File::create("ticks.parquet").unwrap();
    let mut writer = ArrowWriter::try_new(file, schema, None).unwrap();
    writer.write(&batch).unwrap();
    writer.close().unwrap();

    println!("Wrote ticks.parquet");

    // In real code: use memmap for fast reads, or Polars lazy scan_parquet

    println!("Use Python polars.scan_parquet('ticks.parquet') for fast analysis.");
}
