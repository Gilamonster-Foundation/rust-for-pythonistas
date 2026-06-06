//! # Chapter 7 Exercises: PyO3 House Style
//!
//! These exercises build the boundary disciplines for a small settings
//! store that will one day grow a Python face. Notice what's NOT here:
//! pyo3. Every house-style decision — the error enum, the exception
//! mapping, the zero-copy signatures, the Pythonic surface — is a
//! pure-Rust design problem, and the exercises build on each other.
//!
//! Run tests: `cargo test -p ch07-exercises`

// These allows are intentional: exercise stubs have unused parameters
// and fields until the student fills in the todo!() markers.
#![allow(unused_variables, dead_code)]

use std::fmt;

// ============================================================
// Exercise 1: From Stringly-Typed to a Designed Error Enum
// ============================================================
//
// Python version — typed exceptions with useful messages:
// ```python
// class SettingsError(Exception): ...
//
// class KeyMissing(SettingsError):
//     def __init__(self, key):
//         super().__init__(f"no such key: {key}")
//
// class KeyExists(SettingsError):
//     def __init__(self, key):
//         super().__init__(f"key already set: {key}")
//
// class InvalidValue(SettingsError):
//     def __init__(self, key, reason):
//         super().__init__(f"invalid value for {key}: {reason}")
// ```
//
// The enum is defined for you — your job is the Display impl, which is
// the message every user (Rust OR Python) will eventually read.
// Match the formats exactly:
//
//   KeyMissing("retries")                      -> "no such key: retries"
//   KeyExists("retries")                       -> "key already set: retries"
//   InvalidValue { key: "retries",
//                  reason: "empty value" }     -> "invalid value for retries: empty value"

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsError {
    KeyMissing(String),
    KeyExists(String),
    InvalidValue { key: String, reason: String },
}

impl fmt::Display for SettingsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!("Match each variant and write! the message formats shown above")
    }
}

impl std::error::Error for SettingsError {}

// ============================================================
// Exercise 2: The Exception-Mapping Layer
// ============================================================
//
// Python version (what the boundary will eventually do):
// ```python
// # A failed lookup is a KeyError; bad input is a ValueError.
// # That's what Python users expect from dict-like objects.
// ```
//
// Write the single function that decides which Python exception each
// error becomes:
//
//   KeyMissing      -> KeyError    (a failed lookup)
//   KeyExists       -> ValueError  (bad input)
//   InvalidValue    -> ValueError  (bad input)
//
// House style: NO catch-all `_ =>` arm. Match every variant explicitly,
// so adding a new error variant later forces a conscious mapping
// decision instead of silently defaulting.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExceptionKind {
    KeyError,
    ValueError,
}

pub fn exception_kind(err: &SettingsError) -> ExceptionKind {
    todo!("Exhaustively match err and return the right ExceptionKind")
}

// ============================================================
// Exercise 3: Borrow, Don't Clone
// ============================================================
//
// Python version — every slice is a copy, and nobody thinks about it:
// ```python
// def first_word(s: str) -> str:
//     return s.split(" ")[0]        # allocates a new str
//
// def payload(frame: bytes) -> bytes | None:
//     if frame[:2] == b"\xc7\x07":
//         return frame[2:]          # copies the rest of the buffer
//     return None
// ```
//
// In Rust, return *views* instead. `&str` and `&[u8]` are a pointer and
// a length into memory the caller already owns — no allocation, and the
// borrow checker guarantees the view can't outlive the buffer.
//
// 1. `first_word`: return the text before the first space (or the whole
//    string if there's no space) — as a slice of the input. The tests
//    check the POINTER, not just the contents: a `.to_string()` answer
//    will have the right text and still fail.
// 2. `payload`: if `frame` starts with the magic bytes [0xC7, 0x07],
//    return everything after them as a borrowed slice; otherwise None.

pub fn first_word(s: &str) -> &str {
    todo!("Return a slice of s — try s.find(' ') or s.split(' ').next()")
}

pub fn payload(frame: &[u8]) -> Option<&[u8]> {
    todo!("Check for the [0xC7, 0x07] prefix, then slice past it")
}

// ============================================================
// Exercise 4: A Core Type with a Pythonic Surface
// ============================================================
//
// Python version:
// ```python
// class Settings:
//     """Write-once settings: keys can be set exactly once."""
//     def __init__(self):
//         self._entries = {}
//
//     def set(self, key, value):
//         if not value:
//             raise InvalidValue(key, "empty value")
//         if key in self._entries:
//             raise KeyExists(key)
//         self._entries[key] = value
//
//     def get(self, key):
//         return self._entries.get(key)      # None if missing
//
//     def require(self, key):
//         if key not in self._entries:
//             raise KeyMissing(key)
//         return self._entries[key]
//
//     def __repr__(self):
//         return f"Settings(keys={len(self._entries)})"
// ```
//
// Implement `set`, `get`, `require`, and Display. Use the SettingsError
// variants from Exercise 1 — note that `get` returns Option (the Rust
// face) while `require` returns Result (the shape the boundary layer
// will turn into a KeyError). A core designed for two ecosystems offers
// both.

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Settings {
    entries: Vec<(String, String)>,
}

impl Settings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Set a key for the first time.
    ///
    /// - Empty `value` -> InvalidValue with reason "empty value"
    /// - Key already present -> KeyExists
    pub fn set(&mut self, key: &str, value: &str) -> Result<(), SettingsError> {
        todo!("Validate value, check for the key, then push (key, value)")
    }

    /// Look up a value, Rust-style: Option, borrowed.
    pub fn get(&self, key: &str) -> Option<&str> {
        todo!("Find the entry and return the value as &str")
    }

    /// Look up a value, boundary-style: a missing key is an error.
    pub fn require(&self, key: &str) -> Result<&str, SettingsError> {
        todo!("Build on get() — map None to KeyMissing")
    }
}

/// Display doubles as Python's __repr__: "Settings(keys=N)"
impl fmt::Display for Settings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!("Write Settings(keys=N) using self.len()")
    }
}

// ============================================================
// Exercise 5: The Boundary Function
// ============================================================
//
// In the real binding layer, `impl From<SettingsError> for PyErr` turns
// every core error into a typed Python exception, and each binding is a
// one-line delegation. Model that here without pyo3: a "boundary error"
// is the pair (which exception, what message).
//
// What the real thing looks like (don't write this — read it):
// ```text
// impl From<SettingsError> for PyErr {
//     fn from(err: SettingsError) -> PyErr {
//         let message = err.to_string();
//         match exception_kind(&err) {
//             ExceptionKind::KeyError => PyKeyError::new_err(message),
//             ExceptionKind::ValueError => PyValueError::new_err(message),
//         }
//     }
// }
// ```
//
// 1. `to_boundary_error`: compose Exercise 1 (the message) with
//    Exercise 2 (the kind).
// 2. `boundary_require`: delegate to Settings::require, map the error,
//    and return an OWNED String — the borrowed &str must be copied to
//    cross the boundary, because a Python str owns its memory.

pub fn to_boundary_error(err: SettingsError) -> (ExceptionKind, String) {
    todo!("Pair exception_kind(&err) with err.to_string()")
}

pub fn boundary_require(settings: &Settings, key: &str) -> Result<String, (ExceptionKind, String)> {
    todo!("require + map_err(to_boundary_error) + an owned copy of the value")
}

// ============================================================
// Tests — do not modify below this line
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Exercise 1
    #[test]
    fn ex1_key_missing_message() {
        let err = SettingsError::KeyMissing("retries".to_string());
        assert_eq!(err.to_string(), "no such key: retries");
    }

    #[test]
    fn ex1_key_exists_message() {
        let err = SettingsError::KeyExists("retries".to_string());
        assert_eq!(err.to_string(), "key already set: retries");
    }

    #[test]
    fn ex1_invalid_value_message() {
        let err = SettingsError::InvalidValue {
            key: "retries".to_string(),
            reason: "empty value".to_string(),
        };
        assert_eq!(err.to_string(), "invalid value for retries: empty value");
    }

    // Exercise 2
    #[test]
    fn ex2_missing_key_is_key_error() {
        let err = SettingsError::KeyMissing("x".to_string());
        assert_eq!(exception_kind(&err), ExceptionKind::KeyError);
    }

    #[test]
    fn ex2_bad_input_is_value_error() {
        assert_eq!(
            exception_kind(&SettingsError::KeyExists("x".to_string())),
            ExceptionKind::ValueError
        );
        assert_eq!(
            exception_kind(&SettingsError::InvalidValue {
                key: "x".to_string(),
                reason: "empty value".to_string(),
            }),
            ExceptionKind::ValueError
        );
    }

    // Exercise 3
    #[test]
    fn ex3_first_word_before_space() {
        assert_eq!(first_word("hello world"), "hello");
    }

    #[test]
    fn ex3_first_word_whole_string() {
        assert_eq!(first_word("single"), "single");
    }

    #[test]
    fn ex3_first_word_is_zero_copy() {
        let input = "borrow me";
        // Same pointer as the input — proof it's a view, not a copy.
        assert_eq!(first_word(input).as_ptr(), input.as_ptr());
    }

    #[test]
    fn ex3_payload_with_magic() {
        let frame = [0xC7, 0x07, 1, 2, 3];
        assert_eq!(payload(&frame), Some(&frame[2..]));
    }

    #[test]
    fn ex3_payload_is_zero_copy() {
        let frame = [0xC7, 0x07, 9, 9];
        let p = payload(&frame).unwrap();
        assert_eq!(p.as_ptr(), frame[2..].as_ptr());
    }

    #[test]
    fn ex3_payload_without_magic() {
        assert_eq!(payload(&[0x00, 0x07, 1]), None);
        assert_eq!(payload(&[0xC7]), None);
        assert_eq!(payload(&[]), None);
    }

    // Exercise 4
    #[test]
    fn ex4_set_and_get() {
        let mut settings = Settings::new();
        settings.set("retries", "3").unwrap();
        assert_eq!(settings.get("retries"), Some("3"));
        assert_eq!(settings.get("missing"), None);
    }

    #[test]
    fn ex4_set_rejects_empty_value() {
        let mut settings = Settings::new();
        assert_eq!(
            settings.set("retries", ""),
            Err(SettingsError::InvalidValue {
                key: "retries".to_string(),
                reason: "empty value".to_string(),
            })
        );
        assert!(settings.is_empty());
    }

    #[test]
    fn ex4_set_rejects_duplicate_key() {
        let mut settings = Settings::new();
        settings.set("retries", "3").unwrap();
        assert_eq!(
            settings.set("retries", "5"),
            Err(SettingsError::KeyExists("retries".to_string()))
        );
        // The original value is untouched.
        assert_eq!(settings.get("retries"), Some("3"));
    }

    #[test]
    fn ex4_require_found_and_missing() {
        let mut settings = Settings::new();
        settings.set("retries", "3").unwrap();
        assert_eq!(settings.require("retries"), Ok("3"));
        assert_eq!(
            settings.require("missing"),
            Err(SettingsError::KeyMissing("missing".to_string()))
        );
    }

    #[test]
    fn ex4_display_is_the_repr() {
        let mut settings = Settings::new();
        settings.set("a", "1").unwrap();
        settings.set("b", "2").unwrap();
        assert_eq!(settings.to_string(), "Settings(keys=2)");
    }

    // Exercise 5
    #[test]
    fn ex5_boundary_error_pairs_kind_and_message() {
        let err = SettingsError::KeyMissing("retries".to_string());
        assert_eq!(
            to_boundary_error(err),
            (ExceptionKind::KeyError, "no such key: retries".to_string())
        );
    }

    #[test]
    fn ex5_boundary_require_success_is_owned() {
        let mut settings = Settings::new();
        settings.set("retries", "3").unwrap();
        let value: String = boundary_require(&settings, "retries").unwrap();
        assert_eq!(value, "3");
    }

    #[test]
    fn ex5_boundary_require_missing_is_key_error() {
        let settings = Settings::new();
        assert_eq!(
            boundary_require(&settings, "missing"),
            Err((ExceptionKind::KeyError, "no such key: missing".to_string()))
        );
    }
}
