//! # Chapter 7: PyO3 House Style
//!
//! Chapter 6 introduced the two-layer pattern: a pure-Rust core plus a thin
//! Python boundary. This chapter turns that pattern into a *house style* —
//! the conventions that let one codebase serve two ecosystems: a Rust crate
//! on crates.io and a Python wheel on PyPI, with one source of truth.
//!
//! The style has one load-bearing rule, and this crate demonstrates it
//! structurally: **the core never mentions Python.** All PyO3 code lives
//! behind a default-off `python` feature. Run the tests right now and you
//! compile zero lines of binding code and need no Python toolchain. That
//! is not a workaround for this course's CI — it IS the lesson. A
//! well-styled hybrid crate is a first-class Rust library that grows a
//! Python face only when asked:
//!
//! ```text
//! cargo test -p ch07-pyo3-house-style                     # core only — what crates.io users get
//! cargo check -p ch07-pyo3-house-style --features python  # core + bindings — what PyPI users get
//! ```
//!
//! Run the tests: `cargo test -p ch07-pyo3-house-style`

use std::fmt;

// ---------------------------------------------------------------------------
// 1. The layering rule — core code never mentions Python
// ---------------------------------------------------------------------------
//
// Everything from here down to the `python` module at the bottom of this
// file is plain Rust. It compiles without pyo3, tests without a Python
// interpreter, and could be published to crates.io as-is.
//
// Python equivalent of the *idea* (a package whose optional accelerator
// is invisible unless installed):
//
// ```python
// # pip install mylib          -> pure-Python behavior
// # pip install mylib[fast]    -> same API, native accelerator engaged
// try:
//     from mylib._native import decode
// except ImportError:
//     from mylib._pure import decode
// ```
//
// In Rust the switch happens at compile time instead of import time:
//
// ```toml
// [dependencies]
// pyo3 = { version = "0.23", optional = true, features = ["extension-module", "abi3-py39"] }
//
// [features]
// python = ["dep:pyo3"]
// ```
//
// `cargo build` ignores pyo3 entirely. `cargo build --features python`
// compiles the boundary layer. Rust users never pay for the Python face.

// ---------------------------------------------------------------------------
// 2. One error enum in the core
// ---------------------------------------------------------------------------

/// Every way this library can fail, as data.
///
/// Python equivalent — a small exception hierarchy:
/// ```python
/// class CatalogError(Exception): ...
/// class NotFoundError(CatalogError, KeyError): ...
/// class DuplicateNameError(CatalogError, ValueError): ...
/// class CorruptDataError(CatalogError, ValueError): ...
/// ```
///
/// House style: the core defines ONE error enum and every fallible
/// function returns it. No `Result<T, String>`, no `panic!` for expected
/// failures. Stringly-typed errors can't be matched on, can't be mapped
/// to distinct Python exceptions, and can't be extended without grepping
/// for message text. An enum can.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogError {
    /// A lookup for a name that isn't in the catalog.
    NotFound(String),
    /// An attempt to add a name that's already taken.
    DuplicateName(String),
    /// Encoded data ended mid-record. `offset` is where parsing stopped.
    Truncated { offset: usize },
    /// Encoded data contained invalid UTF-8 at `offset`.
    InvalidUtf8 { offset: usize },
}

impl fmt::Display for CatalogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(name) => write!(f, "no entry named {name:?}"),
            Self::DuplicateName(name) => write!(f, "entry {name:?} already exists"),
            Self::Truncated { offset } => write!(f, "data truncated at byte {offset}"),
            Self::InvalidUtf8 { offset } => write!(f, "invalid UTF-8 at byte {offset}"),
        }
    }
}

impl std::error::Error for CatalogError {}

// ---------------------------------------------------------------------------
// 3. The mapping layer — one place where errors become exceptions
// ---------------------------------------------------------------------------

/// Which Python exception type an error should surface as.
///
/// This enum is the *pure-Rust model* of the boundary decision. The actual
/// `From<CatalogError> for PyErr` impl (bottom of this file, feature-gated)
/// consumes it — but the decision itself is core logic, so it lives here
/// where `cargo test` can reach it without Python.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExceptionKind {
    /// Maps to Python's `KeyError` — a failed lookup.
    KeyError,
    /// Maps to Python's `ValueError` — bad input or bad data.
    ValueError,
}

/// Decide, exhaustively, which Python exception each error becomes.
///
/// Python users expect `KeyError` from a failed lookup and `ValueError`
/// from bad data — matching dict and bytes.decode() behavior. Meeting
/// that expectation is part of designing a Pythonic face.
///
/// House style: this `match` has NO catch-all arm. When you add a new
/// error variant, the compiler stops you right here and asks "and what
/// exception is that?" — the mapping can never silently fall behind the
/// error enum. A `_ => ValueError` arm would trade that guarantee away.
pub fn exception_kind(err: &CatalogError) -> ExceptionKind {
    match err {
        CatalogError::NotFound(_) => ExceptionKind::KeyError,
        CatalogError::DuplicateName(_)
        | CatalogError::Truncated { .. }
        | CatalogError::InvalidUtf8 { .. } => ExceptionKind::ValueError,
    }
}

// ---------------------------------------------------------------------------
// 4. A core type designed for two faces
// ---------------------------------------------------------------------------

/// An ordered collection of named text snippets.
///
/// Python users will see this as a class:
/// ```python
/// catalog = Catalog()
/// catalog.add("greeting", "hello, world")
/// catalog.require("greeting")     # -> "hello, world", or KeyError
/// len(catalog)                    # __len__
/// "greeting" in catalog           # __contains__
/// repr(catalog)                   # "Catalog(entries=1)"
/// ```
///
/// Rust users see an ordinary struct with borrowing getters and
/// `Result`-returning methods. Same behavior, two idiomatic faces —
/// that's the point. Neither API is a second-class translation of
/// the other.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Catalog {
    /// `Vec` rather than `HashMap`: insertion order is part of the
    /// contract (and of the encoded byte format below).
    entries: Vec<(String, String)>,
}

impl Catalog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a named snippet. Names are write-once.
    ///
    /// Returns `DuplicateName` instead of silently overwriting — the
    /// boundary layer will surface that as `ValueError`.
    pub fn add(&mut self, name: &str, body: &str) -> Result<(), CatalogError> {
        if self.contains(name) {
            return Err(CatalogError::DuplicateName(name.to_string()));
        }
        self.entries.push((name.to_string(), body.to_string()));
        Ok(())
    }

    /// Look up a snippet, Rust-style: `Option`, borrowed, zero-copy.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, body)| body.as_str())
    }

    /// Look up a snippet, boundary-style: a missing name is an *error*.
    ///
    /// Python has no `Option` — its idiom for a failed lookup is
    /// `KeyError`. The core provides both shapes so the boundary layer
    /// can delegate instead of re-implementing the policy.
    pub fn require(&self, name: &str) -> Result<&str, CatalogError> {
        self.get(name)
            .ok_or_else(|| CatalogError::NotFound(name.to_string()))
    }

    pub fn contains(&self, name: &str) -> bool {
        self.entries.iter().any(|(n, _)| n == name)
    }

    /// Iterate names in insertion order, borrowed.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.entries.iter().map(|(n, _)| n.as_str())
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// `Display` doubles as the model for Python's `__repr__`.
///
/// House style: write the repr once, in the core, and have the boundary's
/// `__repr__` call `to_string()`. One format, two ecosystems.
impl fmt::Display for Catalog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Catalog(entries={})", self.entries.len())
    }
}

// ---------------------------------------------------------------------------
// 5. Zero-copy parsing — what actually crosses the line
// ---------------------------------------------------------------------------
//
// The wire format: each entry is two length-prefixed chunks,
//
//     [name_len: u32 LE][name bytes][body_len: u32 LE][body bytes]
//
// repeated until the buffer ends. Parsing it is where the zero-copy
// mindset shows up. In Python, slicing bytes copies:
//
// ```python
// name = buf[4:4 + name_len].decode()   # allocates a new str
// ```
//
// In Rust, `&buf[4..4 + name_len]` is a *view* — a pointer and a length
// into memory you already own. The lifetime in `fn(&[u8]) -> &str` is the
// compiler-enforced promise that the view never outlives the buffer.
//
// At the FFI line the rules are asymmetric, and worth memorizing:
//
//   - Python `bytes` -> Rust `&[u8]`: FREE. PyO3 borrows the existing
//     buffer; no copy on the way in.
//   - Rust `&str` -> Python `str`: A COPY. Python strings own their
//     memory, so borrowed data must be materialized on the way out.
//
// So the house style is: keep the core zero-copy (Rust callers get the
// full benefit), accept borrows at the boundary (free), and pay the copy
// only on returned values — the one place it's unavoidable. (For large
// binary payloads, Python's buffer protocol / `memoryview` can avoid even
// that, at the cost of pinning Rust memory; know it exists, reach for it
// only when profiling says so.)

/// Find the byte range of the next length-prefixed chunk.
///
/// Bounds problems become `Truncated { offset }` — pointing at the chunk
/// that failed, because "your data is cut off at byte 17" beats "index
/// out of range".
fn chunk_range(buf: &[u8], offset: usize) -> Result<std::ops::Range<usize>, CatalogError> {
    let len_end = offset + 4;
    if buf.len() < len_end {
        return Err(CatalogError::Truncated { offset });
    }
    let len = u32::from_le_bytes(buf[offset..len_end].try_into().expect("4 bytes")) as usize;
    let data_end = len_end + len;
    if buf.len() < data_end {
        return Err(CatalogError::Truncated { offset });
    }
    Ok(len_end..data_end)
}

/// Read the next chunk as UTF-8 text — without copying.
///
/// Note the signature: the returned `&str` borrows from `buf`. No
/// allocation happens here; we validate in place and hand back a view.
fn read_str_chunk<'a>(buf: &'a [u8], offset: &mut usize) -> Result<&'a str, CatalogError> {
    let range = chunk_range(buf, *offset)?;
    let text = std::str::from_utf8(&buf[range.clone()]).map_err(|_| CatalogError::InvalidUtf8 {
        offset: range.start,
    })?;
    *offset = range.end;
    Ok(text)
}

/// Skip a chunk without reading it — bounds are checked, bytes are not.
///
/// "Don't pay for what you don't use": skipped chunks aren't UTF-8
/// validated, because nobody is going to look at them.
fn skip_chunk(buf: &[u8], offset: &mut usize) -> Result<(), CatalogError> {
    *offset = chunk_range(buf, *offset)?.end;
    Ok(())
}

impl Catalog {
    /// Encode the catalog to bytes. The one place encoding allocates.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        for (name, body) in &self.entries {
            for field in [name, body] {
                out.extend_from_slice(&(field.len() as u32).to_le_bytes());
                out.extend_from_slice(field.as_bytes());
            }
        }
        out
    }

    /// Decode a catalog from bytes, validating everything.
    ///
    /// The names and bodies are *stored*, so this copies them out of the
    /// buffer — an honest copy, because the `Catalog` outlives `buf`.
    /// Compare with `peek_names` below, which doesn't.
    pub fn from_bytes(buf: &[u8]) -> Result<Self, CatalogError> {
        let mut catalog = Catalog::new();
        let mut offset = 0;
        while offset < buf.len() {
            let name = read_str_chunk(buf, &mut offset)?;
            let body = read_str_chunk(buf, &mut offset)?;
            catalog.add(name, body)?;
        }
        Ok(catalog)
    }
}

/// List the entry names in an encoded catalog — zero-copy, zero-waste.
///
/// Python equivalent (which copies every slice it touches):
/// ```python
/// def peek_names(buf: bytes) -> list[str]:
///     names, offset = [], 0
///     while offset < len(buf):
///         n = int.from_bytes(buf[offset:offset + 4], "little")
///         names.append(buf[offset + 4:offset + 4 + n].decode())
///         offset += 4 + n
///         n = int.from_bytes(buf[offset:offset + 4], "little")
///         offset += 4 + n   # skip the body
///     return names
/// ```
///
/// The Rust version allocates nothing for the names (they're views into
/// `buf`) and never even UTF-8-validates the bodies it skips. The
/// returned strings are borrowed — the signature ties their lifetime to
/// the buffer's, and the compiler enforces it.
pub fn peek_names(buf: &[u8]) -> Result<Vec<&str>, CatalogError> {
    let mut names = Vec::new();
    let mut offset = 0;
    while offset < buf.len() {
        names.push(read_str_chunk(buf, &mut offset)?);
        skip_chunk(buf, &mut offset)?;
    }
    Ok(names)
}

// ---------------------------------------------------------------------------
// 6. The boundary layer — feature-gated, thin, and Pythonic
// ---------------------------------------------------------------------------
//
// Everything below compiles ONLY with `--features python`. This module is
// the entire Python face of the crate: it converts types, maps errors,
// and adds dunder methods. It contains no business logic — every method
// is a delegation to the core above. If you find yourself writing an
// `if` in this module that isn't about conversion, it belongs in the core.
//
// One source of truth, two ecosystems served:
//
// ```python
// from ch07_pyo3_house_style import Catalog, peek_names
//
// catalog = Catalog()
// catalog.add("greeting", body="hello, world")   # kwargs via #[pyo3(signature)]
// catalog.require("greeting")                    # "hello, world"
// catalog.require("missing")                     # raises KeyError
// catalog.add("greeting", body="again")          # raises ValueError
// peek_names(catalog.to_bytes())                 # ["greeting"]
// ```

#[cfg(feature = "python")]
mod python {
    use pyo3::exceptions::{PyKeyError, PyValueError};
    use pyo3::prelude::*;
    use pyo3::types::PyBytes;

    use super::{exception_kind, Catalog, CatalogError, ExceptionKind};

    /// THE error-mapping layer. Singular.
    ///
    /// Because this impl exists, every binding below can use `?` on a
    /// core `Result` and the right typed Python exception comes out.
    /// No binding ever constructs a PyErr by hand — if one did, the
    /// mapping policy would start to scatter.
    impl From<CatalogError> for PyErr {
        fn from(err: CatalogError) -> PyErr {
            let message = err.to_string();
            match exception_kind(&err) {
                ExceptionKind::KeyError => PyKeyError::new_err(message),
                ExceptionKind::ValueError => PyValueError::new_err(message),
            }
        }
    }

    /// The Python class. A newtype over the core — nothing more.
    #[pyclass(name = "Catalog")]
    struct PyCatalog {
        inner: Catalog,
    }

    #[pymethods]
    impl PyCatalog {
        #[new]
        fn new() -> Self {
            Self {
                inner: Catalog::new(),
            }
        }

        /// `#[pyo3(signature = ...)]` gives Python callers keyword
        /// arguments and defaults — `catalog.add("name", body="...")` —
        /// without contorting the Rust API to match.
        #[pyo3(signature = (name, body = ""))]
        fn add(&mut self, name: &str, body: &str) -> PyResult<()> {
            // `?` + the From impl above = the entire error story.
            self.inner.add(name, body)?;
            Ok(())
        }

        /// `Option<String>` surfaces as `str | None` — Pythonic for an
        /// optional lookup. The `.map(str::to_owned)` is the honest copy
        /// at the boundary: a Python str must own its memory.
        fn get(&self, name: &str) -> Option<String> {
            self.inner.get(name).map(str::to_owned)
        }

        /// The error-raising lookup: `PyResult` + the From impl turn
        /// `CatalogError::NotFound` into a real `KeyError`.
        fn require(&self, name: &str) -> PyResult<String> {
            Ok(self.inner.require(name)?.to_owned())
        }

        /// A `#[getter]` makes this `catalog.names` (a property), not
        /// `catalog.names()` — match what a Python author would write.
        #[getter]
        fn names(&self) -> Vec<String> {
            self.inner.names().map(str::to_owned).collect()
        }

        /// Returning `PyBytes` explicitly: one allocation, owned by
        /// Python, no intermediate `Vec` handoff semantics to explain.
        fn to_bytes<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
            PyBytes::new(py, &self.inner.to_bytes())
        }

        /// `&[u8]` here means Python `bytes` comes in WITHOUT a copy —
        /// PyO3 borrows the caller's buffer for the duration of the call.
        #[staticmethod]
        fn from_bytes(data: &[u8]) -> PyResult<Self> {
            Ok(Self {
                inner: Catalog::from_bytes(data)?,
            })
        }

        // The dunder protocol: len(), `in`, and repr() — implemented by
        // delegation, so Python ergonomics never fork from core behavior.

        fn __len__(&self) -> usize {
            self.inner.len()
        }

        fn __contains__(&self, name: &str) -> bool {
            self.inner.contains(name)
        }

        fn __repr__(&self) -> String {
            // Display IS the repr. Written once, in the core.
            self.inner.to_string()
        }
    }

    /// Free function binding. The core returns borrowed `Vec<&str>`;
    /// the boundary materializes owned strings because they're about to
    /// become Python objects. Borrow inside, copy at the line.
    #[pyfunction]
    fn peek_names(buf: &[u8]) -> PyResult<Vec<String>> {
        Ok(super::peek_names(buf)?
            .into_iter()
            .map(str::to_owned)
            .collect())
    }

    /// The module definition — the table of contents of the Python face.
    /// (A shipping project would build this into a wheel with maturin.)
    #[pymodule]
    fn ch07_pyo3_house_style(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add_class::<PyCatalog>()?;
        m.add_function(wrap_pyfunction!(peek_names, m)?)?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests — note: pure Rust, no Python anywhere
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // The error enum

    #[test]
    fn errors_display_with_context() {
        assert_eq!(
            CatalogError::NotFound("greeting".to_string()).to_string(),
            "no entry named \"greeting\""
        );
        assert_eq!(
            CatalogError::Truncated { offset: 17 }.to_string(),
            "data truncated at byte 17"
        );
    }

    // The mapping layer — every variant has a deliberate destination

    #[test]
    fn failed_lookup_maps_to_key_error() {
        let err = CatalogError::NotFound("x".to_string());
        assert_eq!(exception_kind(&err), ExceptionKind::KeyError);
    }

    #[test]
    fn bad_input_maps_to_value_error() {
        assert_eq!(
            exception_kind(&CatalogError::DuplicateName("x".to_string())),
            ExceptionKind::ValueError
        );
        assert_eq!(
            exception_kind(&CatalogError::Truncated { offset: 0 }),
            ExceptionKind::ValueError
        );
        assert_eq!(
            exception_kind(&CatalogError::InvalidUtf8 { offset: 4 }),
            ExceptionKind::ValueError
        );
    }

    // The core type

    #[test]
    fn add_and_get() {
        let mut catalog = Catalog::new();
        catalog.add("greeting", "hello, world").unwrap();
        assert_eq!(catalog.get("greeting"), Some("hello, world"));
        assert_eq!(catalog.get("missing"), None);
    }

    #[test]
    fn require_found_and_missing() {
        let mut catalog = Catalog::new();
        catalog.add("greeting", "hello").unwrap();
        assert_eq!(catalog.require("greeting"), Ok("hello"));
        assert_eq!(
            catalog.require("missing"),
            Err(CatalogError::NotFound("missing".to_string()))
        );
    }

    #[test]
    fn duplicate_names_rejected() {
        let mut catalog = Catalog::new();
        catalog.add("greeting", "hello").unwrap();
        assert_eq!(
            catalog.add("greeting", "again"),
            Err(CatalogError::DuplicateName("greeting".to_string()))
        );
        // The original entry is untouched.
        assert_eq!(catalog.get("greeting"), Some("hello"));
    }

    #[test]
    fn names_preserve_insertion_order() {
        let mut catalog = Catalog::new();
        catalog.add("b", "2").unwrap();
        catalog.add("a", "1").unwrap();
        let names: Vec<&str> = catalog.names().collect();
        assert_eq!(names, vec!["b", "a"]);
    }

    #[test]
    fn len_contains_empty() {
        let mut catalog = Catalog::new();
        assert!(catalog.is_empty());
        catalog.add("x", "1").unwrap();
        assert_eq!(catalog.len(), 1);
        assert!(catalog.contains("x"));
        assert!(!catalog.contains("y"));
    }

    #[test]
    fn display_is_the_repr() {
        let mut catalog = Catalog::new();
        catalog.add("a", "1").unwrap();
        catalog.add("b", "2").unwrap();
        assert_eq!(catalog.to_string(), "Catalog(entries=2)");
    }

    // The wire format

    #[test]
    fn bytes_roundtrip() {
        let mut catalog = Catalog::new();
        catalog.add("greeting", "hello, world").unwrap();
        catalog.add("farewell", "goodbye").unwrap();

        let decoded = Catalog::from_bytes(&catalog.to_bytes()).unwrap();
        assert_eq!(decoded, catalog);
    }

    #[test]
    fn empty_catalog_roundtrip() {
        let catalog = Catalog::new();
        let bytes = catalog.to_bytes();
        assert!(bytes.is_empty());
        assert_eq!(Catalog::from_bytes(&bytes).unwrap(), catalog);
    }

    #[test]
    fn truncated_length_prefix() {
        // Two bytes can't hold a 4-byte length prefix.
        let err = Catalog::from_bytes(&[1, 0]).unwrap_err();
        assert_eq!(err, CatalogError::Truncated { offset: 0 });
    }

    #[test]
    fn truncated_chunk_body() {
        // Length says 10 bytes follow; only 1 does.
        let err = Catalog::from_bytes(&[10, 0, 0, 0, b'a']).unwrap_err();
        assert_eq!(err, CatalogError::Truncated { offset: 0 });
    }

    #[test]
    fn invalid_utf8_reports_offset() {
        // A 3-byte name chunk containing invalid UTF-8.
        let err = Catalog::from_bytes(&[3, 0, 0, 0, 0xFF, 0xFE, 0xFF]).unwrap_err();
        assert_eq!(err, CatalogError::InvalidUtf8 { offset: 4 });
    }

    #[test]
    fn duplicate_in_encoded_data_rejected() {
        let mut catalog = Catalog::new();
        catalog.add("a", "1").unwrap();
        let mut bytes = catalog.to_bytes();
        let again = bytes.clone();
        bytes.extend_from_slice(&again); // entry "a" appears twice

        assert_eq!(
            Catalog::from_bytes(&bytes).unwrap_err(),
            CatalogError::DuplicateName("a".to_string())
        );
    }

    // Zero-copy parsing

    #[test]
    fn peek_names_lists_names_in_order() {
        let mut catalog = Catalog::new();
        catalog.add("first", "1").unwrap();
        catalog.add("second", "2").unwrap();

        let bytes = catalog.to_bytes();
        assert_eq!(peek_names(&bytes).unwrap(), vec!["first", "second"]);
    }

    #[test]
    fn peek_names_borrows_from_the_buffer() {
        let mut catalog = Catalog::new();
        catalog.add("zero-copy", "body").unwrap();
        let bytes = catalog.to_bytes();

        let names = peek_names(&bytes).unwrap();
        // The returned &str points INTO `bytes` — same memory, no copy.
        // (The name chunk starts after its 4-byte length prefix.)
        assert_eq!(names[0].as_ptr(), bytes[4..].as_ptr());
    }

    #[test]
    fn peek_names_skips_body_validation() {
        // name "a" is valid; the BODY is invalid UTF-8.
        let bytes = [1, 0, 0, 0, b'a', 2, 0, 0, 0, 0xFF, 0xFE];

        // Full decoding validates everything and fails...
        assert_eq!(
            Catalog::from_bytes(&bytes).unwrap_err(),
            CatalogError::InvalidUtf8 { offset: 9 }
        );
        // ...but peeking never reads the bodies it skips.
        assert_eq!(peek_names(&bytes).unwrap(), vec!["a"]);
    }

    #[test]
    fn peek_names_still_checks_bounds() {
        // Valid name, then a body whose length overruns the buffer.
        let bytes = [1, 0, 0, 0, b'a', 99, 0, 0, 0, b'x'];
        assert_eq!(
            peek_names(&bytes).unwrap_err(),
            CatalogError::Truncated { offset: 5 }
        );
    }
}
