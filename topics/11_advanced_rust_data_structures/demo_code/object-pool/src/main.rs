//! Simple pre-allocated object pool (slab style) for HFT.
//!
//! Analogy: Reserve a big stack of paper at the beginning of the day instead of
//! asking the allocator for a new sheet every time (which can cause jitter and contention).
//!
//! In data pipelines we often need to allocate temporary `Order`, `Trade`, `Event` objects
//! at very high rates. A pool avoids the global allocator on the hot path.

use std::cell::UnsafeCell;

pub struct ObjectPool<T> {
    storage: Vec<UnsafeCell<T>>,
    free_list: Vec<usize>, // indices of free slots
}

impl<T: Default> ObjectPool<T> {
    pub fn new(capacity: usize) -> Self {
        let mut storage = Vec::with_capacity(capacity);
        let mut free_list = Vec::with_capacity(capacity);

        for i in 0..capacity {
            storage.push(UnsafeCell::new(T::default()));
            free_list.push(i);
        }

        Self { storage, free_list }
    }

    /// Take an object from the pool. Returns None if pool is exhausted.
    pub fn acquire(&mut self) -> Option<PoolGuard<'_, T>> {
        if let Some(idx) = self.free_list.pop() {
            Some(PoolGuard {
                pool: self,
                index: idx,
            })
        } else {
            None
        }
    }
}

pub struct PoolGuard<'a, T> {
    pool: &'a mut ObjectPool<T>,
    index: usize,
}

impl<'a, T> std::ops::Deref for PoolGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.pool.storage[self.index].get() }
    }
}

impl<'a, T> std::ops::DerefMut for PoolGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.pool.storage[self.index].get() }
    }
}

impl<'a, T> Drop for PoolGuard<'a, T> {
    fn drop(&mut self) {
        // Return the slot to the free list
        self.pool.free_list.push(self.index);
    }
}

// Example object we might pool in a data pipeline
#[derive(Default, Debug)]
struct PipelineEvent {
    seq: u64,
    price: u64,
    qty: u64,
}

fn main() {
    println!("=== Simple Object Pool Demo ===\n");

    let mut pool: ObjectPool<PipelineEvent> = ObjectPool::new(4);

    {
        let mut ev = pool.acquire().expect("pool exhausted");
        ev.seq = 42;
        ev.price = 650000000;
        ev.qty = 100;
        println!("Acquired and filled: {:?}", *ev);
    } // automatically returned to pool here

    // Reuse
    let ev2 = pool.acquire().expect("pool exhausted");
    println!("Reused slot (default or previous value): {:?}", *ev2);

    println!("\nIn real HFT data pipelines you would combine this with a ring buffer and seqlocks.");
}
