//! From-scratch SPSC Ring Buffer for HFT-style low-latency pipelines.
//!
//! This is an educational implementation meant to be simple to understand
//! while still being close to what you would use in production.
//!
//! Key concepts (keep these terms in your mental map):
//! - SPSC: Single Producer, Single Consumer (highest performance pattern)
//! - Sequence / Cursor: Atomic counters that track write and read positions
//! - Cache-line padding: Prevents "false sharing" where two cores fight over the same 64-byte cache line
//! - Wrap-around: When head or tail reaches capacity, it goes back to 0
//! - Zero-copy: Consumer can get a reference into the buffer instead of copying data out
//!
//! Why this matters in HFT data pipelines:
//! The producer (e.g. market data parser) writes updates into the ring.
//! Multiple consumers (strategy, risk, logging) can read without blocking the producer
//! or causing allocator jitter.
//!
//! Reference crates (study their source, use them in real code):
//! - rtrb (excellent realtime-safe SPSC)
//! - ringbuf
//! - crossbeam::queue::ArrayQueue (good for learning)
//!
//! We build an improved version here focused on clarity + padding + zero-copy reads.

use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// A single cache line is usually 64 bytes on modern CPUs.
/// We pad so that hot atomics live on their own cache lines.
const CACHE_LINE: usize = 64;

/// Pad an atomic so it doesn't share a cache line with other hot data.
#[repr(align(64))]
struct PaddedAtomic(AtomicUsize);

impl PaddedAtomic {
    fn new(v: usize) -> Self {
        Self(AtomicUsize::new(v))
    }
}

/// Simple SPSC ring buffer.
/// T must be Copy for simplicity in this educational version.
/// In a real system you would often use MaybeUninit<T> + manual drop.
pub struct SpscRingBuffer<T: Copy> {
    buffer: Box<[UnsafeCell<T>]>,
    capacity: usize,
    /// Producer writes here (mod capacity)
    head: PaddedAtomic,
    /// Consumer reads here
    tail: PaddedAtomic,
}

unsafe impl<T: Copy> Send for SpscRingBuffer<T> {}
unsafe impl<T: Copy> Sync for SpscRingBuffer<T> {}

impl<T: Copy> SpscRingBuffer<T> {
    /// Creates a new ring buffer with the given capacity.
    /// Capacity is rounded up to next power of two for fast modulo via masking.
    pub fn new(mut capacity: usize) -> Self {
        // Make capacity power of two for fast wrapping (head & (capacity-1))
        if !capacity.is_power_of_two() {
            capacity = capacity.next_power_of_two();
        }

        let mut buf = Vec::with_capacity(capacity);
        // Initialize with default value (we will overwrite before reading)
        for _ in 0..capacity {
            buf.push(UnsafeCell::new(unsafe { std::mem::zeroed() }));
        }

        Self {
            buffer: buf.into_boxed_slice(),
            capacity,
            head: PaddedAtomic::new(0),
            tail: PaddedAtomic::new(0),
        }
    }

    #[inline]
    fn mask(&self, val: usize) -> usize {
        val & (self.capacity - 1)
    }

    /// Try to push a value. Returns false if the buffer is full.
    /// This is the producer side (usually one thread).
    #[inline]
    pub fn push(&self, value: T) -> bool {
        let head = self.head.0.load(Ordering::Relaxed);
        let tail = self.tail.0.load(Ordering::Acquire);

        // Full when next head position would overwrite tail
        if self.mask(head + 1) == self.mask(tail) {
            return false;
        }

        // SAFETY: We checked there is space. Only the producer writes here.
        unsafe {
            let slot = &mut *self.buffer[self.mask(head)].get();
            *slot = value;
        }

        // Make the write visible to consumers
        self.head.0.store(head + 1, Ordering::Release);
        true
    }

    /// Try to pop a value. Returns None if empty.
    /// This is the consumer side.
    #[inline]
    pub fn pop(&self) -> Option<T> {
        let tail = self.tail.0.load(Ordering::Relaxed);
        let head = self.head.0.load(Ordering::Acquire);

        if self.mask(tail) == self.mask(head) {
            return None; // empty
        }

        let val = unsafe {
            let slot = &*self.buffer[self.mask(tail)].get();
            *slot
        };

        self.tail.0.store(tail + 1, Ordering::Release);
        Some(val)
    }

    /// Zero-copy read: get a reference to the current tail element without copying.
    /// The caller must copy the data out quickly if they need to keep it.
    /// Returns None if empty.
    #[inline]
    pub fn peek(&self) -> Option<&T> {
        let tail = self.tail.0.load(Ordering::Relaxed);
        let head = self.head.0.load(Ordering::Acquire);

        if self.mask(tail) == self.mask(head) {
            return None;
        }

        // SAFETY: The slot is valid and will not be overwritten until consumer advances tail.
        unsafe {
            Some(&*self.buffer[self.mask(tail)].get())
        }
    }

    /// Advance the consumer after a successful peek.
    #[inline]
    pub fn advance(&self) {
        let tail = self.tail.0.load(Ordering::Relaxed);
        self.tail.0.store(tail + 1, Ordering::Release);
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn len(&self) -> usize {
        let head = self.head.0.load(Ordering::Relaxed);
        let tail = self.tail.0.load(Ordering::Relaxed);
        if head >= tail {
            head - tail
        } else {
            self.capacity - (tail - head)
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_full(&self) -> bool {
        self.len() == self.capacity - 1
    }
}

fn main() {
    println!("=== Educational SPSC Ring Buffer Demo ===\n");

    let ring: SpscRingBuffer<u64> = SpscRingBuffer::new(8); // will become 8 (power of 2)

    // Producer side
    for i in 0..6u64 {
        let ok = ring.push(i * 100);
        println!("push {} -> success={}", i * 100, ok);
    }

    println!("\nBuffer state: len={}, capacity={}, empty={}, full={}",
             ring.len(), ring.capacity(), ring.is_empty(), ring.is_full());

    // Consumer side - using pop
    println!("\n--- Consuming with pop() ---");
    while let Some(val) = ring.pop() {
        println!("popped: {}", val);
    }

    // Refill and demonstrate zero-copy peek
    println!("\n--- Refilling and using zero-copy peek + advance ---");
    for i in 100..103u64 {
        ring.push(i);
    }

    while let Some(val) = ring.peek() {
        println!("peeked (zero-copy): {}", val);
        ring.advance();
    }

    println!("\nDemo complete. Try increasing burst size and measuring with --release.");
    println!("Experiment: add a second consumer thread and watch what breaks (or use a MPMC ring).");
}
