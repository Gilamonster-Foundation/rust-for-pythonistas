//! # Chapter 1: Ownership
//!
//! This module demonstrates Rust's ownership system through examples that
//! map to familiar Python patterns.
//!
//! Run the tests: `cargo test -p ch01-ownership`

// ---------------------------------------------------------------------------
// 1. Move Semantics
// ---------------------------------------------------------------------------

/// In Python, when you pass a list to a function, both the caller and the
/// function can still use it (shared reference). In Rust, passing a value
/// *moves* it — the caller can no longer use it.
///
/// This function takes ownership of the Vec. After calling it, the caller's
/// variable is no longer valid.
///
/// Python equivalent (but different!):
/// ```python
/// def consume_list(items):
///     print(f"Got {len(items)} items")
///     # caller can still use their variable — Python shares references
/// ```
pub fn consume_items(items: Vec<String>) -> usize {
    items.len()
}

/// This function borrows the Vec immutably. The caller keeps ownership.
///
/// Python equivalent:
/// ```python
/// def count_items(items):
///     return len(items)
///     # caller's variable is unaffected — same in Rust with borrowing
/// ```
pub fn count_items(items: &[String]) -> usize {
    items.len()
}

// ---------------------------------------------------------------------------
// 2. Borrowing — Shared (&T) vs Exclusive (&mut T)
// ---------------------------------------------------------------------------

/// Read-only access: you can look, but you can't touch.
///
/// Python has no equivalent — everything is mutable by default.
/// The closest analogy is a function that *promises* not to modify its input,
/// but Python can't enforce that promise.
pub fn first_item(items: &[String]) -> Option<&str> {
    items.first().map(|s| s.as_str())
}

/// Read-write access: exactly one mutable reference at a time.
///
/// Python equivalent:
/// ```python
/// def add_greeting(items):
///     items.append("hello")  # mutates the caller's list
/// ```
///
/// The difference: Rust guarantees nobody else is reading `items` while
/// this function has mutable access.
pub fn add_greeting(items: &mut Vec<String>) {
    items.push(String::from("hello"));
}

// ---------------------------------------------------------------------------
// 3. RAII — Resource Acquisition Is Initialization
// ---------------------------------------------------------------------------

/// A simple resource that prints when it's created and dropped.
/// This is Rust's version of Python's context manager (`__enter__`/`__exit__`),
/// except it's automatic — you can't forget to clean up.
///
/// Python equivalent:
/// ```python
/// class ManagedResource:
///     def __init__(self, name):
///         self.name = name
///         print(f"Acquired: {name}")
///     def __del__(self):
///         print(f"Released: {self.name}")  # but __del__ timing is unreliable!
/// ```
pub struct ManagedResource {
    pub name: String,
    drop_log: Option<std::sync::Arc<std::sync::Mutex<Vec<String>>>>,
}

impl ManagedResource {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            drop_log: None,
        }
    }

    /// Create a resource that logs its drop to a shared vec (for testing).
    pub fn with_log(name: &str, log: std::sync::Arc<std::sync::Mutex<Vec<String>>>) -> Self {
        Self {
            name: name.to_string(),
            drop_log: Some(log),
        }
    }
}

impl Drop for ManagedResource {
    fn drop(&mut self) {
        if let Some(ref log) = self.drop_log {
            if let Ok(mut entries) = log.lock() {
                entries.push(format!("dropped:{}", self.name));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 4. Clone vs Copy
// ---------------------------------------------------------------------------

/// Demonstrates that Clone creates an independent copy (like deepcopy).
///
/// Python equivalent:
/// ```python
/// import copy
/// original = [1, 2, 3]
/// cloned = copy.deepcopy(original)
/// original.append(4)
/// assert cloned == [1, 2, 3]  # independent
/// ```
pub fn demonstrate_clone() -> (Vec<i32>, Vec<i32>) {
    let original = vec![1, 2, 3];
    let cloned = original.clone();
    // Both are valid and independent
    (original, cloned)
}

/// Demonstrates that Copy types don't need explicit cloning.
///
/// Python equivalent:
/// ```python
/// x = 42
/// y = x    # ints are immutable, so this "just works"
/// ```
pub fn demonstrate_copy() -> (i32, i32) {
    let x = 42;
    let y = x; // i32 implements Copy — no move, no clone needed
    (x, y)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn move_transfers_ownership() {
        let items = vec!["one".to_string(), "two".to_string()];
        let count = consume_items(items);
        assert_eq!(count, 2);
        // `items` is no longer valid here — it was moved into consume_items.
        // Try uncommenting the next line to see the compile error:
        // let _ = items.len();
    }

    #[test]
    fn borrow_preserves_ownership() {
        let items = vec!["one".to_string(), "two".to_string()];
        let count = count_items(&items);
        assert_eq!(count, 2);
        // `items` is still valid — we only lent a reference
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn shared_borrow_reads() {
        let items = vec!["alpha".to_string(), "beta".to_string()];
        assert_eq!(first_item(&items), Some("alpha"));
    }

    #[test]
    fn exclusive_borrow_mutates() {
        let mut items = vec!["one".to_string()];
        add_greeting(&mut items);
        assert_eq!(items.len(), 2);
        assert_eq!(items[1], "hello");
    }

    #[test]
    fn raii_drops_in_reverse_order() {
        let log = Arc::new(Mutex::new(Vec::new()));

        {
            let _a = ManagedResource::with_log("first", Arc::clone(&log));
            let _b = ManagedResource::with_log("second", Arc::clone(&log));
            // Both resources are alive here
        }
        // Both dropped — in reverse order (LIFO), just like Python's
        // context manager stack or C++'s destructor order.

        let entries = log.lock().unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0], "dropped:second"); // LIFO
        assert_eq!(entries[1], "dropped:first");
    }

    #[test]
    fn clone_creates_independent_copy() {
        let (original, cloned) = demonstrate_clone();
        assert_eq!(original, vec![1, 2, 3]);
        assert_eq!(cloned, vec![1, 2, 3]);
        // They're equal but independent — modifying one doesn't affect the other
    }

    #[test]
    fn copy_types_dont_move() {
        let (x, y) = demonstrate_copy();
        assert_eq!(x, 42);
        assert_eq!(y, 42);
    }
}
