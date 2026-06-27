//! Demonstrates bounds-check elimination with unsafe in a very hot loop.
//! Compare performance of safe indexing vs unchecked in a tight summation.

use std::time::Instant;

fn sum_safe(data: &[u64], indices: &[usize]) -> u64 {
    let mut sum = 0;
    for &i in indices {
        sum += data[i]; // bounds check on every access
    }
    sum
}

fn sum_unchecked(data: &[u64], indices: &[usize]) -> u64 {
    let mut sum = 0;
    for &i in indices {
        // SAFETY: caller guarantees indices are valid
        sum += unsafe { *data.get_unchecked(i) };
    }
    sum
}

fn main() {
    let data: Vec<u64> = (0..10_000).map(|i| i as u64).collect();
    let indices: Vec<usize> = (0..100_000).map(|i| i % 10_000).collect();

    let start = Instant::now();
    let s1 = sum_safe(&data, &indices);
    println!("Safe: {} in {:?}", s1, start.elapsed());

    let start = Instant::now();
    let s2 = sum_unchecked(&data, &indices);
    println!("Unchecked: {} in {:?}", s2, start.elapsed());

    println!("(Difference is usually small but measurable in the hottest loops. Always measure.)");
}
