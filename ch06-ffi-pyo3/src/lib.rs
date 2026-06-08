//! # Chapter 6: FFI & PyO3
//!
//! This module shows the two ways Rust code reaches Python: the raw C ABI
//! (what `ctypes` calls — manual, fragile, segfault-prone) and PyO3
//! (declarative bindings where a macro writes the glue for you).
//!
//! The architecture here is the one real mixed crates use: the core logic
//! is **pure Rust** with thorough tests and zero Python knowledge. The
//! PyO3 binding layer lives at the bottom of this file behind
//! `#[cfg(feature = "python")]` — off by default, so `cargo test` never
//! needs a Python interpreter.
//!
//! Run the tests: `cargo test -p ch06-ffi-pyo3`
//! Type-check the bindings: `cargo check -p ch06-ffi-pyo3 --features python`

use std::collections::{HashMap, HashSet};
use std::fmt;

// ---------------------------------------------------------------------------
// 1. The hard way: the raw C ABI (what ctypes sees)
// ---------------------------------------------------------------------------

/// A function exported with the C calling convention.
///
/// This is the *only* kind of function `ctypes` can call. From Python:
/// ```python
/// import ctypes
/// lib = ctypes.CDLL("./target/release/libch06_ffi_pyo3.so")
/// lib.ffi_add.argtypes = [ctypes.c_int64, ctypes.c_int64]
/// lib.ffi_add.restype = ctypes.c_int64
/// lib.ffi_add(2, 3)  # 5
/// ```
///
/// Notice everything YOU have to get right: the library path, the argument
/// types, the return type. Get `restype` wrong and ctypes happily
/// reinterprets the bits — no error, just garbage numbers.
///
/// `#[no_mangle]` keeps the symbol name `ffi_add` instead of a mangled
/// Rust name, and `extern "C"` uses the C calling convention.
#[no_mangle]
pub extern "C" fn ffi_add(a: i64, b: i64) -> i64 {
    a + b
}

/// The C ABI has no `&str`, no `Vec`, no `Option` — only pointers and
/// integers. Passing text means passing a raw pointer and a length, and
/// trusting the caller got both right.
///
/// From Python, the caller side looks like this:
/// ```python
/// data = b"hello world, hello ffi"
/// lib.ffi_count_spaces.argtypes = [ctypes.c_char_p, ctypes.c_size_t]
/// lib.ffi_count_spaces.restype = ctypes.c_uint64
/// lib.ffi_count_spaces(data, len(data))  # 3
/// # Pass len(data) + 1000 instead and you read past the buffer.
/// # Best case: garbage. Worst case: segfault. ctypes won't stop you.
/// ```
///
/// # Safety
///
/// `ptr` must be non-null and point to `len` initialized bytes that remain
/// valid for the duration of the call. The Rust compiler cannot check this
/// — that's exactly the contract C extensions have always made you uphold
/// by hand, and exactly what PyO3 automates away.
#[no_mangle]
pub unsafe extern "C" fn ffi_count_spaces(ptr: *const u8, len: usize) -> u64 {
    if ptr.is_null() {
        return 0;
    }
    let bytes = std::slice::from_raw_parts(ptr, len);
    bytes.iter().filter(|&&b| b == b' ').count() as u64
}

// ---------------------------------------------------------------------------
// 2. The pure-Rust core: a small text-statistics library
// ---------------------------------------------------------------------------
//
// Everything from here down to the `python` module is ordinary Rust.
// No FFI, no unsafe, no Python types. This is the part you test hard,
// because it's the part that does the actual work. The binding layer
// below is thin enough to verify by inspection.

/// Statistics about a piece of text.
///
/// Python equivalent (what the binding layer will expose):
/// ```python
/// @dataclass(frozen=True)
/// class TextStats:
///     lines: int
///     words: int
///     chars: int
///     unique_words: int
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextStats {
    pub lines: usize,
    pub words: usize,
    pub chars: usize,
    pub unique_words: usize,
}

/// Analyze a piece of text.
///
/// Python equivalent:
/// ```python
/// def analyze(text: str) -> TextStats:
///     words = text.split()
///     return TextStats(
///         lines=len(text.splitlines()),
///         words=len(words),
///         chars=len(text),
///         unique_words=len({w.lower() for w in words}),
///     )
/// ```
pub fn analyze(text: &str) -> TextStats {
    let words: Vec<&str> = text.split_whitespace().collect();
    let unique: HashSet<String> = words.iter().map(|w| w.to_lowercase()).collect();
    TextStats {
        lines: text.lines().count(),
        words: words.len(),
        chars: text.chars().count(),
        unique_words: unique.len(),
    }
}

/// The `n` most frequent words, most frequent first.
///
/// Python equivalent:
/// ```python
/// from collections import Counter
/// def top_words(text: str, n: int) -> list[tuple[str, int]]:
///     words = [w.strip(string.punctuation).lower() for w in text.split()]
///     return Counter(w for w in words if w).most_common(n)
/// ```
///
/// Ties are broken alphabetically so the output is deterministic —
/// `Counter.most_common` leaves tie order unspecified; we do better.
pub fn top_words(text: &str, n: usize) -> Vec<(String, usize)> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for raw in text.split_whitespace() {
        let cleaned = raw
            .trim_matches(|c: char| !c.is_alphanumeric())
            .to_lowercase();
        if !cleaned.is_empty() {
            *counts.entry(cleaned).or_insert(0) += 1;
        }
    }
    let mut pairs: Vec<(String, usize)> = counts.into_iter().collect();
    pairs.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    pairs.truncate(n);
    pairs
}

/// Count occurrences of each word in a list.
///
/// Python equivalent:
/// ```python
/// def tally(words: list[str]) -> dict[str, int]:
///     counts = {}
///     for w in words:
///         counts[w] = counts.get(w, 0) + 1
///     return counts
/// ```
///
/// Note the *shapes* here: this function takes a slice and returns a
/// `HashMap`. When PyO3 wraps it, the Python caller passes a `list[str]`
/// and gets back a `dict[str, int]` — the conversions are automatic.
pub fn tally(words: &[String]) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for w in words {
        *counts.entry(w.clone()).or_insert(0) += 1;
    }
    counts
}

// ---------------------------------------------------------------------------
// 3. Errors the Python side can understand
// ---------------------------------------------------------------------------

/// Things that can go wrong when parsing a number from messy input.
///
/// Python equivalent: there isn't one, really. Python functions just
/// `raise ValueError(...)`. In Rust we name each failure mode, and the
/// binding layer maps the whole enum to `ValueError` (section 4).
///
/// Contrast this with the C-extension world: a C function signals failure
/// by returning -1 or NULL and setting `errno` — and ctypes won't even
/// check *that* unless you ask. Forget the check and the error silently
/// becomes a "valid" value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NumberError {
    /// The input was empty or only whitespace.
    Empty,
    /// The input wasn't a number; carries the offending text.
    Invalid(String),
}

impl fmt::Display for NumberError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NumberError::Empty => write!(f, "empty input: nothing to parse"),
            NumberError::Invalid(s) => write!(f, "not a number: {s:?}"),
        }
    }
}

impl std::error::Error for NumberError {}

/// Parse a number the way spreadsheet exports write them: surrounding
/// whitespace and thousands separators allowed.
///
/// Python equivalent:
/// ```python
/// def parse_flexible_number(text: str) -> float:
///     cleaned = text.strip().replace(",", "")
///     if not cleaned:
///         raise ValueError("empty input: nothing to parse")
///     return float(cleaned)  # raises ValueError on bad input
/// ```
///
/// The Rust version returns `Result` instead of raising. The caller MUST
/// handle the error case — there is no "forgot the try/except" failure mode.
pub fn parse_flexible_number(input: &str) -> Result<f64, NumberError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(NumberError::Empty);
    }
    let cleaned: String = trimmed.chars().filter(|&c| c != ',').collect();
    cleaned
        .parse::<f64>()
        .map_err(|_| NumberError::Invalid(trimmed.to_string()))
}

// ---------------------------------------------------------------------------
// 4. The PyO3 binding layer — feature-gated, off by default
// ---------------------------------------------------------------------------
//
// This module only exists when you build with `--features python`.
// The default build (and this repo's CI) never compiles it, never links
// libpython, and never needs a Python interpreter. That separation is
// the whole architecture: a pure-Rust core anyone can `cargo test`, plus
// an optional shim that maturin compiles into a wheel.
//
// Compare the amount of code here with what a hand-written C extension
// needs — PyArg_ParseTuple format strings, manual refcounting with
// Py_INCREF/Py_DECREF, a PyMethodDef table, a PyModuleDef struct, an
// init function... PyO3's macros generate all of it from plain signatures.

#[cfg(feature = "python")]
mod python {
    use super::{NumberError, TextStats};
    use pyo3::exceptions::PyValueError;
    use pyo3::prelude::*;
    use std::collections::HashMap;

    // -- Error mapping: Rust Err -> Python exception --------------------
    //
    // `#[pyfunction]`s return PyResult<T>. The `?` operator converts our
    // NumberError into a PyErr via this From impl, and PyO3 raises it as
    // a real Python exception on the other side of the boundary:
    //
    //   >>> parse_number("abc")
    //   Traceback (most recent call last):
    //   ValueError: not a number: "abc"
    //
    // No errno. No silently-ignored return codes. Rust's "you must handle
    // the Err" becomes Python's "an exception was raised".
    impl From<NumberError> for PyErr {
        fn from(err: NumberError) -> PyErr {
            PyValueError::new_err(err.to_string())
        }
    }

    // -- A #[pyclass]: a Rust struct Python can hold --------------------
    //
    // This wrapper turns the pure-Rust TextStats into a Python object.
    // `name = "TextStats"` is what Python sees; `frozen` makes it
    // immutable, like @dataclass(frozen=True).
    #[pyclass(name = "TextStats", frozen)]
    struct PyTextStats {
        inner: TextStats,
    }

    #[pymethods]
    impl PyTextStats {
        // #[getter] methods become read-only properties: stats.words
        #[getter]
        fn lines(&self) -> usize {
            self.inner.lines
        }

        #[getter]
        fn words(&self) -> usize {
            self.inner.words
        }

        #[getter]
        fn chars(&self) -> usize {
            self.inner.chars
        }

        #[getter]
        fn unique_words(&self) -> usize {
            self.inner.unique_words
        }

        // Dunder methods work too — this is Python's repr(stats).
        fn __repr__(&self) -> String {
            format!(
                "TextStats(lines={}, words={}, chars={}, unique_words={})",
                self.inner.lines, self.inner.words, self.inner.chars, self.inner.unique_words
            )
        }
    }

    // -- #[pyfunction]s: thin wrappers over the pure-Rust core ----------
    //
    // Each wrapper is one or two lines. The type conversions happen in
    // the generated glue:
    //
    //   Python str        -> &str          (borrowed, zero-copy read)
    //   Python list[str]  -> Vec<String>
    //   Rust HashMap      -> Python dict
    //   Rust Vec<(S, u)>  -> Python list[tuple[str, int]]
    //   Rust Err(e)       -> raised exception
    //
    // Pass a list of ints where list[str] is expected and you get a
    // TypeError at the call site — not a segfault three frames later.

    /// analyze(text) -> TextStats
    #[pyfunction]
    fn analyze(text: &str) -> PyTextStats {
        PyTextStats {
            inner: super::analyze(text),
        }
    }

    /// top_words(text, n=10) -> list[tuple[str, int]]
    ///
    /// The signature attribute gives Python callers a default argument,
    /// exactly like `def top_words(text, n=10)`.
    #[pyfunction]
    #[pyo3(signature = (text, n = 10))]
    fn top_words(text: &str, n: usize) -> Vec<(String, usize)> {
        super::top_words(text, n)
    }

    /// tally(words) -> dict[str, int]
    #[pyfunction]
    fn tally(words: Vec<String>) -> HashMap<String, usize> {
        super::tally(&words)
    }

    /// parse_number(text) -> float, raising ValueError on bad input.
    #[pyfunction]
    fn parse_number(text: &str) -> PyResult<f64> {
        // `?` converts NumberError -> PyErr via the From impl above.
        Ok(super::parse_flexible_number(text)?)
    }

    // -- The #[pymodule]: what `import ch06_ffi_pyo3` finds -------------
    //
    // The function name must match the compiled module's import name.
    // maturin builds the cdylib, names it correctly for the platform
    // (.so / .pyd), and packages it as a wheel.
    #[pymodule]
    fn ch06_ffi_pyo3(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add_function(wrap_pyfunction!(analyze, m)?)?;
        m.add_function(wrap_pyfunction!(top_words, m)?)?;
        m.add_function(wrap_pyfunction!(tally, m)?)?;
        m.add_function(wrap_pyfunction!(parse_number, m)?)?;
        m.add_class::<PyTextStats>()?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests — pure Rust, no Python interpreter required
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // The C ABI layer

    #[test]
    fn ffi_add_works() {
        assert_eq!(ffi_add(2, 3), 5);
        assert_eq!(ffi_add(-10, 4), -6);
    }

    #[test]
    fn ffi_count_spaces_works() {
        let data = b"hello world, hello ffi";
        // SAFETY: pointer and length both come from the same live slice.
        let count = unsafe { ffi_count_spaces(data.as_ptr(), data.len()) };
        assert_eq!(count, 3);
    }

    #[test]
    fn ffi_count_spaces_null_is_zero() {
        // SAFETY: the function documents that null is checked and returns 0.
        let count = unsafe { ffi_count_spaces(std::ptr::null(), 0) };
        assert_eq!(count, 0);
    }

    #[test]
    fn ffi_count_spaces_short_len_undercounts() {
        // This is the ctypes trap in miniature: the length is the contract.
        // Pass a smaller len and you silently get a different answer.
        let data = b"a b c";
        // SAFETY: 3 <= data.len(), so the read stays in bounds.
        let count = unsafe { ffi_count_spaces(data.as_ptr(), 3) };
        assert_eq!(count, 1); // only "a b" was visible
    }

    // The pure-Rust core: analyze

    #[test]
    fn analyze_counts_everything() {
        let stats = analyze("the quick brown fox\njumps over the lazy dog");
        assert_eq!(stats.lines, 2);
        assert_eq!(stats.words, 9);
        assert_eq!(stats.chars, 43);
        assert_eq!(stats.unique_words, 8); // "the" appears twice
    }

    #[test]
    fn analyze_empty_text() {
        let stats = analyze("");
        assert_eq!(
            stats,
            TextStats {
                lines: 0,
                words: 0,
                chars: 0,
                unique_words: 0,
            }
        );
    }

    #[test]
    fn analyze_unique_is_case_insensitive() {
        let stats = analyze("Rust rust RUST");
        assert_eq!(stats.words, 3);
        assert_eq!(stats.unique_words, 1);
    }

    #[test]
    fn analyze_counts_chars_not_bytes() {
        // Python's len("héllo") is 5; Rust's "héllo".len() is 6 (bytes).
        // We count chars so the Rust answer matches Python's intuition.
        let stats = analyze("héllo");
        assert_eq!(stats.chars, 5);
    }

    // top_words

    #[test]
    fn top_words_orders_by_frequency() {
        let words = top_words("apple banana apple cherry apple banana", 2);
        assert_eq!(
            words,
            vec![("apple".to_string(), 3), ("banana".to_string(), 2)]
        );
    }

    #[test]
    fn top_words_breaks_ties_alphabetically() {
        let words = top_words("beta alpha beta alpha", 2);
        assert_eq!(
            words,
            vec![("alpha".to_string(), 2), ("beta".to_string(), 2)]
        );
    }

    #[test]
    fn top_words_strips_punctuation_and_case() {
        let words = top_words("Hello, hello! HELLO?", 1);
        assert_eq!(words, vec![("hello".to_string(), 3)]);
    }

    #[test]
    fn top_words_empty_text() {
        assert!(top_words("", 5).is_empty());
    }

    // tally

    #[test]
    fn tally_counts_occurrences() {
        let words = vec!["a".to_string(), "b".to_string(), "a".to_string()];
        let counts = tally(&words);
        assert_eq!(counts.get("a"), Some(&2));
        assert_eq!(counts.get("b"), Some(&1));
        assert_eq!(counts.len(), 2);
    }

    #[test]
    fn tally_empty_list() {
        assert!(tally(&[]).is_empty());
    }

    // parse_flexible_number + error mapping

    #[test]
    fn parse_plain_number() {
        let value = parse_flexible_number("42.5").unwrap();
        assert!((value - 42.5).abs() < f64::EPSILON);
    }

    #[test]
    fn parse_with_whitespace_and_commas() {
        let value = parse_flexible_number("  1,234.56  ").unwrap();
        assert!((value - 1234.56).abs() < f64::EPSILON);
    }

    #[test]
    fn parse_negative() {
        let value = parse_flexible_number("-3.5").unwrap();
        assert!((value + 3.5).abs() < f64::EPSILON);
    }

    #[test]
    fn parse_empty_is_error() {
        assert_eq!(parse_flexible_number("   "), Err(NumberError::Empty));
    }

    #[test]
    fn parse_garbage_is_error() {
        assert_eq!(
            parse_flexible_number("abc"),
            Err(NumberError::Invalid("abc".to_string()))
        );
    }

    #[test]
    fn error_messages_read_like_python_exceptions() {
        // These Display strings are what Python users will see as str(exc)
        // once the binding layer maps NumberError -> ValueError.
        assert_eq!(
            NumberError::Empty.to_string(),
            "empty input: nothing to parse"
        );
        assert_eq!(
            NumberError::Invalid("abc".to_string()).to_string(),
            "not a number: \"abc\""
        );
    }
}
