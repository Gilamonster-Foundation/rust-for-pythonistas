//! # Chapter 8 Exercises: Escaping the GIL
//!
//! Each exercise shows a Python snippet and asks you to write the Rust
//! equivalent — except this time, the Rust version actually runs in
//! parallel. Replace the `todo!()` markers with working code.
//!
//! Every test asserts that your parallel result equals a sequential
//! reference result. Correctness first; speed comes along for free.
//!
//! Run tests: `cargo test -p ch08-exercises`

// These allows are intentional: exercise stubs have unused parameters
// and fields until the student fills in the todo!() markers.
#![allow(unused_variables, dead_code)]

use std::sync::{Arc, Mutex};

// ============================================================
// Exercise 1: Scoped Threads — Sum a Slice in Chunks
// ============================================================
//
// Python version (gets NO speedup — the GIL serializes the workers):
// ```python
// import threading
//
// def sum_threaded(numbers: list[int], n_chunks: int) -> int:
//     chunk_size = max(1, -(-len(numbers) // n_chunks))  # ceiling division
//     chunks = [numbers[i:i + chunk_size] for i in range(0, len(numbers), chunk_size)]
//     results = [0] * len(chunks)
//
//     def worker(i, chunk):
//         results[i] = sum(chunk)
//
//     threads = [threading.Thread(target=worker, args=(i, c))
//                for i, c in enumerate(chunks)]
//     for t in threads: t.start()
//     for t in threads: t.join()
//     return sum(results)
// ```
//
// Write the Rust version with std::thread::scope. Because scoped
// threads are guaranteed to finish before the scope returns, they can
// BORROW the slice — no copying, no Arc needed.
//
// Hints:
// - `numbers.chunks(chunk_size)` yields borrowed sub-slices
// - `std::thread::scope(|s| { ... s.spawn(move || ...) ... })`
// - collect the handles, then `.join().unwrap()` each and sum

pub fn sum_threaded(numbers: &[i64], n_chunks: usize) -> i64 {
    todo!("Split into chunks, spawn a scoped thread per chunk, sum the partial sums")
}

// ============================================================
// Exercise 2: Arc<Mutex<T>> — A Shared Counter
// ============================================================
//
// Python version:
// ```python
// import threading
//
// class SharedCounter:
//     def __init__(self):
//         self._value = 0
//         self._lock = threading.Lock()   # lock and data: separate objects!
//
//     def increment(self):
//         with self._lock:
//             self._value += 1
//
//     def value(self):
//         with self._lock:
//             return self._value
//
// def run_incrementers(counter, n_threads, increments_each):
//     def worker():
//         for _ in range(increments_each):
//             counter.increment()
//     threads = [threading.Thread(target=worker) for _ in range(n_threads)]
//     for t in threads: t.start()
//     for t in threads: t.join()
// ```
//
// In Rust the Mutex OWNS the value — there is no way to touch the u64
// without locking. Implement `increment`, `value`, and
// `run_incrementers`.
//
// Hints:
// - `self.inner.lock().unwrap()` gives a guard; `*guard += 1` mutates
// - `run_incrementers` needs `std::thread::spawn` + a clone of the
//   counter per thread (that's why SharedCounter derives Clone — cloning
//   an Arc just bumps the refcount; both clones point at the same Mutex)

#[derive(Clone)]
pub struct SharedCounter {
    inner: Arc<Mutex<u64>>,
}

impl SharedCounter {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(0)),
        }
    }

    /// Add 1 to the counter (lock, mutate, unlock).
    pub fn increment(&self) {
        todo!("Lock the mutex and add 1")
    }

    /// Read the current value.
    pub fn value(&self) -> u64 {
        todo!("Lock the mutex and return a copy of the value")
    }
}

impl Default for SharedCounter {
    fn default() -> Self {
        Self::new()
    }
}

/// Spawn `n_threads` threads that each call `increment()`
/// `increments_each` times, and wait for all of them to finish.
pub fn run_incrementers(counter: &SharedCounter, n_threads: usize, increments_each: u64) {
    todo!("Spawn threads with counter.clone(), join them all")
}

// ============================================================
// Exercise 3: Channels — Workers Report Partial Results
// ============================================================
//
// Python version:
// ```python
// import threading
// from queue import Queue
//
// def sum_of_squares_via_queue(values: list[int], n_workers: int) -> int:
//     q = Queue()
//     chunk_size = max(1, -(-len(values) // n_workers))
//     chunks = [values[i:i + chunk_size] for i in range(0, len(values), chunk_size)]
//
//     def worker(chunk):
//         q.put(sum(v * v for v in chunk))
//
//     threads = [threading.Thread(target=worker, args=(c,)) for c in chunks]
//     for t in threads: t.start()
//     for t in threads: t.join()
//     return sum(q.get() for _ in range(len(chunks)))   # must count gets!
// ```
//
// The Rust version uses mpsc::channel. Bonus elegance: when every
// Sender is dropped, the Receiver's iterator ends on its own — you
// don't need to know how many messages to expect.
//
// Hints:
// - `let (tx, rx) = std::sync::mpsc::channel();`
// - spawned threads need owned data: `chunk.to_vec()` each chunk
// - clone `tx` into each thread, send the partial sum
// - `drop(tx)` after spawning, then `rx.iter().sum()`

pub fn sum_of_squares_via_channel(values: &[i64], n_workers: usize) -> i64 {
    todo!("Chunk the values, send each chunk's sum of squares through a channel, sum the receiver")
}

// ============================================================
// Exercise 4: rayon — The One-Word Upgrade
// ============================================================
//
// Python version:
// ```python
// def count_vowels(words: list[str]) -> int:
//     return sum(sum(1 for c in w if c in "aeiou") for w in words)
//
// # The "parallel" version is a whole architectural decision:
// from multiprocessing import Pool
// def count_vowels_parallel(words):
//     with Pool() as pool:                       # fork interpreters
//         counts = pool.map(count_word, words)   # pickle every string
//     return sum(counts)
// ```
//
// In Rust, parallelizing is a one-word diff: `iter()` -> `par_iter()`.
// The sequential version is given; write the parallel one.
//
// Hints:
// - `use rayon::prelude::*;` (inside the function is fine)
// - same body as the sequential version, with `par_iter()`

pub fn count_vowels_sequential(words: &[String]) -> usize {
    words
        .iter()
        .map(|w| w.chars().filter(|c| "aeiou".contains(*c)).count())
        .sum()
}

pub fn count_vowels_parallel(words: &[String]) -> usize {
    todo!("Same as count_vowels_sequential, but with par_iter")
}

// ============================================================
// Exercise 5: rayon Map-Reduce — The Busiest Collatz Number
// ============================================================
//
// The Collatz sequence: repeatedly apply n -> n/2 (even) or n -> 3n+1
// (odd) until you reach 1. `collatz_steps` (provided) counts the steps.
//
// Python version:
// ```python
// def busiest_collatz(limit: int) -> tuple[int, int]:
//     """Find (n, steps) where n in 1..limit takes the MOST steps.
//     Ties go to the smaller n."""
//     best_n, best_steps = 1, 0
//     for n in range(1, limit):
//         steps = collatz_steps(n)
//         if steps > best_steps:
//             best_n, best_steps = n, steps
//     return best_n, best_steps
// ```
//
// Write the rayon version. The interesting part is making ties
// DETERMINISTIC: a parallel max may inspect candidates in any order, so
// "first one wins" is not reproducible. Encode the tie-break in the key
// instead: maximize `(steps, Reverse(n))` so equal step counts prefer
// the smaller n — then every run, sequential or parallel, agrees.
//
// Hints:
// - `(1..limit).into_par_iter()`
// - `.map(|n| (n, collatz_steps(n)))`
// - `.max_by_key(|&(n, steps)| (steps, std::cmp::Reverse(n)))`
// - the range is non-empty for limit >= 2; `.unwrap()` is fine here

/// Number of Collatz steps to reach 1. Provided — the exercise is the
/// parallel reduction, not the arithmetic.
pub fn collatz_steps(mut n: u64) -> u64 {
    let mut steps = 0;
    while n > 1 {
        n = if n.is_multiple_of(2) {
            n / 2
        } else {
            3 * n + 1
        };
        steps += 1;
    }
    steps
}

/// Return `(n, steps)` for the n in `1..limit` with the most Collatz
/// steps. Ties prefer the smaller n.
pub fn busiest_collatz(limit: u64) -> (u64, u64) {
    todo!("Parallel map n -> (n, steps), then max_by_key with a deterministic tie-break")
}

// ============================================================
// Tests — do not modify below this line
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Exercise 1
    #[test]
    fn ex1_sum_matches_sequential() {
        let numbers: Vec<i64> = (1..=1000).collect();
        let expected: i64 = numbers.iter().sum();
        assert_eq!(sum_threaded(&numbers, 4), expected);
    }

    #[test]
    fn ex1_more_chunks_than_numbers() {
        let numbers = vec![1, 2, 3];
        assert_eq!(sum_threaded(&numbers, 16), 6);
    }

    #[test]
    fn ex1_empty_slice() {
        assert_eq!(sum_threaded(&[], 4), 0);
    }

    // Exercise 2
    #[test]
    fn ex2_increment_and_read() {
        let counter = SharedCounter::new();
        counter.increment();
        counter.increment();
        counter.increment();
        assert_eq!(counter.value(), 3);
    }

    #[test]
    fn ex2_clones_share_state() {
        let counter = SharedCounter::new();
        let alias = counter.clone();
        counter.increment();
        alias.increment();
        assert_eq!(counter.value(), 2); // both clones hit the same Mutex
    }

    #[test]
    fn ex2_concurrent_increments_are_not_lost() {
        let counter = SharedCounter::new();
        run_incrementers(&counter, 8, 1000);
        // Without the Mutex, increments would be lost to races.
        // With it: exactly 8 * 1000, every time.
        assert_eq!(counter.value(), 8000);
    }

    // Exercise 3
    #[test]
    fn ex3_matches_sequential() {
        let values: Vec<i64> = (1..=100).collect();
        let expected: i64 = values.iter().map(|v| v * v).sum();
        assert_eq!(sum_of_squares_via_channel(&values, 4), expected);
    }

    #[test]
    fn ex3_single_worker() {
        let values = vec![3, 4];
        assert_eq!(sum_of_squares_via_channel(&values, 1), 25);
    }

    #[test]
    fn ex3_empty_input() {
        assert_eq!(sum_of_squares_via_channel(&[], 4), 0);
    }

    // Exercise 4
    #[test]
    fn ex4_parallel_matches_sequential() {
        let words: Vec<String> = (0..300)
            .map(|i| format!("parallelism-example-{i}"))
            .collect();
        assert_eq!(
            count_vowels_parallel(&words),
            count_vowels_sequential(&words)
        );
    }

    #[test]
    fn ex4_known_value() {
        let words = vec!["aeiou".to_string(), "xyz".to_string(), "rust".to_string()];
        assert_eq!(count_vowels_parallel(&words), 6); // 5 + 0 + 1
    }

    #[test]
    fn ex4_empty_input() {
        assert_eq!(count_vowels_parallel(&[]), 0);
    }

    // Exercise 5
    #[test]
    fn ex5_collatz_steps_provided() {
        assert_eq!(collatz_steps(1), 0);
        assert_eq!(collatz_steps(2), 1);
        assert_eq!(collatz_steps(6), 8); // 6→3→10→5→16→8→4→2→1
        assert_eq!(collatz_steps(27), 111); // the famous slow starter
    }

    #[test]
    fn ex5_busiest_below_30() {
        // 27 takes 111 steps — the unique maximum below 30.
        assert_eq!(busiest_collatz(30), (27, 111));
    }

    #[test]
    fn ex5_trivial_range() {
        assert_eq!(busiest_collatz(2), (1, 0));
    }

    #[test]
    fn ex5_parallel_matches_sequential_reference() {
        use std::cmp::Reverse;
        // Sequential reference with the same deterministic tie-break.
        let expected = (1..5000u64)
            .map(|n| (n, collatz_steps(n)))
            .max_by_key(|&(n, steps)| (steps, Reverse(n)))
            .unwrap();
        assert_eq!(busiest_collatz(5000), expected);
    }
}
