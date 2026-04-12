# Chapter 1: Ownership

## The Big Idea

In Python, you never think about who "owns" a piece of data. Python's garbage
collector handles it — objects live as long as someone references them, and
get cleaned up when nobody does. This works, but it means Python can never
*guarantee* at compile time that your data is safe.

Rust flips this: every piece of data has exactly **one owner** at any time.
When the owner goes out of scope, the data is dropped. No garbage collector
needed. And because the compiler enforces this, entire categories of bugs
(use-after-free, double-free, data races) simply cannot exist.

## Python Analogies

### Ownership = Variable Binding

```python
# Python: x and y both point to the same list
x = [1, 2, 3]
y = x           # y is another reference to the same list
x.append(4)     # mutates through x...
print(y)        # ...visible through y: [1, 2, 3, 4]
```

```rust
// Rust: ownership MOVES from x to y
let x = vec![1, 2, 3];
let y = x;       // x is MOVED to y — x is no longer valid
// println!("{:?}", x);  // compile error! x was moved
println!("{:?}", y);     // works: [1, 2, 3]
```

**Key insight:** In Python, assignment creates a new reference. In Rust,
assignment *transfers ownership*. The old variable is gone.

### Borrowing = Passing to a Function (but with rules)

```python
# Python: functions receive references — they can mutate your data
def add_item(lst):
    lst.append(42)

my_list = [1, 2, 3]
add_item(my_list)
print(my_list)  # [1, 2, 3, 42] — mutated!
```

```rust
// Rust: you choose whether to lend read-only or read-write access
fn print_items(items: &Vec<i32>) {      // immutable borrow: can read, can't modify
    println!("{:?}", items);
}

fn add_item(items: &mut Vec<i32>) {     // mutable borrow: can modify
    items.push(42);
}

let mut my_list = vec![1, 2, 3];
print_items(&my_list);                   // lend read-only
add_item(&mut my_list);                  // lend read-write
println!("{:?}", my_list);              // [1, 2, 3, 42]
```

**Key insight:** Python always gives mutable access. Rust makes you choose —
and guarantees that while someone has mutable access, nobody else can read.
This is how Rust prevents data races at compile time.

### RAII = Context Managers

```python
# Python: the `with` block ensures the file is closed
with open("data.txt") as f:
    data = f.read()
# f is closed here — guaranteed by __exit__
```

```rust
// Rust: the file is closed when `f` goes out of scope — no `with` needed
{
    let f = std::fs::File::open("data.txt").unwrap();
    // use f...
}   // f is dropped here — Drop trait runs, file is closed
```

**Key insight:** Python's `with` is opt-in — you can forget it. Rust's RAII
is automatic — cleanup *always* happens when the owner goes out of scope.
This pattern extends to locks, network connections, temp files — anything
with a destructor.

### Clone = `copy.deepcopy()`

```python
import copy
x = [1, 2, 3]
y = copy.deepcopy(x)   # y is an independent copy
x.append(4)
print(y)                # [1, 2, 3] — unaffected
```

```rust
let x = vec![1, 2, 3];
let y = x.clone();       // y is an independent copy
// x is still valid here — we cloned, not moved
println!("{:?}", x);     // [1, 2, 3]
println!("{:?}", y);     // [1, 2, 3]
```

### Copy = Immutable Value Types

```python
# Python integers are immutable — assignment looks like a copy
x = 42
y = x
x = 99
print(y)  # 42 — unaffected, because ints are immutable values
```

```rust
// Rust: types that implement Copy behave like Python immutables
let x: i32 = 42;
let y = x;     // x is COPIED, not moved (i32 implements Copy)
println!("{}", x);  // 42 — still valid
println!("{}", y);  // 42
```

**Key insight:** In Python, small immutable types (int, str, tuple) feel like
they're copied. In Rust, this is made explicit with the `Copy` trait. Types
that are cheap to copy (integers, booleans, chars) implement `Copy`. Types
that are expensive to copy (Vec, String) require explicit `.clone()`.

## Summary

| Python | Rust | What Changes |
|--------|------|-------------|
| Reference counting + GC | Ownership + drop | Memory safety without runtime cost |
| `y = x` (shared reference) | `let y = x;` (move) | Transfer, not sharing |
| Everything is mutable | `&` vs `&mut` | Compiler-enforced access control |
| `with` blocks | RAII (automatic) | Cleanup can't be forgotten |
| `copy.deepcopy()` | `.clone()` | Explicit, visible cost |
| Immutable builtins | `Copy` trait | Compiler knows what's cheap |

## Next Steps

Open `src/lib.rs` to see these concepts in working code, then try the
exercises in `exercises/`.
