# Chapter 0: Getting Started

## Apologies in advance

Yes, we're doing hello world. If you've written Python for years, this
will feel patronizing. Bear with it — the point isn't the program, it's
the *toolchain*. Python's toolchain (interpreter, pip, venv) and Rust's
toolchain (compiler, cargo, crates) solve similar problems in very
different ways, and understanding the Rust toolchain early will save you
hours of confusion later.

Think of this chapter as the equivalent of teaching someone who's always
driven automatic how the clutch works before taking them on a mountain
road. You already know how to drive. You just need to learn where the
new pedals are.

## Installing Rust

### The short version

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then restart your shell (or `source ~/.cargo/env`).

### What you just installed

| Tool | Python equivalent | What it does |
|------|------------------|-------------|
| `rustup` | `pyenv` | Manages Rust toolchain versions |
| `rustc` | `python` | The compiler (you rarely call it directly) |
| `cargo` | `pip` + `venv` + `setuptools` + `pytest` | Build, deps, test, run — everything |

That last one is the important one. In Python, you need separate tools
for package management (pip), virtual environments (venv), building
(setuptools/poetry/hatch), testing (pytest), and formatting (black).
In Rust, `cargo` does all of these. One tool.

### Verify it works

```bash
rustc --version    # should print rustc 1.XX.X
cargo --version    # should print cargo 1.XX.X
```

## Python vs Rust: Side by side

### Hello world

```python
# hello.py
print("Hello, world!")
```

```bash
$ python hello.py
Hello, world!
```

```rust
// src/main.rs
fn main() {
    println!("Hello, world!");
}
```

```bash
$ cargo run
Hello, world!
```

**What's different:**
- Rust needs a `main()` function — no top-level code
- `println!` has an exclamation mark because it's a *macro* (don't worry
  about why yet — it's so the compiler can check your format string)
- `cargo run` compiles *and* runs — no separate compile step needed

### Variables

```python
name = "World"
count = 42
pi = 3.14
active = True
```

```rust
let name = "World";      // &str (string slice — like a view)
let count = 42;           // i32 (32-bit integer — Rust picks a default)
let pi = 3.14;            // f64 (64-bit float)
let active = true;        // bool
```

**What's different:**
- `let` instead of bare assignment
- Semicolons at the end of statements
- Types are inferred (like Python) but *fixed* (unlike Python)
- Variables are **immutable by default** — use `let mut` to make mutable

```python
count = 0
count += 1  # fine in Python
```

```rust
let count = 0;
// count += 1;  // compile error! count is not mutable

let mut count = 0;
count += 1;       // this works
```

**Key insight:** Python variables are always mutable. Rust variables are
immutable by default. This seems restrictive, but it catches an entire
class of bugs — accidental mutation — at compile time.

### Functions

```python
def greet(name: str) -> str:
    return f"Hello, {name}!"
```

```rust
fn greet(name: &str) -> String {
    format!("Hello, {name}!")
}
```

**What's different:**
- `fn` instead of `def`
- Types are mandatory in function signatures (not optional hints)
- `&str` is a *borrowed* string (read-only view), `String` is an *owned*
  string (you'll learn the difference in Chapter 1)
- `format!` instead of f-strings — same idea, different syntax
- No `return` keyword needed — the last expression is the return value
  (though `return` works if you want it)

### Collections

```python
# List
numbers = [1, 2, 3]
numbers.append(4)

# Dict
scores = {"alice": 95, "bob": 87}
scores["carol"] = 91
```

```rust
// Vec (growable array — like Python list)
let mut numbers = vec![1, 2, 3];
numbers.push(4);

// HashMap (like Python dict)
use std::collections::HashMap;
let mut scores = HashMap::new();
scores.insert("alice", 95);
scores.insert("bob", 87);
scores.insert("carol", 91);
```

**What's different:**
- `vec![]` macro creates a Vec (Rust's growable array)
- `push` instead of `append`
- HashMap needs `use` to import it (like Python's `from collections import`)
- Both need `mut` because we're modifying them

### Control flow

```python
# If/else
if score >= 90:
    grade = "A"
elif score >= 80:
    grade = "B"
else:
    grade = "C"

# For loop
for item in items:
    print(item)

# While
while count > 0:
    count -= 1
```

```rust
// If/else — nearly identical, but no colons and requires braces
let grade = if score >= 90 {
    "A"
} else if score >= 80 {
    "B"
} else {
    "C"
};  // note: if/else is an expression — it returns a value!

// For loop
for item in &items {
    println!("{item}");
}

// While
while count > 0 {
    count -= 1;
}
```

**Key insight:** In Rust, `if/else` is an *expression* that returns a
value. You can assign its result to a variable. Python recently added
something similar with the walrus operator, but Rust's version is more
general.

### String formatting

```python
name = "World"
print(f"Hello, {name}!")           # f-string
print("Hello, {}!".format(name))   # .format()
print("Value: %d" % 42)            # % formatting
```

```rust
let name = "World";
println!("Hello, {name}!");                  // inline variable (Rust 1.58+)
println!("Hello, {}!", name);                // positional
println!("Hello, {n}!", n = name);           // named
println!("Pi is approximately {:.2}", 3.14159);  // format spec
```

Same idea, slightly different syntax. The `!` on `println!` means it's
a macro — the compiler checks your format string at compile time. If you
write `println!("{}")` without an argument, that's a *compile error*, not
a runtime crash.

## Project structure

### Python

```
my_project/
    my_project/
        __init__.py
        main.py
    tests/
        test_main.py
    requirements.txt
    setup.py / pyproject.toml
```

### Rust

```
my_project/
    src/
        main.rs    (for binaries)
        lib.rs     (for libraries)
    tests/         (integration tests)
    Cargo.toml     (like pyproject.toml — deps, metadata, everything)
```

`Cargo.toml` is the single source of truth. No `requirements.txt` vs
`setup.py` vs `pyproject.toml` confusion. One file.

## The compile-run cycle

Python: **write → run → see error → fix**

```bash
python my_script.py  # runs immediately, crashes at runtime on errors
```

Rust: **write → compile → see error → fix → run**

```bash
cargo run            # compiles first, then runs — catches errors before running
cargo check          # just checks for errors without building (faster)
cargo build          # builds without running
cargo test           # builds and runs tests
```

This feels slower at first. But the errors you catch at compile time are
errors you'll *never* see at runtime. No more "this worked in dev but
crashes in production because a variable was None."

## Cargo is your new best friend

| Command | What it does | Python equivalent |
|---------|-------------|-------------------|
| `cargo new my_project` | Create new project | `mkdir` + `pyproject.toml` |
| `cargo run` | Build and run | `python main.py` |
| `cargo test` | Run tests | `pytest` |
| `cargo check` | Type check (fast) | `mypy` |
| `cargo build` | Build binary | N/A (Python is interpreted) |
| `cargo build --release` | Optimized build | N/A |
| `cargo add serde` | Add dependency | `pip install serde` |
| `cargo clippy` | Lint | `ruff check` |
| `cargo fmt` | Format | `black` |
| `cargo doc --open` | Generate docs | `sphinx` / `pdoc` |

## Next Steps

If `cargo test -p ch00-getting-started` passes, your toolchain works.
Open `src/lib.rs` for annotated examples, then try the exercises in
`exercises/`.

When you're ready: Chapter 1 is where the real fun starts — ownership
is the concept that makes Rust *Rust*, and it's the one thing Python
developers find most surprising.
