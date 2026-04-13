//! # Chapter 2 Exercises: Error Handling
//!
//! Each exercise shows a Python snippet and asks you to write the Rust
//! equivalent. Replace the `todo!()` markers with working code.
//!
//! Run tests: `cargo test -p ch02-exercises`

// These allows are intentional: exercise stubs have unused parameters
// and fields until the student fills in the todo!() markers.
#![allow(unused_variables, dead_code, clippy::ptr_arg)]

use std::fmt;

// ============================================================
// Exercise 1: Option Basics
// ============================================================
//
// Python version:
// ```python
// EXTENSIONS = {
//     "rs": "Rust",
//     "py": "Python",
//     "js": "JavaScript",
//     "ts": "TypeScript",
// }
//
// def language_for_extension(ext):
//     return EXTENSIONS.get(ext)
//
// assert language_for_extension("rs") == "Rust"
// assert language_for_extension("go") is None
// ```
//
// Implement a function that returns the language name for a file extension,
// or None if the extension is unknown. Use a match expression.

pub fn language_for_extension(ext: &str) -> Option<&'static str> {
    todo!("Match on ext: rs->Rust, py->Python, js->JavaScript, ts->TypeScript, _->None")
}

// ============================================================
// Exercise 2: Option Chaining
// ============================================================
//
// Python version:
// ```python
// def describe_extension(ext):
//     lang = language_for_extension(ext)
//     if lang is not None:
//         return f"{ext} is a {lang} file"
//     return None
//
// assert describe_extension("py") == "py is a Python file"
// assert describe_extension("go") is None
// ```
//
// Use Option::map to transform the value without unwrapping.

pub fn describe_extension(ext: &str) -> Option<String> {
    todo!("Use language_for_extension and .map() to build the description string")
}

// ============================================================
// Exercise 3: Custom Error Type
// ============================================================
//
// Python version:
// ```python
// class TemperatureError(Exception): pass
//
// def celsius_to_fahrenheit(celsius):
//     if celsius < -273.15:
//         raise TemperatureError(f"below absolute zero: {celsius}")
//     return celsius * 9/5 + 32
//
// assert celsius_to_fahrenheit(100) == 212.0
// assert celsius_to_fahrenheit(0) == 32.0
// # celsius_to_fahrenheit(-300) raises TemperatureError
// ```
//
// 1. Define a TemperatureError enum with a BelowAbsoluteZero variant
//    that carries the invalid value (f64).
// 2. Implement Display for it.
// 3. Implement celsius_to_fahrenheit returning Result<f64, TemperatureError>.

#[derive(Debug, PartialEq)]
pub enum TemperatureError {
    // todo!(): Add a BelowAbsoluteZero variant that holds an f64
    BelowAbsoluteZero(f64),
}

impl fmt::Display for TemperatureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!("Display: 'below absolute zero: <value>'")
    }
}

pub fn celsius_to_fahrenheit(celsius: f64) -> Result<f64, TemperatureError> {
    todo!("Return Err if below -273.15, otherwise Ok(fahrenheit)")
}

// ============================================================
// Exercise 4: The ? Operator
// ============================================================
//
// Python version:
// ```python
// def parse_pair(s):
//     """Parse 'x,y' into a tuple of floats."""
//     parts = s.split(',')
//     if len(parts) != 2:
//         raise ValueError(f"expected 'x,y', got: {s}")
//     x = float(parts[0])  # might raise ValueError
//     y = float(parts[1])  # might raise ValueError
//     return (x, y)
//
// assert parse_pair("3.5,7.2") == (3.5, 7.2)
// # parse_pair("oops") raises ValueError
// ```
//
// Implement parse_pair. Use the provided PairError type and the ? operator
// to propagate errors from split and parse.

#[derive(Debug, PartialEq)]
pub enum PairError {
    BadFormat(String),
    BadNumber(String),
}

impl fmt::Display for PairError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadFormat(s) => write!(f, "expected 'x,y', got: {s}"),
            Self::BadNumber(s) => write!(f, "not a valid number: {s}"),
        }
    }
}

pub fn parse_pair(s: &str) -> Result<(f64, f64), PairError> {
    todo!("Split on ',', check for exactly 2 parts, parse each as f64")
}

// ============================================================
// Exercise 5: Collecting Results from an Iterator
// ============================================================
//
// Python version:
// ```python
// def parse_scores(lines):
//     """Parse 'name:score' lines into a dict.
//
//     Raises ValueError on malformed lines or non-integer scores.
//     """
//     result = {}
//     for line in lines:
//         if ':' not in line:
//             raise ValueError(f"missing ':' in: {line}")
//         name, score_str = line.split(':', 1)
//         score = int(score_str)  # raises ValueError if not a number
//         result[name] = score
//     return result
//
// assert parse_scores(["alice:95", "bob:87"]) == {"alice": 95, "bob": 87}
// # parse_scores(["alice:95", "bad"]) raises ValueError
// ```
//
// Implement parse_scores using iterators and .collect() to gather
// Result<(String, i32), ScoreError> into Result<Vec<(String, i32)>, ScoreError>.

#[derive(Debug, PartialEq)]
pub enum ScoreError {
    MissingColon(String),
    InvalidScore(String),
}

impl fmt::Display for ScoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingColon(s) => write!(f, "missing ':' in: {s}"),
            Self::InvalidScore(s) => write!(f, "invalid score: {s}"),
        }
    }
}

pub fn parse_scores(lines: &[&str]) -> Result<Vec<(String, i32)>, ScoreError> {
    todo!("Iterate over lines, split each on ':', parse score, collect into Result<Vec>")
}

// ============================================================
// Tests — do not modify below this line
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Exercise 1
    #[test]
    fn ex1_known_extension() {
        assert_eq!(language_for_extension("rs"), Some("Rust"));
        assert_eq!(language_for_extension("py"), Some("Python"));
        assert_eq!(language_for_extension("js"), Some("JavaScript"));
        assert_eq!(language_for_extension("ts"), Some("TypeScript"));
    }

    #[test]
    fn ex1_unknown_extension() {
        assert_eq!(language_for_extension("go"), None);
        assert_eq!(language_for_extension(""), None);
    }

    // Exercise 2
    #[test]
    fn ex2_describe_known() {
        assert_eq!(
            describe_extension("py"),
            Some("py is a Python file".to_string())
        );
    }

    #[test]
    fn ex2_describe_unknown() {
        assert_eq!(describe_extension("go"), None);
    }

    // Exercise 3
    #[test]
    fn ex3_valid_conversion() {
        assert_eq!(celsius_to_fahrenheit(100.0), Ok(212.0));
        assert_eq!(celsius_to_fahrenheit(0.0), Ok(32.0));
        assert_eq!(celsius_to_fahrenheit(-40.0), Ok(-40.0)); // the crossover point!
    }

    #[test]
    fn ex3_below_absolute_zero() {
        assert_eq!(
            celsius_to_fahrenheit(-300.0),
            Err(TemperatureError::BelowAbsoluteZero(-300.0))
        );
    }

    #[test]
    fn ex3_exactly_absolute_zero_is_ok() {
        assert!(celsius_to_fahrenheit(-273.15).is_ok());
    }

    #[test]
    fn ex3_display() {
        let err = TemperatureError::BelowAbsoluteZero(-300.0);
        assert_eq!(err.to_string(), "below absolute zero: -300");
    }

    // Exercise 4
    #[test]
    fn ex4_valid_pair() {
        assert_eq!(parse_pair("3.5,7.2"), Ok((3.5, 7.2)));
    }

    #[test]
    fn ex4_negative_numbers() {
        assert_eq!(parse_pair("-1.5,2.5"), Ok((-1.5, 2.5)));
    }

    #[test]
    fn ex4_bad_format() {
        assert_eq!(
            parse_pair("oops"),
            Err(PairError::BadFormat("oops".to_string()))
        );
    }

    #[test]
    fn ex4_bad_number() {
        assert!(matches!(
            parse_pair("1.0,abc"),
            Err(PairError::BadNumber(_))
        ));
    }

    // Exercise 5
    #[test]
    fn ex5_valid_scores() {
        assert_eq!(
            parse_scores(&["alice:95", "bob:87"]),
            Ok(vec![("alice".to_string(), 95), ("bob".to_string(), 87),])
        );
    }

    #[test]
    fn ex5_missing_colon() {
        assert_eq!(
            parse_scores(&["alice:95", "bad"]),
            Err(ScoreError::MissingColon("bad".to_string()))
        );
    }

    #[test]
    fn ex5_invalid_score() {
        assert_eq!(
            parse_scores(&["alice:xyz"]),
            Err(ScoreError::InvalidScore("xyz".to_string()))
        );
    }
}
