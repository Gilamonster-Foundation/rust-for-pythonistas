//! # Chapter 0: Getting Started
//!
//! Welcome! This module covers the absolute basics: variables, functions,
//! control flow, and collections. If you've written Python, you already
//! understand the *concepts* — this is just the new syntax.
//!
//! Run the tests: `cargo test -p ch00-getting-started`

// ---------------------------------------------------------------------------
// 1. Variables and types
// ---------------------------------------------------------------------------

/// In Python you'd write: `def add(a: int, b: int) -> int: return a + b`
///
/// In Rust, types in function signatures are mandatory (not just hints).
/// The compiler uses them to catch bugs before your code runs.
pub fn add(a: i32, b: i32) -> i32 {
    a + b // no `return` needed — the last expression is the return value
}

/// Demonstrate that variables are immutable by default.
///
/// Python equivalent:
/// ```python
/// def counting_demo():
///     count = 0
///     count += 1
///     count += 1
///     return count  # 2
/// ```
pub fn counting_demo() -> i32 {
    let mut count = 0; // `mut` makes it mutable — without this, count += 1 won't compile
    count += 1;
    count += 1;
    count
}

// ---------------------------------------------------------------------------
// 2. String formatting
// ---------------------------------------------------------------------------

/// Python: `f"Hello, {name}!"`
/// Rust:   `format!("Hello, {name}!")`
///
/// The `!` means `format!` is a macro. The compiler checks the format
/// string at compile time — mismatched args are a compile error, not
/// a runtime crash.
pub fn greet(name: &str) -> String {
    format!("Hello, {name}!")
}

/// Python: `f"Pi is approximately {pi:.2f}"`
/// Rust:   `format!("Pi is approximately {pi:.2}")`
pub fn format_pi() -> String {
    let pi = std::f64::consts::PI;
    format!("Pi is approximately {pi:.2}")
}

// ---------------------------------------------------------------------------
// 3. Control flow — if/else as an expression
// ---------------------------------------------------------------------------

/// In Python, if/else is a statement. In Rust, it's an expression
/// that returns a value.
///
/// Python equivalent:
/// ```python
/// def letter_grade(score):
///     if score >= 90:
///         return "A"
///     elif score >= 80:
///         return "B"
///     elif score >= 70:
///         return "C"
///     else:
///         return "F"
/// ```
pub fn letter_grade(score: u32) -> &'static str {
    if score >= 90 {
        "A"
    } else if score >= 80 {
        "B"
    } else if score >= 70 {
        "C"
    } else {
        "F"
    }
}

// ---------------------------------------------------------------------------
// 4. Collections — Vec and HashMap
// ---------------------------------------------------------------------------

/// Python: `[x**2 for x in range(1, n+1)]`
///
/// Rust has iterators and `collect()` — the equivalent of list
/// comprehensions, but more composable.
pub fn squares(n: u32) -> Vec<u32> {
    (1..=n).map(|x| x * x).collect()
}

/// Python: `sum(items)`
///
/// Rust iterators have `.sum()` built in.
pub fn sum_items(items: &[i32]) -> i32 {
    items.iter().sum()
}

/// Python: `[x for x in items if x > threshold]`
///
/// Rust: `.filter()` + `.collect()`
pub fn filter_above(items: &[i32], threshold: i32) -> Vec<i32> {
    items.iter().copied().filter(|&x| x > threshold).collect()
}

/// Count word frequencies — like Python's `collections.Counter`.
///
/// Python equivalent:
/// ```python
/// from collections import Counter
/// def word_frequencies(text):
///     return dict(Counter(text.lower().split()))
/// ```
pub fn word_frequencies(text: &str) -> std::collections::HashMap<String, usize> {
    let mut counts = std::collections::HashMap::new();
    for word in text.split_whitespace() {
        let lower = word.to_lowercase();
        *counts.entry(lower).or_insert(0) += 1;
    }
    counts
}

// ---------------------------------------------------------------------------
// 5. Iterators — Rust's superpower
// ---------------------------------------------------------------------------

/// Python: `", ".join(items)`
/// Rust: `items.join(", ")`
///
/// Straightforward — but Rust's version works on slices of &str, not
/// arbitrary iterables. For more complex joins, use iterators.
pub fn join_words(words: &[&str]) -> String {
    words.join(", ")
}

/// Python: `list(zip(keys, values))`
///
/// Rust has `.zip()` on iterators.
pub fn zip_to_pairs(keys: &[&str], values: &[i32]) -> Vec<(String, i32)> {
    keys.iter()
        .zip(values.iter())
        .map(|(k, v)| (k.to_string(), *v))
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
        assert_eq!(add(-1, 1), 0);
    }

    #[test]
    fn test_counting() {
        assert_eq!(counting_demo(), 2);
    }

    #[test]
    fn test_greet() {
        assert_eq!(greet("World"), "Hello, World!");
        assert_eq!(greet("Rustacean"), "Hello, Rustacean!");
    }

    #[test]
    fn test_format_pi() {
        assert_eq!(format_pi(), "Pi is approximately 3.14");
    }

    #[test]
    fn test_letter_grade() {
        assert_eq!(letter_grade(95), "A");
        assert_eq!(letter_grade(85), "B");
        assert_eq!(letter_grade(75), "C");
        assert_eq!(letter_grade(50), "F");
        assert_eq!(letter_grade(90), "A"); // boundary
    }

    #[test]
    fn test_squares() {
        assert_eq!(squares(5), vec![1, 4, 9, 16, 25]);
        assert_eq!(squares(0), vec![]);
    }

    #[test]
    fn test_sum() {
        assert_eq!(sum_items(&[1, 2, 3, 4]), 10);
        assert_eq!(sum_items(&[]), 0);
    }

    #[test]
    fn test_filter() {
        assert_eq!(filter_above(&[1, 5, 3, 8, 2], 3), vec![5, 8]);
    }

    #[test]
    fn test_word_frequencies() {
        let counts = word_frequencies("the cat sat on the mat");
        assert_eq!(counts["the"], 2);
        assert_eq!(counts["cat"], 1);
        assert_eq!(counts["mat"], 1);
    }

    #[test]
    fn test_join_words() {
        assert_eq!(join_words(&["one", "two", "three"]), "one, two, three");
    }

    #[test]
    fn test_zip() {
        let pairs = zip_to_pairs(&["a", "b"], &[1, 2]);
        assert_eq!(pairs, vec![("a".to_string(), 1), ("b".to_string(), 2)]);
    }
}
