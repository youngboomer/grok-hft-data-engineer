//! Simplified Binance-style depth delta parser + sequence check.
//! Educational only. Real code would use proper JSON or a fast binary protocol.

#[derive(Debug)]
struct DepthDelta {
    last_update_id: u64,
    bids: Vec<(u64, u64)>,
    asks: Vec<(u64, u64)>,
}

fn parse_fake_depth(raw: &str) -> Option<DepthDelta> {
    // In real life: use serde or a manual zero-copy parser.
    // Here we just simulate the structure.
    if !raw.contains("lastUpdateId") { return None; }
    Some(DepthDelta {
        last_update_id: 123456789,
        bids: vec![(6500000000, 1200)],
        asks: vec![(6500010000, 800)],
    })
}

fn main() {
    let example = r#"{"lastUpdateId":123456789,"bids":[["65000.00","0.012"]],"asks":[["65001.00","0.008"]]}"#;
    if let Some(delta) = parse_fake_depth(example) {
        println!("Parsed delta lastUpdateId={}", delta.last_update_id);
        println!("First bid level: {:?}", delta.bids.first());
    }
    println!("Real version would also handle snapshot vs delta using lastUpdateId rules from Binance docs.");
}
