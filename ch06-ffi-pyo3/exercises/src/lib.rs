//! # Chapter 6 Exercises: FFI & PyO3
//!
//! These exercises practice the shapes that make Rust code bindable:
//! thin C-ABI boundaries, conversion-friendly signatures, and error
//! types that map cleanly to Python exceptions. Everything is pure Rust
//! — no Python interpreter needed to solve them.
//!
//! Run tests: `cargo test -p ch06-exercises`

// These allows are intentional: exercise stubs have unused parameters
// and fields until the student fills in the todo!() markers.
#![allow(unused_variables, dead_code)]

use std::collections::HashMap;
use std::fmt;

// ============================================================
// Exercise 1: Keep the Unsafe Boundary Thin
// ============================================================
//
// The golden rule of FFI: logic lives in safe Rust; the extern wrapper
// only translates. The C-ABI wrapper below is already written — it's
// one line, and it stays one line. Your job is the safe core it calls.
//
// Python version (what a ctypes caller would eventually reach):
// ```python
// def scale_and_clamp(value, factor, max_value):
//     """Scale value by factor, clamping the result to [0, max_value]."""
//     return min(max(value * factor, 0.0), max_value)
// ```
//
// Implement `scale_and_clamp`. The tests exercise the safe function;
// the extern wrapper compiles against your implementation.

pub fn scale_and_clamp(value: f64, factor: f64, max_value: f64) -> f64 {
    todo!("Multiply value by factor, then clamp the result between 0.0 and max_value")
}

/// The C-ABI boundary — provided for you. Notice how little it does:
/// no logic, just a calling-convention change. This is the shape PyO3
/// generates for you behind `#[pyfunction]`.
#[no_mangle]
pub extern "C" fn ffi_scale_and_clamp(value: f64, factor: f64, max_value: f64) -> f64 {
    scale_and_clamp(value, factor, max_value)
}

// ============================================================
// Exercise 2: list[tuple] -> dict (the FromPyObject shape)
// ============================================================
//
// When a Python caller passes `list[tuple[str, int]]` to a #[pyfunction]
// that takes `Vec<(String, i64)>`, PyO3 converts element by element.
// This exercise is that conversion's classic consumer: Python's
// dict() constructor.
//
// Python version:
// ```python
// def from_pairs(pairs: list[tuple[str, int]]) -> dict[str, int]:
//     return dict(pairs)  # later duplicates win
// ```
//
// Build a HashMap from the pairs. If a key appears more than once,
// the LAST occurrence wins — exactly like dict(pairs) in Python.

pub fn from_pairs(pairs: Vec<(String, i64)>) -> HashMap<String, i64> {
    todo!("Insert each pair into a HashMap; later duplicates overwrite earlier ones")
}

// ============================================================
// Exercise 3: dict.get — None Becomes Option
// ============================================================
//
// Python's dict.get returns None for missing keys; Rust's HashMap::get
// returns Option. PyO3 maps between them automatically: a #[pyfunction]
// returning Option<i64> gives Python `int | None`.
//
// Python version:
// ```python
// def lookup(table: dict[str, int], key: str) -> int | None:
//     return table.get(key)
//
// def lookup_or(table: dict[str, int], key: str, default: int) -> int:
//     return table.get(key, default)
// ```
//
// Implement both. Hint: `HashMap::get` returns `Option<&i64>` — you'll
// need `.copied()` to turn it into `Option<i64>`.

pub fn lookup(table: &HashMap<String, i64>, key: &str) -> Option<i64> {
    todo!("Return Some(value) if key exists, None otherwise")
}

pub fn lookup_or(table: &HashMap<String, i64>, key: &str, default: i64) -> i64 {
    todo!("Return the value for key, or default if missing — try unwrap_or")
}

// ============================================================
// Exercise 4: An Error Type That Maps to ValueError
// ============================================================
//
// A binding layer converts Rust errors into Python exceptions, and the
// Display string becomes the exception message — it's what Python users
// see in their traceback. Write messages worthy of that.
//
// Python version:
// ```python
// def parse_percent(text: str) -> float:
//     text = text.strip()
//     if not text:
//         raise ValueError("empty input: expected a percentage")
//     if not text.endswith("%"):
//         raise ValueError(f"missing % sign: '{text}'")
//     try:
//         return float(text[:-1]) / 100.0
//     except ValueError:
//         raise ValueError(f"not a number: '{text[:-1]}'")
// ```
//
// 1. Implement Display for PercentError (messages must match the tests)
// 2. Implement parse_percent: "85%" -> 0.85, "12.5%" -> 0.125

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PercentError {
    /// Input was empty or only whitespace.
    Empty,
    /// Input didn't end with '%'; carries the trimmed input.
    MissingSign(String),
    /// The part before '%' wasn't a number; carries that part.
    BadNumber(String),
}

impl fmt::Display for PercentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!("Match each variant — see the Python messages above and the tests below")
    }
}

impl std::error::Error for PercentError {}

pub fn parse_percent(text: &str) -> Result<f64, PercentError> {
    todo!("Trim, check for emptiness, require a trailing '%', parse, divide by 100")
}

// ============================================================
// Exercise 5: A Struct Shaped Like a #[pyclass]
// ============================================================
//
// A #[pyclass] is just a Rust struct whose methods make sense from
// Python: a constructor, mutating methods, and queries. Build the
// struct; in a real binding crate you'd add #[pyclass]/#[pymethods]
// and it would ship to Python unchanged.
//
// Python version:
// ```python
// class WordTally:
//     def __init__(self):
//         self._counts = {}
//
//     def add(self, word: str) -> None:
//         w = word.lower()
//         self._counts[w] = self._counts.get(w, 0) + 1
//
//     def count(self, word: str) -> int:
//         return self._counts.get(word.lower(), 0)
//
//     def total(self) -> int:
//         return sum(self._counts.values())
//
//     def most_common(self, n: int) -> list[tuple[str, int]]:
//         ranked = sorted(self._counts.items(), key=lambda kv: (-kv[1], kv[0]))
//         return ranked[:n]
// ```
//
// Implement the four methods. most_common sorts by count (descending),
// breaking ties alphabetically — deterministic output, always.

#[derive(Debug, Default)]
pub struct WordTally {
    counts: HashMap<String, usize>,
}

impl WordTally {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record one occurrence of a word (case-insensitive).
    pub fn add(&mut self, word: &str) {
        todo!("Lowercase the word and increment its count")
    }

    /// How many times has this word been added (case-insensitive)?
    pub fn count(&self, word: &str) -> usize {
        todo!("Look up the lowercased word; missing words count as 0")
    }

    /// Total number of words added.
    pub fn total(&self) -> usize {
        todo!("Sum all the counts")
    }

    /// The n most frequent words: count descending, ties alphabetical.
    pub fn most_common(&self, n: usize) -> Vec<(String, usize)> {
        todo!("Collect into a Vec, sort by (count desc, word asc), truncate to n")
    }
}

// ============================================================
// Tests — do not modify below this line
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Exercise 1
    #[test]
    fn ex1_scales() {
        assert!((scale_and_clamp(2.0, 3.0, 100.0) - 6.0).abs() < f64::EPSILON);
    }

    #[test]
    fn ex1_clamps_high() {
        assert!((scale_and_clamp(50.0, 3.0, 100.0) - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn ex1_clamps_low() {
        assert!(scale_and_clamp(-5.0, 2.0, 100.0).abs() < f64::EPSILON);
    }

    // Exercise 2
    #[test]
    fn ex2_builds_map() {
        let pairs = vec![("a".to_string(), 1), ("b".to_string(), 2)];
        let map = from_pairs(pairs);
        assert_eq!(map.get("a"), Some(&1));
        assert_eq!(map.get("b"), Some(&2));
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn ex2_last_duplicate_wins() {
        let pairs = vec![("k".to_string(), 1), ("k".to_string(), 99)];
        let map = from_pairs(pairs);
        assert_eq!(map.get("k"), Some(&99));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn ex2_empty_pairs() {
        assert!(from_pairs(vec![]).is_empty());
    }

    // Exercise 3
    #[test]
    fn ex3_lookup_present() {
        let mut table = HashMap::new();
        table.insert("answer".to_string(), 42);
        assert_eq!(lookup(&table, "answer"), Some(42));
    }

    #[test]
    fn ex3_lookup_missing_is_none() {
        let table = HashMap::new();
        assert_eq!(lookup(&table, "missing"), None);
    }

    #[test]
    fn ex3_lookup_or_default() {
        let mut table = HashMap::new();
        table.insert("answer".to_string(), 42);
        assert_eq!(lookup_or(&table, "answer", 0), 42);
        assert_eq!(lookup_or(&table, "missing", -1), -1);
    }

    // Exercise 4
    #[test]
    fn ex4_parses_whole_percent() {
        let value = parse_percent("85%").unwrap();
        assert!((value - 0.85).abs() < 1e-12);
    }

    #[test]
    fn ex4_parses_fractional_percent() {
        let value = parse_percent("12.5%").unwrap();
        assert!((value - 0.125).abs() < 1e-12);
    }

    #[test]
    fn ex4_trims_whitespace() {
        let value = parse_percent("  100%  ").unwrap();
        assert!((value - 1.0).abs() < 1e-12);
    }

    #[test]
    fn ex4_empty_is_error() {
        assert_eq!(parse_percent("   "), Err(PercentError::Empty));
    }

    #[test]
    fn ex4_missing_sign_is_error() {
        assert_eq!(
            parse_percent("85"),
            Err(PercentError::MissingSign("85".to_string()))
        );
    }

    #[test]
    fn ex4_bad_number_is_error() {
        assert_eq!(
            parse_percent("abc%"),
            Err(PercentError::BadNumber("abc".to_string()))
        );
    }

    #[test]
    fn ex4_messages_match_python_exceptions() {
        // These are the strings Python users would see as str(exc).
        assert_eq!(
            PercentError::Empty.to_string(),
            "empty input: expected a percentage"
        );
        assert_eq!(
            PercentError::MissingSign("85".to_string()).to_string(),
            "missing % sign: '85'"
        );
        assert_eq!(
            PercentError::BadNumber("abc".to_string()).to_string(),
            "not a number: 'abc'"
        );
    }

    // Exercise 5
    #[test]
    fn ex5_add_and_count() {
        let mut tally = WordTally::new();
        tally.add("rust");
        tally.add("Rust");
        tally.add("python");
        assert_eq!(tally.count("RUST"), 2);
        assert_eq!(tally.count("python"), 1);
        assert_eq!(tally.count("missing"), 0);
    }

    #[test]
    fn ex5_total() {
        let mut tally = WordTally::new();
        tally.add("a");
        tally.add("b");
        tally.add("a");
        assert_eq!(tally.total(), 3);
    }

    #[test]
    fn ex5_most_common_ranks_by_count() {
        let mut tally = WordTally::new();
        for word in ["apple", "banana", "apple", "cherry", "apple", "banana"] {
            tally.add(word);
        }
        assert_eq!(
            tally.most_common(2),
            vec![("apple".to_string(), 3), ("banana".to_string(), 2)]
        );
    }

    #[test]
    fn ex5_most_common_breaks_ties_alphabetically() {
        let mut tally = WordTally::new();
        for word in ["beta", "alpha", "beta", "alpha"] {
            tally.add(word);
        }
        assert_eq!(
            tally.most_common(2),
            vec![("alpha".to_string(), 2), ("beta".to_string(), 2)]
        );
    }

    #[test]
    fn ex5_empty_tally() {
        let tally = WordTally::new();
        assert_eq!(tally.total(), 0);
        assert!(tally.most_common(5).is_empty());
    }
}
