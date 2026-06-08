# Chapter 8: Escaping the GIL

## The Big Idea

Every Python developer eventually hits the same wall: you have a CPU-bound
loop, you reach for `threading`, and... nothing gets faster. The Global
Interpreter Lock (GIL) lets only one thread execute Python bytecode at a
time. Threads are great for *waiting* (network, disk) and useless for
*computing*. The sanctioned workaround, `multiprocessing`, gets real
parallelism by spawning whole interpreter processes and pickling every
argument and result across process boundaries — heavyweight, copy-everything
parallelism.

Rust has no GIL. Its threads are real OS threads that genuinely run on
multiple cores. So what stops them from corrupting shared data the way the
GIL "protects" Python objects? Two marker traits — **`Send`** (safe to move
to another thread) and **`Sync`** (safe to share between threads) — checked
at *compile time*. The type system does the GIL's job without serializing
execution: a data race in Rust is not a 3 a.m. heisenbug, it's a compile
error with a line number.

## Python Analogies

### `threading.Thread` = `std::thread` (but actually parallel)

```python
import threading

def worker(start, end, results, i):
    results[i] = count_primes(start, end)   # GIL: one thread at a time

threads = [threading.Thread(target=worker, args=(...)) for i in range(4)]
for t in threads: t.start()
for t in threads: t.join()
# Elapsed time: the same as one thread. Often worse.
```

```rust
std::thread::scope(|s| {
    let handles: Vec<_> = chunks
        .map(|(start, end)| s.spawn(move || count_primes(start, end)))
        .collect();
    handles.into_iter().map(|h| h.join().unwrap()).sum::<usize>()
});
// Elapsed time: roughly divided by your core count.
```

**Key insight:** the API shape is nearly identical — spawn, join, collect.
What changes is the semantics: Rust threads run simultaneously.
`thread::scope` adds a guarantee Python can't express: every spawned thread
provably finishes before the scope exits, so threads may safely *borrow*
local data instead of copying it.

### The GIL = `Send` + `Sync` (a lock at runtime vs a proof at compile time)

Python makes shared objects "safe" by never letting two threads run at
once. Rust makes them safe by refusing to compile programs that share the
wrong types:

```rust
use std::rc::Rc;
use std::thread;

let data = Rc::new(vec![1, 2, 3]);      // Rc: non-atomic refcount
thread::spawn(move || data.len());      // ERROR: `Rc<Vec<i32>>` cannot be
                                        // sent between threads safely
```

`Rc`'s reference count is plain (non-atomic) — exactly like CPython's
refcounting, which is precisely *why* CPython needs a GIL. Swap in `Arc`
(atomic reference count) and the same code compiles. The compiler error you
just read **is a data race that never got to exist**.

| Single-threaded type | Thread-safe sibling | Python's version |
|----------------------|--------------------|------------------|
| `Rc<T>` | `Arc<T>` | every object (GIL-guarded refcount) |
| `RefCell<T>` | `Mutex<T>` / `RwLock<T>` | every object (GIL-guarded mutation) |

### `threading.Lock` = `Mutex<T>` (but the lock *owns* the data)

```python
counter = 0
lock = threading.Lock()

def worker():
    global counter
    with lock:
        counter += 1
    # Nothing stops a careless coworker from writing
    # `counter += 1` without taking the lock. Hope is the strategy.
```

```rust
let counter = Arc::new(Mutex::new(0));

// The i32 lives INSIDE the Mutex. There is no way to reach it except
// .lock(), and the lock releases when the guard goes out of scope.
*counter.lock().unwrap() += 1;
```

**Key insight:** Python's lock and the data it protects are unrelated
objects, associated only by convention. `Mutex<T>` *contains* its data —
forgetting to take the lock is not a bug you can write.

### `queue.Queue` = `mpsc::channel`

```python
from queue import Queue
q = Queue()
# workers: q.put(result)
# main:    q.get()   — but how many gets? Sentinels? Poison pills?
```

```rust
let (tx, rx) = std::sync::mpsc::channel();
// workers: tx.send(result)
// main:    rx.iter().sum()  — ends automatically when all senders drop
```

**Key insight:** same multi-producer queue, but channel closure is tied to
ownership: when every `Sender` is dropped, the receiver's iterator simply
ends. No sentinel values, no counting expected messages. Channels also pair
beautifully with the ownership model: a value *moves* through the channel,
so sender and receiver can never touch it at the same time.

### `multiprocessing.Pool` = `rayon` (without the pickling tax)

```python
from multiprocessing import Pool

with Pool() as pool:                        # fork N interpreters
    flags = pool.map(is_prime, range(2, N)) # pickle every argument,
total = sum(flags)                          # pickle every result back
```

```rust
use rayon::prelude::*;

let total = (2..n).into_par_iter().filter(|&n| is_prime(n)).count();
```

**Key insight:** in Python, process-based parallelism is an architectural
decision — serialization costs, shared-memory gymnastics, "can this even be
pickled?" In Rust, it's a one-word diff: `iter()` → `par_iter()`. rayon's
work-stealing pool shares the address space (zero copies) and `Send`/`Sync`
still stand guard: if the closure captured something thread-unsafe, the
one-word change would not compile.

## The FFI Payoff: Releasing the GIL

This is where the course arc lands. A PyO3 extension (Chapters 6–7) holds
the GIL while it touches Python objects — but pure-Rust computation doesn't
need Python at all. `py.allow_threads(...)` releases the GIL for the
duration of a closure:

```rust
#[pyfunction]
fn count_primes_releasing_gil(py: Python<'_>, lo: u64, hi: u64) -> usize {
    py.allow_threads(|| {
        (lo..hi).into_par_iter().filter(|&n| is_prime(n)).count()
    })
}
```

Two things happen at once:

1. **The Rust side fans out** across every core with rayon — no GIL exists
   in Rust-land to stop it.
2. **The Python side keeps moving** — other Python threads run freely while
   Rust computes, because the GIL is released.

The closure can't touch Python objects — PyO3 enforces that with `Send`
bounds, the same machinery from this chapter. Python's biggest weakness
becomes a non-issue: keep writing Python, and let the hot loop escape to
threads the GIL cannot see. (This code is feature-gated behind
`--features python` in this chapter so the default build needs no Python.)

## A Note on Testing Parallel Code

The tests in this chapter assert one thing, many ways: **the parallel
result equals the sequential result.** They never assert on timing.
Wall-clock speedups depend on core counts, schedulers, and how noisy the
machine is — assertions like "4 threads should be 3x faster" are flaky by
construction, especially on shared CI runners. Benchmark performance in
prose and profiles; test *determinism*.

## Summary

| Python | Rust | What Changes |
|--------|------|-------------|
| The GIL | `Send` / `Sync` | Safety moves from a runtime lock to compile-time proof |
| `threading.Thread` | `std::thread::scope` / `spawn` | Threads actually run in parallel; scoped threads may borrow |
| Shared objects + hope | `Arc<T>` | Atomic refcount; sharing is explicit in the type |
| `threading.Lock` + convention | `Mutex<T>` | The lock owns the data; unlocked access won't compile |
| `queue.Queue` | `mpsc::channel` | Closes automatically when senders drop; values move |
| `multiprocessing.Pool` | `rayon` `par_iter` | No process spawn, no pickling — a one-word diff |
| C extension holding the GIL | `py.allow_threads` | Rust computes on all cores while Python keeps running |

## Next Steps

Open `src/lib.rs` to see these concepts in working code — including two
"data races that won't compile" captured as `compile_fail` doctests — then
try the exercises in `exercises/`.
