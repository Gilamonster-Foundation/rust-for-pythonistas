//! # Chapter 8: Escaping the GIL
//!
//! Python threads never run Python bytecode in parallel — the Global
//! Interpreter Lock (GIL) serializes them. Rust threads are real OS
//! threads with no interpreter lock, and the `Send`/`Sync` traits make
//! data races a *compile error* instead of a runtime hazard.
//!
//! Run the tests: `cargo test -p ch08-escaping-the-gil`

use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

use rayon::prelude::*;

// ---------------------------------------------------------------------------
// 1. A CPU-bound workload — the kind Python threads can't speed up
// ---------------------------------------------------------------------------

/// Trial-division primality test. Deliberately CPU-bound: pure
/// arithmetic, no I/O, nothing for a thread to "wait" on.
///
/// Python equivalent:
/// ```python
/// def is_prime(n: int) -> bool:
///     if n < 2:
///         return False
///     if n < 4:
///         return True
///     if n % 2 == 0:
///         return False
///     d = 3
///     while d * d <= n:
///         if n % d == 0:
///             return False
///         d += 2
///     return True
/// ```
///
/// In Python, running this on threads buys you NOTHING: the GIL lets
/// only one thread execute bytecode at a time, so 4 threads of
/// `is_prime` take as long as 1 thread (often longer, due to lock
/// contention). The standard escape hatch is `multiprocessing`, which
/// gets real parallelism by paying a heavy toll: spawning whole
/// interpreter processes and pickling every argument and result across
/// process boundaries.
pub fn is_prime(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    if n < 4 {
        return true;
    }
    if n.is_multiple_of(2) {
        return false;
    }
    let mut d = 3;
    while d * d <= n {
        if n.is_multiple_of(d) {
            return false;
        }
        d += 2;
    }
    true
}

/// Count primes in `lo..hi`, sequentially. This is our baseline — every
/// parallel version below must produce exactly this answer.
///
/// Python equivalent:
/// ```python
/// def count_primes(lo: int, hi: int) -> int:
///     return sum(1 for n in range(lo, hi) if is_prime(n))
/// ```
pub fn count_primes(lo: u64, hi: u64) -> usize {
    (lo..hi).filter(|&n| is_prime(n)).count()
}

// ---------------------------------------------------------------------------
// 2. Real threads — std::thread::scope and borrowing across threads
// ---------------------------------------------------------------------------

/// Count primes by splitting the range across real OS threads.
///
/// Python equivalent (which does NOT get a speedup, thanks to the GIL):
/// ```python
/// import threading
///
/// def count_primes_threaded(lo, hi, n_threads):
///     results = [0] * n_threads
///     chunk = -(-(hi - lo) // n_threads)  # ceiling division
///
///     def worker(i):
///         start = min(lo + i * chunk, hi)
///         end = min(start + chunk, hi)
///         results[i] = count_primes(start, end)  # all serialized by the GIL
///
///     threads = [threading.Thread(target=worker, args=(i,)) for i in range(n_threads)]
///     for t in threads: t.start()
///     for t in threads: t.join()
///     return sum(results)
/// ```
///
/// The Rust version is structurally identical — but the threads
/// genuinely run in parallel on separate cores. `thread::scope`
/// guarantees every spawned thread finishes before the scope returns,
/// which is why the threads are allowed to *borrow* from the enclosing
/// function (here they only capture small `u64` copies, but they could
/// borrow a `&[u64]` slice too — see the exercises).
pub fn count_primes_threaded(lo: u64, hi: u64, n_threads: usize) -> usize {
    let n_threads = n_threads.max(1) as u64;
    let chunk = (hi.saturating_sub(lo)).div_ceil(n_threads);

    thread::scope(|s| {
        let mut handles = Vec::new();
        for i in 0..n_threads {
            let start = (lo + i * chunk).min(hi);
            let end = (start + chunk).min(hi);
            // Each thread owns its own start/end and returns its own
            // count. No shared mutable state — nothing to race on.
            handles.push(s.spawn(move || count_primes(start, end)));
        }
        handles
            .into_iter()
            .map(|h| h.join().expect("worker thread panicked"))
            .sum()
    })
}

// ---------------------------------------------------------------------------
// 3. Send and Sync — the type system does the GIL's job, without the lock
// ---------------------------------------------------------------------------
//
// Why is sharing data between Python threads "safe"? Because the GIL
// serializes every bytecode instruction — safety by *never actually
// running in parallel*.
//
// Rust takes the opposite deal. Threads really run in parallel, and two
// marker traits decide, at compile time, what may cross a thread
// boundary:
//
//   - `Send`: this type can be MOVED to another thread.
//   - `Sync`: this type can be SHARED (`&T`) between threads.
//
// You almost never implement these yourself — the compiler derives them
// structurally. `Rc` (non-atomic refcount, like CPython's refcounting
// without the GIL guarding it) is NOT Send. `RefCell` (runtime borrow
// checking, single-threaded) is NOT Sync. Their thread-safe siblings
// are `Arc` (atomic refcount) and `Mutex` (a real lock).

/// Sum the lengths of all words using threads that SHARE the data.
///
/// `thread::spawn` (unlike `thread::scope`) may outlive the caller, so
/// it cannot borrow — everything it captures must be `'static` and
/// `Send`. To share one `Vec` among such threads we use `Arc`, the
/// atomically reference-counted pointer.
///
/// Here is the version that does NOT compile. `Rc`'s reference count is
/// not atomic, so two threads cloning it at once would corrupt the
/// count — the compiler rejects it because `Rc<Vec<String>>` is not
/// `Send`:
///
/// ```compile_fail
/// use std::rc::Rc;
/// use std::thread;
///
/// let words = Rc::new(vec!["hello".to_string(), "world".to_string()]);
/// let shared = Rc::clone(&words);
/// let handle = thread::spawn(move || shared.len()); // ERROR: `Rc<...>` cannot
///                                                   // be sent between threads
/// handle.join().unwrap();
/// ```
///
/// In Python the equivalent bug is *silent*: every object is happily
/// shared between threads, and the GIL papers over the danger by never
/// letting two threads touch it simultaneously. Rust gives you the
/// parallelism and moves the safety check to the compiler.
///
/// Python equivalent (shared read-only list — safe only because of the GIL):
/// ```python
/// def total_chars(words: list[str], n_threads: int) -> int:
///     results = [0] * n_threads
///
///     def worker(i):
///         results[i] = sum(len(w) for w in words[i::n_threads])
///
///     threads = [threading.Thread(target=worker, args=(i,)) for i in range(n_threads)]
///     for t in threads: t.start()
///     for t in threads: t.join()
///     return sum(results)
/// ```
pub fn total_chars_across_threads(words: Vec<String>, n_threads: usize) -> usize {
    let n_threads = n_threads.max(1);
    let words = Arc::new(words); // atomic refcount: Arc<Vec<String>> IS Send + Sync

    let mut handles = Vec::new();
    for i in 0..n_threads {
        let words = Arc::clone(&words); // bump the refcount, move the clone in
        handles.push(thread::spawn(move || {
            // Stride partitioning: thread i takes words[i], words[i+n], ...
            words
                .iter()
                .skip(i)
                .step_by(n_threads)
                .map(|w| w.len())
                .sum::<usize>()
        }));
    }
    handles
        .into_iter()
        .map(|h| h.join().expect("worker thread panicked"))
        .sum()
}

// ---------------------------------------------------------------------------
// 4. Shared MUTABLE state — Arc<Mutex<T>> and channels
// ---------------------------------------------------------------------------

/// Count primes with worker threads adding into one shared counter.
///
/// Python equivalent:
/// ```python
/// import threading
///
/// counter = 0
/// lock = threading.Lock()
///
/// def worker(start, end):
///     global counter
///     local = count_primes(start, end)   # compute OUTSIDE the lock
///     with lock:
///         counter += local               # mutate INSIDE the lock
/// ```
///
/// The crucial difference: Python's `threading.Lock` and the data it
/// protects are *unrelated objects* — nothing stops a worker from
/// updating `counter` without taking the lock. Rust's `Mutex<T>` OWNS
/// the data. The only way to reach the `usize` inside is through
/// `.lock()`, which returns a guard; the lock releases when the guard
/// goes out of scope (like `with lock:`, but impossible to forget).
///
/// And here is the race the compiler refuses to compile. `RefCell` is
/// Python-style "checked at runtime, single thread assumed" interior
/// mutability — it is not `Sync`, so you cannot share it across
/// threads even inside an `Arc`:
///
/// ```compile_fail
/// use std::cell::RefCell;
/// use std::sync::Arc;
/// use std::thread;
///
/// let counter = Arc::new(RefCell::new(0));
/// let shared = Arc::clone(&counter);
/// let handle = thread::spawn(move || {
///     *shared.borrow_mut() += 1; // ERROR: `RefCell<i32>` cannot be
///                                // shared between threads safely
/// });
/// handle.join().unwrap();
/// ```
///
/// Swap `RefCell` for `Mutex` and it compiles — that one-word change is
/// the whole single-threaded/multi-threaded migration, enforced by types.
pub fn count_primes_shared_counter(lo: u64, hi: u64, n_threads: usize) -> usize {
    let n_threads = n_threads.max(1) as u64;
    let chunk = (hi.saturating_sub(lo)).div_ceil(n_threads);
    let counter = Arc::new(Mutex::new(0usize));

    let mut handles = Vec::new();
    for i in 0..n_threads {
        let counter = Arc::clone(&counter);
        let start = (lo + i * chunk).min(hi);
        let end = (start + chunk).min(hi);
        handles.push(thread::spawn(move || {
            let local = count_primes(start, end); // compute outside the lock
            *counter.lock().expect("mutex poisoned") += local; // brief critical section
        }));
    }
    for handle in handles {
        handle.join().expect("worker thread panicked");
    }

    let total = *counter.lock().expect("mutex poisoned");
    total
}

/// Count primes using a channel instead of a shared counter.
///
/// `mpsc::channel` is Rust's `queue.Queue`: multi-producer,
/// single-consumer, and the receiving end doubles as an iterator.
///
/// Python equivalent:
/// ```python
/// from queue import Queue
///
/// def count_primes_with_queue(lo, hi, n_threads):
///     q = Queue()
///     chunk = -(-(hi - lo) // n_threads)
///
///     def worker(start, end):
///         q.put(count_primes(start, end))
///
///     threads = []
///     for i in range(n_threads):
///         start = min(lo + i * chunk, hi)
///         end = min(start + chunk, hi)
///         t = threading.Thread(target=worker, args=(start, end))
///         t.start()
///         threads.append(t)
///     for t in threads: t.join()
///     return sum(q.get() for _ in range(n_threads))
/// ```
///
/// Channels often beat shared state: each value has exactly ONE owner
/// at a time, ownership transfers through the channel, and there is no
/// lock to hold wrong. "Do not communicate by sharing memory; share
/// memory by communicating."
pub fn count_primes_channel(lo: u64, hi: u64, n_threads: usize) -> usize {
    let n_threads = n_threads.max(1) as u64;
    let chunk = (hi.saturating_sub(lo)).div_ceil(n_threads);
    let (tx, rx) = mpsc::channel();

    for i in 0..n_threads {
        let tx = tx.clone(); // multi-producer: every worker gets a sender
        let start = (lo + i * chunk).min(hi);
        let end = (start + chunk).min(hi);
        thread::spawn(move || {
            tx.send(count_primes(start, end)).expect("receiver dropped");
        });
    }
    drop(tx); // drop the original sender so the iterator below terminates

    // rx.iter() yields until every sender is dropped — no sentinel
    // values, no "poison pill" pattern, no counting how many to expect.
    rx.iter().sum()
}

// ---------------------------------------------------------------------------
// 5. rayon — the "free" upgrade from sequential to parallel
// ---------------------------------------------------------------------------

/// Count primes in parallel with rayon. Compare to `count_primes`:
/// the ONLY change is `iter` → `par_iter` (`into_par_iter` for ranges).
///
/// Python equivalent:
/// ```python
/// from multiprocessing import Pool
///
/// def count_primes_parallel(lo, hi):
///     with Pool() as pool:
///         flags = pool.map(is_prime, range(lo, hi))  # pickles every int
///     return sum(flags)                              # across process pipes
/// ```
///
/// `multiprocessing.Pool` is Python's honest workaround for the GIL —
/// but it forks whole interpreter processes, pickles every argument,
/// and pickles every result back. rayon's work-stealing thread pool
/// shares the address space: no copies, no serialization, and chunking
/// is automatic. The sequential and parallel versions are guaranteed to
/// produce identical results, so the tests assert exactly that.
pub fn count_primes_rayon(lo: u64, hi: u64) -> usize {
    (lo..hi).into_par_iter().filter(|&n| is_prime(n)).count()
}

/// A map/reduce over a slice: total characters across all words.
///
/// Python equivalent:
/// ```python
/// def total_chars(words: list[str]) -> int:
///     return sum(len(w) for w in words)
///
/// # The multiprocessing version must pickle every string to the workers:
/// with Pool() as pool:
///     total = sum(pool.map(len, words))
/// ```
///
/// With rayon, `.par_iter()` borrows the slice in place — zero copies.
/// `Send`/`Sync` still guard the door: if the item type were not safe
/// to share across threads, `.par_iter()` would not compile.
pub fn total_chars_rayon(words: &[String]) -> usize {
    words.par_iter().map(|w| w.len()).sum()
}

/// The same computation, sequentially — kept for the tests to compare
/// against, and to show how little changes: `iter` vs `par_iter`.
pub fn total_chars_sequential(words: &[String]) -> usize {
    words.iter().map(|w| w.len()).sum()
}

// ---------------------------------------------------------------------------
// 6. The FFI payoff — releasing the GIL from a PyO3 extension
// ---------------------------------------------------------------------------
//
// Here is where the whole course arc pays off. A PyO3 extension module
// holds the GIL while it talks to Python objects — but pure-Rust work
// doesn't need Python at all. `py.allow_threads(...)` RELEASES the GIL
// for the duration of a closure, letting:
//
//   1. other Python threads keep running while Rust crunches numbers, and
//   2. the Rust code inside fan out across every core with rayon.
//
// The closure cannot touch any Python object (PyO3 enforces this with —
// you guessed it — `Send` bounds), so it is provably safe to let the
// interpreter carry on without us.
//
// From Python, the result looks like magic:
//
// ```python
// import threading
// from my_extension import count_primes_releasing_gil
//
// # This call saturates every core via rayon, and while it runs, OTHER
// # Python threads keep executing — the GIL is released for the duration.
// t = threading.Thread(target=count_primes_releasing_gil, args=(2, 10_000_000))
// t.start()
// do_other_python_work()  # not blocked!
// t.join()
// ```
//
// Python's weakness (threads can't compute in parallel) becomes a
// non-issue: you keep writing Python, and the hot loop escapes to Rust
// threads that the GIL cannot see.
//
// This module only compiles with `--features python` so that the
// default build and CI need no Python installation.

#[cfg(feature = "python")]
pub mod python {
    use pyo3::prelude::*;

    /// Count primes in `lo..hi` on all cores, with the GIL released.
    ///
    /// The signature takes `py: Python<'_>` — the token that PROVES we
    /// hold the GIL. `allow_threads` consumes that proof, releases the
    /// lock, runs the closure on plain Rust data, and re-acquires the
    /// GIL before returning to Python.
    #[pyfunction]
    pub fn count_primes_releasing_gil(py: Python<'_>, lo: u64, hi: u64) -> usize {
        py.allow_threads(|| super::count_primes_rayon(lo, hi))
    }

    /// The module definition: `import ch08_escaping_the_gil` from Python
    /// (build the wheel with `maturin`, as covered in Chapter 6).
    #[pymodule]
    fn ch08_escaping_the_gil(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add_function(wrap_pyfunction!(count_primes_releasing_gil, m)?)?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
//
// Note what these tests do NOT assert: timing. Parallel code is tested
// for CORRECTNESS (parallel result == sequential result) because
// wall-clock speedups depend on core counts and noisy schedulers —
// especially on shared CI runners. Discuss performance in prose;
// assert determinism in tests.

#[cfg(test)]
mod tests {
    use super::*;

    // There are 168 primes below 1000 — a classic checkable constant.
    const PRIMES_BELOW_1000: usize = 168;

    // The CPU-bound baseline

    #[test]
    fn is_prime_basics() {
        assert!(!is_prime(0));
        assert!(!is_prime(1));
        assert!(is_prime(2));
        assert!(is_prime(3));
        assert!(!is_prime(4));
        assert!(is_prime(97));
        assert!(!is_prime(1_000_000));
        assert!(is_prime(1_000_003));
    }

    #[test]
    fn sequential_baseline() {
        assert_eq!(count_primes(0, 1000), PRIMES_BELOW_1000);
        assert_eq!(count_primes(0, 10), 4); // 2, 3, 5, 7
        assert_eq!(count_primes(10, 10), 0); // empty range
        assert_eq!(count_primes(20, 10), 0); // inverted range
    }

    // Scoped threads

    #[test]
    fn threaded_matches_sequential() {
        assert_eq!(count_primes_threaded(0, 1000, 4), PRIMES_BELOW_1000);
    }

    #[test]
    fn threaded_with_more_threads_than_work() {
        // 16 threads over 10 numbers: most chunks are empty, answer unchanged.
        assert_eq!(count_primes_threaded(0, 10, 16), 4);
    }

    #[test]
    fn threaded_handles_zero_threads() {
        // Degenerate input is clamped to one thread, not a panic.
        assert_eq!(count_primes_threaded(0, 100, 0), count_primes(0, 100));
    }

    // Arc — shared read-only data across spawned threads

    #[test]
    fn arc_shared_data_matches_sequential() {
        let words: Vec<String> = ["alpha", "beta", "gamma", "delta", "epsilon"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let expected: usize = words.iter().map(|w| w.len()).sum();
        assert_eq!(total_chars_across_threads(words, 3), expected);
    }

    #[test]
    fn arc_shared_data_empty_input() {
        assert_eq!(total_chars_across_threads(Vec::new(), 4), 0);
    }

    // Arc<Mutex<T>> — shared mutable state

    #[test]
    fn shared_counter_matches_sequential() {
        assert_eq!(count_primes_shared_counter(0, 1000, 4), PRIMES_BELOW_1000);
    }

    #[test]
    fn shared_counter_single_thread() {
        assert_eq!(count_primes_shared_counter(0, 1000, 1), PRIMES_BELOW_1000);
    }

    // Channels

    #[test]
    fn channel_matches_sequential() {
        assert_eq!(count_primes_channel(0, 1000, 4), PRIMES_BELOW_1000);
    }

    #[test]
    fn channel_empty_range() {
        assert_eq!(count_primes_channel(100, 100, 4), 0);
    }

    // rayon

    #[test]
    fn rayon_matches_sequential() {
        // The load-bearing assertion of the chapter: parallel and
        // sequential produce IDENTICAL results.
        assert_eq!(count_primes_rayon(0, 10_000), count_primes(0, 10_000));
    }

    #[test]
    fn rayon_known_value() {
        assert_eq!(count_primes_rayon(0, 1000), PRIMES_BELOW_1000);
    }

    #[test]
    fn rayon_map_reduce_matches_sequential() {
        let words: Vec<String> = (0..500).map(|i| format!("word-{i}")).collect();
        assert_eq!(total_chars_rayon(&words), total_chars_sequential(&words));
    }

    #[test]
    fn all_strategies_agree() {
        // Five implementations, one answer. This is the property that
        // makes parallelism safe to adopt: same inputs, same outputs,
        // regardless of scheduling.
        let expected = count_primes(0, 2000);
        assert_eq!(count_primes_threaded(0, 2000, 4), expected);
        assert_eq!(count_primes_shared_counter(0, 2000, 4), expected);
        assert_eq!(count_primes_channel(0, 2000, 4), expected);
        assert_eq!(count_primes_rayon(0, 2000), expected);
    }
}
