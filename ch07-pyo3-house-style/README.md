# Chapter 7: PyO3 House Style

## The Big Idea

Chapter 6 introduced the two-layer pattern: a pure-Rust core plus a thin
PyO3 boundary. This chapter turns that pattern into a **house style** — the
conventions for shipping a Rust core with Python reach. One codebase, one
source of truth, two ecosystems served: a crate on crates.io and a wheel
on PyPI, with neither audience treated as second-class.

The style is five rules. Each one answers the same question from the
roadmap: *how should this feel from Python?* — without ever letting the
answer leak into the Rust core.

## Rule 1: The Core Never Mentions Python

The core logic is a plain Rust crate. The bindings are a skin, gated
behind a default-off feature:

```toml
[dependencies]
pyo3 = { version = "0.23", optional = true, features = ["extension-module", "abi3-py39"] }

[features]
python = ["dep:pyo3"]
```

```rust
// ... an entire crate of pure Rust, then at the very bottom:

#[cfg(feature = "python")]
mod python {
    // the ONLY module allowed to say `use pyo3`
}
```

Why this is load-bearing:

- **crates.io users pay nothing.** No pyo3 in their dependency tree, no
  Python toolchain at build time, no compile-time cost for a face they
  never see.
- **Tests need no Python.** `cargo test` exercises all the logic. This
  repo's CI runs exactly that — plain `cargo test` on a machine with no
  Python build setup — and this chapter's tests pass there. The
  constraint isn't a workaround; it *is* the lesson.
- **The boundary stays honest.** If binding code can only live in one
  gated module, business logic can't quietly migrate into it.

`abi3-py39` is part of the rule: one compiled wheel works on every
CPython ≥ 3.9, instead of one wheel per Python version.

## Rule 2: A Pythonic Face on a Rusty Core

Python users should never sense they're holding Rust. Design the surface
they touch to match what a Python author would have written:

```rust
#[pymethods]
impl PyCatalog {
    #[pyo3(signature = (name, body = ""))]   // keyword args + defaults
    fn add(&mut self, name: &str, body: &str) -> PyResult<()> { ... }

    #[getter]                                 // catalog.names, a property
    fn names(&self) -> Vec<String> { ... }

    fn __len__(&self) -> usize { ... }        // len(catalog)
    fn __contains__(&self, name: &str) -> bool { ... }  // "x" in catalog
    fn __repr__(&self) -> String { ... }      // sensible repr()
}
```

```python
catalog = Catalog()
catalog.add("greeting", body="hello, world")
len(catalog)            # 1
"greeting" in catalog   # True
repr(catalog)           # 'Catalog(entries=1)'
```

Meanwhile the Rust API stays Rusty: borrowing getters, `Option` for
optional lookups, `Result` for failures, iterators instead of collected
lists. The two faces are *both* idiomatic because the boundary translates
between them — neither API contorts to imitate the other.

One trick worth stealing: implement `Display` on the core type and make
`__repr__` call `to_string()`. The repr is written once, in the core,
and can never drift between ecosystems.

## Rule 3: Errors Map at the Boundary — Once

The core defines **one error enum**. Every fallible function returns it.
At the edge, **one** `From` impl converts it to typed Python exceptions:

```rust
// CORE: errors as data, no Python anywhere
pub enum CatalogError {
    NotFound(String),            // a failed lookup
    DuplicateName(String),       // bad input
    Truncated { offset: usize }, // bad data
    InvalidUtf8 { offset: usize },
}

// BOUNDARY: the single mapping layer
impl From<CatalogError> for PyErr {
    fn from(err: CatalogError) -> PyErr {
        let message = err.to_string();
        match exception_kind(&err) {
            ExceptionKind::KeyError => PyKeyError::new_err(message),
            ExceptionKind::ValueError => PyValueError::new_err(message),
        }
    }
}
```

Now every binding is a one-liner — `self.inner.require(name)?` — and a
missing key raises a real `KeyError` in Python, exactly as a dict would.

The discipline that makes this durable:

- **No catch-all arm.** The mapping `match` lists every variant. Add a
  new error variant and the compiler stops you at the mapping and asks
  "and what exception is that?"
- **No stringly-typed errors cross the boundary.** A `Result<T, String>`
  can only ever become one generic exception; an enum can become
  `KeyError` here and `ValueError` there. Map *types*, not message text.
- **No hand-built `PyErr` in bindings.** If a binding constructs its own
  exception, the policy has started to scatter. All roads go through the
  one `From` impl.

## Rule 4: Know What Crosses the Line

The zero-copy mindset is about the FFI cost model. The rules are
asymmetric:

| Direction | Signature | Cost |
|-----------|-----------|------|
| Python `bytes` → Rust | `&[u8]` | **Free** — PyO3 borrows the caller's buffer |
| Python `str` → Rust | `&str` | **Free** — borrowed for the call |
| Rust → Python `str` | `String` / `.to_owned()` | **A copy** — Python strings own their memory |
| Rust → Python `bytes` | `PyBytes::new(py, ...)` | **A copy** — same reason |

So the house style is:

- **Keep the core zero-copy.** Functions like
  `fn peek_names(buf: &[u8]) -> Result<Vec<&str>, _>` return *views* into
  the input — no allocation, and the lifetime ties the views to the
  buffer at compile time. Rust callers get the full benefit.
- **Accept borrows at the boundary.** Taking `&[u8]` instead of
  `Vec<u8>` means Python `bytes` comes in without a copy.
- **Pay the copy only on the way out**, where it's unavoidable — the
  `.map(str::to_owned)` in a binding is an honest, visible cost, not an
  accident.
- **Don't validate what you skip.** The chapter's `peek_names` never
  UTF-8-checks the bodies it steps over. Zero-copy thinking is really
  zero-*waste* thinking.

For large binary payloads, Python's buffer protocol (`memoryview`) can
avoid even the outbound copy, at the price of pinning Rust memory while
Python holds the view. Know it exists; reach for it when profiling says
the copy matters, not before.

## Rule 5: One Source of Truth

The crates.io crate and the PyPI wheel are the *same code*, so keep them
provably the same:

- **One repo, one version number.** The wheel's version is the crate's
  version (maturin reads it straight from `Cargo.toml`). Never let the
  two surfaces drift apart.
- **Behavior parity by construction.** Every binding delegates to the
  core; the core's tests are therefore testing both ecosystems' behavior.
  If a binding contains logic, parity is on the honor system — don't.
- **README examples for both audiences.** A user landing from crates.io
  and a user landing from PyPI should each find a usage example in their
  own language:

```rust
// Rust users
let mut catalog = Catalog::new();
catalog.add("greeting", "hello, world")?;
assert_eq!(catalog.require("greeting")?, "hello, world");
```

```python
# Python users
from ch07_pyo3_house_style import Catalog
catalog = Catalog()
catalog.add("greeting", body="hello, world")
assert catalog.require("greeting") == "hello, world"
```

## Summary

| House rule | Mechanism | What it buys |
|-----------|-----------|--------------|
| Core never mentions Python | optional `pyo3` + `#[cfg(feature = "python")]` | Rust users pay nothing; tests need no Python |
| Pythonic face, Rusty core | `#[pyo3(signature)]`, `#[getter]`, dunders | Both APIs idiomatic, neither contorted |
| One error enum, one mapping | `From<CoreError> for PyErr`, no catch-alls | Typed exceptions; compiler-enforced coverage |
| Know what crosses the line | borrow in (`&[u8]`), copy out (`to_owned`) | Zero-copy core; honest, visible boundary costs |
| One source of truth | delegation-only bindings, shared version | crates.io and PyPI can't drift apart |

## Try It

```bash
cargo test -p ch07-pyo3-house-style                     # the core — no Python required
cargo check -p ch07-pyo3-house-style --features python  # compile the boundary layer too
```

## Next Steps

Open `src/lib.rs` to see the whole style in one annotated file — the
feature-gated `python` module at the bottom is the entire binding layer.
Then work the exercises in `exercises/`, which practice the same
disciplines (error design, exception mapping, borrow-don't-clone, API
shape) in pure Rust.
