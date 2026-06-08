# Chapter 6: FFI & PyO3

## The Big Idea

Python has always known how to call native code — that's why NumPy is fast.
But the traditional routes are painful. `ctypes` makes *you* declare every
argument type and return type by hand; get one wrong and you don't get an
exception, you get garbage bits or a segfault. Hand-written C extensions are
worse: `PyArg_ParseTuple` format strings, manual reference counting, and a
module-initialization dance that has to be exactly right.

**PyO3 writes the glue for you.** You write ordinary Rust functions and
structs, add `#[pyfunction]`, `#[pyclass]`, and `#[pymodule]` attributes,
and the macros generate everything a C extension needs — argument parsing,
type conversion, reference counting, error propagation. Then **maturin**
compiles it into a wheel that anyone can `pip install` — *without a Rust
toolchain on their machine*.

That last part is the point of this whole course: Rust's payoff for a
Python team isn't "rewrite the service," it's "ship the hot path as a
module your colleagues import like any other package."

## Python Analogies

### The old way: `ctypes` — you write the contract by hand

```python
import ctypes

lib = ctypes.CDLL("./libtextstats.so")   # you find the library
lib.ffi_add.argtypes = [ctypes.c_int64, ctypes.c_int64]  # you declare types
lib.ffi_add.restype = ctypes.c_int64     # you declare the return type
lib.ffi_add(2, 3)                        # 5 — if you got all of that right

# Get restype wrong? No error — just wrong numbers.
# Pass a bad pointer/length pair? Best case garbage, worst case segfault.
```

The C ABI only speaks pointers and integers. Strings become
pointer-plus-length pairs, and the *caller* is responsible for both being
right. `src/lib.rs` section 1 shows this boundary from the Rust side —
note the `unsafe` keyword and the `# Safety` contract that the compiler
cannot check for you.

### The new way: PyO3 — the macro writes the contract

```rust
use pyo3::prelude::*;

#[pyfunction]
fn add(a: i64, b: i64) -> i64 {
    a + b
}

#[pymodule]
fn textstats(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(add, m)?)?;
    Ok(())
}
```

```python
import textstats
textstats.add(2, 3)      # 5
textstats.add("x", "y")  # TypeError — at the call site, not a segfault later
```

**Key insight:** the type information ctypes made you re-declare in Python
already exists in the Rust signature. PyO3 reads it from there. Wrong types
become a `TypeError` raised immediately — the conversion layer checks every
argument before your Rust code ever runs.

### Type conversions: the FromPyObject / IntoPyObject mental model

Arguments convert *from* Python, return values convert *to* Python:

| Python side | Rust side | Notes |
|-------------|-----------|-------|
| `str` | `&str` / `String` | `&str` borrows — zero-copy read |
| `int` | `i64`, `usize`, ... | range-checked; `OverflowError` if too big |
| `float` | `f64` | |
| `bool` | `bool` | |
| `list[T]` | `Vec<T>` | converts element by element |
| `dict[K, V]` | `HashMap<K, V>` | |
| `tuple[A, B]` | `(A, B)` | |
| `None` / value | `Option<T>` | `None` ↔ `Option::None` |
| raised exception | `Result<T, E>` | see error mapping below |

This is why the chapter's core functions have the signatures they do:
`tally(words: &[String]) -> HashMap<String, usize>` is *already* the shape
of `def tally(words: list[str]) -> dict[str, int]`. Design your Rust API
with these shapes and the binding layer stays one line per function.

### Error mapping: `Result` becomes a raised exception

C functions signal failure by returning `-1` or `NULL` and setting `errno`
— and ctypes won't check unless you ask. Forget the check and the error
silently becomes a "valid" value.

PyO3 uses Rust's `Result` instead, and the failure path is impossible to
ignore on *either* side of the boundary:

```rust
impl From<NumberError> for PyErr {
    fn from(err: NumberError) -> PyErr {
        PyValueError::new_err(err.to_string())
    }
}

#[pyfunction]
fn parse_number(text: &str) -> PyResult<f64> {
    Ok(parse_flexible_number(text)?)   // `?` converts the error
}
```

```python
>>> parse_number("abc")
Traceback (most recent call last):
ValueError: not a number: "abc"
```

Rust forces the library author to handle the `Err`; Python's exception
machinery forces the caller to notice it. Your error enum's `Display`
string becomes the exception message — write it for the traceback reader.

### `#[pyclass]`: structs become Python classes

```rust
#[pyclass(frozen)]               // like @dataclass(frozen=True)
struct TextStats { ... }

#[pymethods]
impl TextStats {
    #[getter]
    fn words(&self) -> usize { ... }   // becomes a read-only property

    fn __repr__(&self) -> String { ... }  // dunders work too
}
```

From Python it's an ordinary object: `stats.words`, `repr(stats)`. The
data lives in Rust; Python holds a handle.

## The maturin Workflow

[maturin](https://github.com/PyO3/maturin) is to PyO3 what `setuptools`
is to C extensions — except it actually handles the hard parts. A minimal
`pyproject.toml`:

```toml
[build-system]
requires = ["maturin>=1.0,<2.0"]
build-backend = "maturin"

[project]
name = "textstats"
requires-python = ">=3.9"

[tool.maturin]
features = ["pyo3/extension-module"]
```

The development loop:

```bash
pip install maturin
maturin develop    # compile + install into the active venv
python -c "import textstats; print(textstats.add(2, 3))"
```

And for shipping:

```bash
maturin build --release    # produces a wheel in target/wheels/
```

Because this chapter's crate uses `abi3-py39` (Python's **stable ABI**),
one compiled wheel works on every CPython from 3.9 up — no per-version
rebuilds. Build wheels for each OS/architecture in CI, publish to PyPI,
and your users just run `pip install textstats`. They never see Rust:
no toolchain, no compile step, no `error: linker 'cc' not found`. That is
the distribution story C extensions never quite delivered.

## Why the Feature Gate?

Look at this chapter's `Cargo.toml`: pyo3 is `optional = true`, enabled
only by the `python` feature, and the binding module sits behind
`#[cfg(feature = "python")]`. This is deliberate, and it's how real
mixed crates ship:

- **The core is pure Rust.** `cargo test` runs everywhere — including
  this repo's CI — with no Python interpreter in sight.
- **The bindings are a thin shim.** They convert types and map errors;
  the logic they wrap is already tested.
- **The wheel build opts in:** `maturin develop --features python`.

```bash
cargo test -p ch06-ffi-pyo3                       # the default: pure Rust
cargo check -p ch06-ffi-pyo3 --features python    # type-check the glue
```

Keep your logic and your bindings separable and you can test, benchmark,
and reuse the core as a normal Rust crate — the Python package is just
one consumer of it.

## Summary

| Python | Rust + PyO3 | What Changes |
|--------|-------------|-------------|
| `ctypes` argtypes/restype | Rust function signatures | Types declared once, checked by the compiler |
| `PyArg_ParseTuple`, refcounting | `#[pyfunction]` | The macro writes the glue |
| C extension classes | `#[pyclass]` + `#[pymethods]` | A struct with attributes |
| `errno`, `NULL` returns | `PyResult` / `From<E> for PyErr` | Errors become real exceptions |
| `setup.py` + compiler flags | `maturin` | One tool: develop, build, publish |
| One wheel per Python version | `abi3` stable ABI | One wheel for 3.9+ |
| Segfault at a distance | `TypeError` at the call site | The boundary checks everything |

## Next Steps

Open `src/lib.rs` to see the raw C ABI next to the PyO3 layer, then try
the exercises in `exercises/` — they practice the conversion-friendly
shapes and exception-ready error types that make a Rust core easy to bind.
