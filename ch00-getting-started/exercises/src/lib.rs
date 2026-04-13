//! # Chapter 0 Exercises: Getting Started
//!
//! These exercises are deliberately simple — they're here to make sure
//! your toolchain works and you're comfortable with the basic syntax.
//! If you can do these, you're ready for Chapter 1.
//!
//! Run tests: `cargo test -p ch00-exercises`

#![allow(unused_variables, dead_code)]

// ============================================================
// Exercise 1: A Simple Function
// ============================================================
//
// Python version:
// ```python
// def multiply(a, b):
//     return a * b
//
// assert multiply(3, 4) == 12
// assert multiply(-2, 5) == -10
// ```
//
// Write the Rust equivalent.

pub fn multiply(a: i32, b: i32) -> i32 {
    todo!("Return a * b")
}

// ============================================================
// Exercise 2: String Formatting
// ============================================================
//
// Python version:
// ```python
// def describe(name, age):
//     return f"{name} is {age} years old"
//
// assert describe("Alice", 30) == "Alice is 30 years old"
// ```
//
// Use format!() — Rust's equivalent of f-strings.

pub fn describe(name: &str, age: u32) -> String {
    todo!("Return \"{name} is {age} years old\" using format!()")
}

// ============================================================
// Exercise 3: Conditional Logic
// ============================================================
//
// Python version:
// ```python
// def fizzbuzz(n):
//     if n % 15 == 0:
//         return "FizzBuzz"
//     elif n % 3 == 0:
//         return "Fizz"
//     elif n % 5 == 0:
//         return "Buzz"
//     else:
//         return str(n)
// ```
//
// Write the Rust version. Remember: if/else is an expression.

pub fn fizzbuzz(n: u32) -> String {
    todo!("Return FizzBuzz, Fizz, Buzz, or the number as a string")
}

// ============================================================
// Exercise 4: Working with Vec
// ============================================================
//
// Python version:
// ```python
// def double_all(numbers):
//     return [n * 2 for n in numbers]
//
// assert double_all([1, 2, 3]) == [2, 4, 6]
// ```
//
// Use .iter(), .map(), and .collect() — Rust's version of list
// comprehensions.

pub fn double_all(numbers: &[i32]) -> Vec<i32> {
    todo!("Return a new Vec with each number doubled")
}

// ============================================================
// Exercise 5: Putting It Together
// ============================================================
//
// Python version:
// ```python
// def summarize_scores(scores):
//     """Given a list of scores, return a summary string.
//
//     >>> summarize_scores([85, 92, 78, 95, 88])
//     '5 scores, min 78, max 95, avg 87.6'
//     """
//     n = len(scores)
//     if n == 0:
//         return "no scores"
//     lo = min(scores)
//     hi = max(scores)
//     avg = sum(scores) / n
//     return f"{n} scores, min {lo}, max {hi}, avg {avg:.1}"
// ```
//
// Write the Rust version. You'll need:
// - .len() for count
// - .iter().min() and .iter().max() (these return Option!)
// - .iter().sum::<i32>() for sum
// - format!("{:.1}", value) for one decimal place

pub fn summarize_scores(scores: &[i32]) -> String {
    todo!("Return summary string, or \"no scores\" if empty")
}

// ============================================================
// Tests — do not modify below this line
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Exercise 1
    #[test]
    fn ex1_multiply() {
        assert_eq!(multiply(3, 4), 12);
        assert_eq!(multiply(-2, 5), -10);
        assert_eq!(multiply(0, 100), 0);
    }

    // Exercise 2
    #[test]
    fn ex2_describe() {
        assert_eq!(describe("Alice", 30), "Alice is 30 years old");
        assert_eq!(describe("Bob", 0), "Bob is 0 years old");
    }

    // Exercise 3
    #[test]
    fn ex3_fizzbuzz() {
        assert_eq!(fizzbuzz(1), "1");
        assert_eq!(fizzbuzz(3), "Fizz");
        assert_eq!(fizzbuzz(5), "Buzz");
        assert_eq!(fizzbuzz(15), "FizzBuzz");
        assert_eq!(fizzbuzz(30), "FizzBuzz");
        assert_eq!(fizzbuzz(7), "7");
    }

    // Exercise 4
    #[test]
    fn ex4_double_all() {
        assert_eq!(double_all(&[1, 2, 3]), vec![2, 4, 6]);
        assert_eq!(double_all(&[]), Vec::<i32>::new());
        assert_eq!(double_all(&[-1, 0, 1]), vec![-2, 0, 2]);
    }

    // Exercise 5
    #[test]
    fn ex5_summarize() {
        assert_eq!(
            summarize_scores(&[85, 92, 78, 95, 88]),
            "5 scores, min 78, max 95, avg 87.6"
        );
    }

    #[test]
    fn ex5_empty() {
        assert_eq!(summarize_scores(&[]), "no scores");
    }

    #[test]
    fn ex5_single() {
        assert_eq!(
            summarize_scores(&[100]),
            "1 scores, min 100, max 100, avg 100.0"
        );
    }
}
