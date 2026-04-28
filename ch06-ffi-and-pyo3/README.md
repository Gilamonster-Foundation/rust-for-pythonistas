# Chapter 6: FFI & PyO3

## The Big Idea

You've spent five chapters learning Rust through the lens of Python.
Now the question becomes: do you have to *choose*? Can you use both?

The answer is yes — and this is one of Rust's strongest practical
advantages. PyO3 lets you write Rust code that Python can call as if
it were a native Python module. No C glue code, no ctypes, no CFFI.
You write Rust, and Python sees a normal module with classes and functions.

This chapter teaches the *design patterns* for building Python-callable
Rust libraries. The examples and exercises are pure Rust (no PyO3
compilation required) — they teach you how to structure code at the
boundary so that when you're ready to add PyO3, the hard design work
is already done.

## Why Rust + Python?

Python is excellent for:
- Rapid prototyping and scripting
- Data science and ML ecosystems (numpy, pandas, scikit-learn)
- Glue code and orchestration

Rust is excellent for:
- CPU-intensive computation
- Memory-safe systems code
- Guaranteed-correct data transformations

The best architecture uses both: Python for the outer layer (user
interaction, orchestration, data pipeline wiring) and Rust for the
inner layer (hashing, parsing, validation, transformation). PyO3
is the bridge.

## The Boundary Design Pattern

The most important concept in this chapter isn't a Rust feature — it's
a design pattern. When building a Rust library that Python will call,
you need **two layers**:

```
Python code
    ↓
┌─────────────────────────────┐
│  Boundary layer (PyO3)      │  ← Converts Python types ↔ Rust types
│  - Python-friendly API      │  ← Error handling: Result → PyErr
│  - Accepts/returns Py types │  ← Owns the GIL interaction
└─────────────────────────────┘
    ↓
┌─────────────────────────────┐
│  Core layer (pure Rust)     │  ← No PyO3 dependency
│  - Business logic           │  ← Testable with cargo test
│  - Rust-idiomatic types     │  ← Can be used from other Rust code too
└─────────────────────────────┘
```

The boundary layer is thin: it converts types and handles errors.
The core layer is where the real work happens. This separation means:

1. **Your core logic is testable without Python** — `cargo test` works
2. **Your core logic is reusable** — other Rust crates can use it directly
3. **The PyO3 layer is thin enough to be obvious** — easy to maintain
4. **You can change the Python API without changing the core** — and vice versa

### Python equivalent

```python
# This is like writing a C extension with ctypes, but much cleaner.
# Compare:

# ctypes (painful)
import ctypes
lib = ctypes.CDLL('./libhash.so')
lib.hash_bytes.restype = ctypes.c_char_p
lib.hash_bytes.argtypes = [ctypes.c_char_p, ctypes.c_size_t]
result = lib.hash_bytes(b"hello", 5)

# PyO3 (natural)
import my_rust_module
result = my_rust_module.hash_bytes(b"hello")  # just works
```

## What PyO3 Code Looks Like

You don't need to compile this — just read the pattern:

```rust
// The CORE layer — pure Rust, no PyO3
pub fn hash_bytes_core(data: &[u8]) -> String {
    blake3::hash(data).to_hex().to_string()
}

// The BOUNDARY layer — PyO3 wrapper
// (This is what you'd add when ready to build a Python module)
//
// use pyo3::prelude::*;
//
// #[pyfunction]
// fn hash_bytes(data: &[u8]) -> String {
//     hash_bytes_core(data)  // delegates to the core
// }
//
// #[pymodule]
// fn my_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
//     m.add_function(wrap_pyfunction!(hash_bytes, m)?)?;
//     Ok(())
// }
```

The pattern: **pure Rust core function** + **thin PyO3 wrapper that delegates**.

## Type Mapping: Python ↔ Rust

When designing the boundary, you need to know how types map:

| Python type | Rust type (core) | PyO3 boundary |
|-------------|-----------------|---------------|
| `str` | `String` / `&str` | automatic |
| `bytes` | `Vec<u8>` / `&[u8]` | automatic |
| `int` | `i64` / `u64` | automatic |
| `float` | `f64` | automatic |
| `bool` | `bool` | automatic |
| `list[T]` | `Vec<T>` | automatic |
| `dict[K,V]` | `HashMap<K,V>` | automatic |
| `None` | `Option<T>` → `None` | automatic |
| custom class | Rust struct | `#[pyclass]` + `#[pymethods]` |
| exception | `Result<T, E>` | `PyResult<T>` (auto-converts) |

Most basic types convert automatically. Custom types need `#[pyclass]`.

## Error Handling at the Boundary

```rust
// Core: returns Result with a Rust error type
pub fn parse_config_core(json: &str) -> Result<Config, ConfigError> {
    serde_json::from_str(json).map_err(ConfigError::Parse)
}

// Boundary: converts to PyResult (which becomes a Python exception)
//
// #[pyfunction]
// fn parse_config(json: &str) -> PyResult<PyConfig> {
//     let config = parse_config_core(json)
//         .map_err(|e| PyValueError::new_err(e.to_string()))?;
//     Ok(PyConfig::from(config))
// }
```

**Key insight:** Your core Rust code uses idiomatic `Result<T, E>`.
The boundary layer converts Rust errors into Python exceptions. The
core never knows about Python. The boundary never knows about business
logic.

## Summary

| Concept | What It Means |
|---------|--------------|
| Two-layer pattern | Core (pure Rust) + Boundary (PyO3 wrapper) |
| Type mapping | Most Python types auto-convert to/from Rust |
| Error handling | `Result<T, E>` in core → `PyResult<T>` at boundary |
| `#[pyfunction]` | Expose a Rust function to Python |
| `#[pyclass]` | Expose a Rust struct as a Python class |
| `#[pymethods]` | Add methods to a `#[pyclass]` |
| `#[pymodule]` | Define the Python module entry point |
| maturin | Build tool that packages Rust as a Python wheel |

## What to Do Next

The exercises in this chapter focus on designing the **core layer** —
writing Rust code that's structured for Python interop without actually
requiring PyO3. When you're ready to build a real Python module:

1. Install maturin: `pip install maturin`
2. Create a new project: `maturin init --bindings pyo3`
3. Move your core code in, write the thin boundary layer
4. Build: `maturin develop` (installs into your venv)
5. Import from Python: `import my_module`

## Next Steps

Open `src/lib.rs` to see the core-layer patterns in working code,
then try the exercises in `exercises/`.
