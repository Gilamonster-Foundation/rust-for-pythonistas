//! # Chapter 1 Exercises: Ownership
//!
//! Each exercise shows a Python snippet and asks you to write the Rust
//! equivalent. Replace the `todo!()` markers with working code.
//!
//! Run tests: `cargo test -p ch01-exercises`
//!
//! Hint: if you're stuck, look at the test to see what the function should
//! return, then check back at ch01-ownership/src/lib.rs for the patterns.

// These allows are intentional: exercise stubs have unused parameters
// and fields until the student fills in the todo!() markers.
#![allow(unused_variables, dead_code, clippy::ptr_arg)]

// ============================================================
// Exercise 1: Transfer and Return
// ============================================================
//
// Python version:
// ```python
// def transfer_and_extend(items):
//     items.append("extra")
//     return items
//
// original = ["a", "b"]
// result = transfer_and_extend(original)
// # original is "consumed" — pretend it's gone
// assert result == ["a", "b", "extra"]
// ```
//
// In Rust, write a function that:
// 1. Takes ownership of a Vec<String>
// 2. Pushes "extra" onto it
// 3. Returns the Vec (giving ownership back to the caller)

pub fn transfer_and_extend(items: Vec<String>) -> Vec<String> {
    todo!("Take ownership, push \"extra\", return the Vec")
}

// ============================================================
// Exercise 2: Borrow to Count
// ============================================================
//
// Python version:
// ```python
// def count_long_words(words, min_length):
//     return sum(1 for w in words if len(w) >= min_length)
//
// words = ["hi", "hello", "hey", "magnificent"]
// assert count_long_words(words, 4) == 2
// # words is still usable here
// ```
//
// In Rust, write a function that:
// 1. Borrows a slice of strings (don't take ownership!)
// 2. Returns how many have length >= min_length

pub fn count_long_words(words: &[String], min_length: usize) -> usize {
    todo!("Count words with length >= min_length without taking ownership")
}

// ============================================================
// Exercise 3: Mutable Borrow
// ============================================================
//
// Python version:
// ```python
// def remove_short_words(words, min_length):
//     words[:] = [w for w in words if len(w) >= min_length]
//
// words = ["hi", "hello", "hey", "magnificent"]
// remove_short_words(words, 4)
// assert words == ["hello", "magnificent"]
// ```
//
// In Rust, write a function that:
// 1. Takes a mutable borrow of a Vec<String>
// 2. Removes all strings shorter than min_length
// Hint: Vec has a `retain` method that works like Python's filter

pub fn remove_short_words(words: &mut Vec<String>, min_length: usize) {
    todo!("Remove words shorter than min_length using &mut")
}

// ============================================================
// Exercise 4: Clone When You Need Two Copies
// ============================================================
//
// Python version:
// ```python
// def split_and_keep(items):
//     first_half = items[:len(items)//2]
//     # items is still the full list
//     return (first_half, items)
//
// a, b = split_and_keep([1, 2, 3, 4])
// assert a == [1, 2]
// assert b == [1, 2, 3, 4]
// ```
//
// In Rust, write a function that:
// 1. Takes ownership of a Vec<i32>
// 2. Creates a clone of just the first half
// 3. Returns (first_half, original_full_vec)
// Hint: you'll need .clone() or slice + .to_vec()

pub fn split_and_keep(items: Vec<i32>) -> (Vec<i32>, Vec<i32>) {
    todo!("Clone the first half, return both halves — (first_half, full)")
}

// ============================================================
// Exercise 5: The Scope Drop
// ============================================================
//
// Python version:
// ```python
// class Logger:
//     def __init__(self, entries):
//         self.entries = entries
//     def log(self, msg):
//         self.entries.append(f"log:{msg}")
//     def __del__(self):
//         self.entries.append("logger:closed")
// ```
//
// In Rust, implement the same pattern using Drop.
// The struct and constructor are provided — implement `log` and `Drop`.

pub struct Logger {
    entries: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
}

impl Logger {
    pub fn new(entries: std::sync::Arc<std::sync::Mutex<Vec<String>>>) -> Self {
        Self { entries }
    }

    pub fn log(&self, msg: &str) {
        todo!("Push \"log:{{msg}}\" onto self.entries")
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        // TODO: Push "logger:closed" onto self.entries
        // (We can't use todo!() here because panic in Drop aborts the process.
        // Replace this comment block with your implementation.)
    }
}

// ============================================================
// Tests — do not modify below this line
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn ex1_transfer_and_extend() {
        let items = vec!["a".to_string(), "b".to_string()];
        let result = transfer_and_extend(items);
        assert_eq!(result, vec!["a", "b", "extra"]);
    }

    #[test]
    fn ex2_count_long_words() {
        let words: Vec<String> = vec!["hi", "hello", "hey", "magnificent"]
            .into_iter()
            .map(String::from)
            .collect();

        assert_eq!(count_long_words(&words, 4), 2);
        // words is still valid — we only borrowed
        assert_eq!(words.len(), 4);
    }

    #[test]
    fn ex3_remove_short_words() {
        let mut words: Vec<String> = vec!["hi", "hello", "hey", "magnificent"]
            .into_iter()
            .map(String::from)
            .collect();

        remove_short_words(&mut words, 4);
        assert_eq!(words, vec!["hello", "magnificent"]);
    }

    #[test]
    fn ex4_split_and_keep() {
        let items = vec![1, 2, 3, 4];
        let (first_half, full) = split_and_keep(items);
        assert_eq!(first_half, vec![1, 2]);
        assert_eq!(full, vec![1, 2, 3, 4]);
    }

    #[test]
    fn ex5_logger_drop() {
        let entries = Arc::new(Mutex::new(Vec::new()));

        {
            let logger = Logger::new(Arc::clone(&entries));
            logger.log("starting");
            logger.log("working");
            // logger is dropped at end of scope
        }

        let entries = entries.lock().unwrap();
        assert_eq!(entries[0], "log:starting");
        assert_eq!(entries[1], "log:working");
        assert_eq!(entries[2], "logger:closed");
    }
}
