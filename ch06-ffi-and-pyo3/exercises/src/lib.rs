//! # Chapter 6 Exercises: FFI & PyO3
//!
//! These exercises practice the boundary design pattern: writing Rust
//! code structured for Python interop. Everything is pure Rust — no
//! PyO3 required. The focus is on API design at the language boundary.
//!
//! Run tests: `cargo test -p ch06-exercises`

#![allow(unused_variables, dead_code)]

use serde::{Deserialize, Serialize};

// ============================================================
// Exercise 1: Python-Friendly Function Signatures
// ============================================================
//
// Python version:
// ```python
// def word_count(text: str) -> dict[str, int]:
//     """Count word frequencies in text."""
//     words = text.lower().split()
//     counts = {}
//     for word in words:
//         counts[word] = counts.get(word, 0) + 1
//     return counts
// ```
//
// Write a Rust function with Python-friendly types:
// - Input: &str (maps to Python str)
// - Output: HashMap<String, usize> (maps to dict[str, int])

pub fn word_count(text: &str) -> std::collections::HashMap<String, usize> {
    todo!("Split text on whitespace, lowercase, count frequencies")
}

// ============================================================
// Exercise 2: Struct with Builder Pattern
// ============================================================
//
// Python version:
// ```python
// class Record:
//     def __init__(self, key: str, value: str, ttl: int | None = None):
//         self.key = key
//         self.value = value
//         self.ttl = ttl
//
//     def to_json(self) -> str:
//         return json.dumps({"key": self.key, "value": self.value, "ttl": self.ttl})
//
//     @staticmethod
//     def from_json(json_str: str) -> 'Record':
//         data = json.loads(json_str)
//         return Record(**data)
// ```
//
// Build a Record struct with:
// - Serialize + Deserialize (for JSON roundtrip)
// - Builder method for optional TTL
// - to_json() and from_json() methods

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Record {
    pub key: String,
    pub value: String,
    pub ttl: Option<u64>,
}

impl Record {
    pub fn new(key: &str, value: &str) -> Self {
        todo!("Create a Record with ttl = None")
    }

    pub fn with_ttl(self, ttl: u64) -> Self {
        todo!("Return a new Record with the given TTL")
    }

    pub fn to_json(&self) -> String {
        todo!("Serialize to JSON string")
    }

    pub fn from_json(json: &str) -> Result<Self, String> {
        todo!("Deserialize from JSON, map error to String")
    }
}

// ============================================================
// Exercise 3: Error Handling at the Boundary
// ============================================================
//
// Python version:
// ```python
// class ValidationError(Exception):
//     pass
//
// def validate_email(email: str) -> str:
//     """Validate and normalize an email address.
//
//     Returns the normalized email, or raises ValidationError.
//     """
//     if '@' not in email:
//         raise ValidationError("missing @")
//     local, domain = email.rsplit('@', 1)
//     if not local:
//         raise ValidationError("empty local part")
//     if not domain or '.' not in domain:
//         raise ValidationError("invalid domain")
//     return f"{local}@{domain.lower()}"
// ```
//
// Write this as a Rust function returning Result.
// At the PyO3 boundary, this would become:
//   ValidationError → PyValueError
//   Ok(email) → str

#[derive(Debug, PartialEq)]
pub enum ValidationError {
    MissingAt,
    EmptyLocal,
    InvalidDomain(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingAt => write!(f, "missing @"),
            Self::EmptyLocal => write!(f, "empty local part"),
            Self::InvalidDomain(d) => write!(f, "invalid domain: {d}"),
        }
    }
}

pub fn validate_email(email: &str) -> Result<String, ValidationError> {
    todo!("Validate and normalize email: split on @, check parts, lowercase domain")
}

// ============================================================
// Exercise 4: Batch Processing with Results
// ============================================================
//
// Python version:
// ```python
// def validate_emails(emails: list[str]) -> dict:
//     """Validate multiple emails.
//
//     Returns {"valid": [...], "errors": [{"email": ..., "error": ...}]}
//     """
//     valid = []
//     errors = []
//     for email in emails:
//         try:
//             valid.append(validate_email(email))
//         except ValidationError as e:
//             errors.append({"email": email, "error": str(e)})
//     return {"valid": valid, "errors": errors}
// ```
//
// Write the Rust version. Use a struct for the result so it
// serializes cleanly to JSON for Python.

#[derive(Debug, Serialize, PartialEq)]
pub struct BatchResult {
    pub valid: Vec<String>,
    pub errors: Vec<BatchError>,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct BatchError {
    pub email: String,
    pub error: String,
}

pub fn validate_emails(emails: &[&str]) -> BatchResult {
    todo!("Validate each email, collect valid and errors separately")
}

// ============================================================
// Exercise 5: Content-Addressed Store with JSON Export
// ============================================================
//
// Python version:
// ```python
// class KeyValueStore:
//     def __init__(self):
//         self._store = {}
//
//     def put(self, key: str, value: str) -> str:
//         """Store a value, return its content hash."""
//         cid = hash_content(value.encode())
//         self._store[cid] = {"key": key, "value": value}
//         return cid
//
//     def get(self, cid: str) -> dict | None:
//         return self._store.get(cid)
//
//     def export(self) -> str:
//         """Export all entries as JSON."""
//         return json.dumps(list(self._store.values()), indent=2)
// ```
//
// Build a KeyValueStore where:
// - put() stores entries by content hash of the value
// - get() retrieves by CID
// - export() returns JSON string (for Python to deserialize)

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Entry {
    pub key: String,
    pub value: String,
    pub cid: String,
}

pub struct KeyValueStore {
    entries: std::collections::HashMap<String, Entry>,
}

impl KeyValueStore {
    pub fn new() -> Self {
        Self {
            entries: std::collections::HashMap::new(),
        }
    }

    /// Store a key-value pair, return the content ID (hash of value bytes).
    pub fn put(&mut self, key: &str, value: &str) -> String {
        todo!("Hash the value, store an Entry, return the CID")
    }

    /// Retrieve an entry by CID.
    pub fn get(&self, cid: &str) -> Option<&Entry> {
        todo!("Look up by CID")
    }

    /// Export all entries as a JSON string.
    pub fn export(&self) -> String {
        todo!("Serialize all entries to JSON")
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for KeyValueStore {
    fn default() -> Self {
        Self::new()
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
    fn ex1_word_count() {
        let counts = word_count("hello world hello");
        assert_eq!(counts["hello"], 2);
        assert_eq!(counts["world"], 1);
    }

    #[test]
    fn ex1_word_count_case_insensitive() {
        let counts = word_count("Hello HELLO hello");
        assert_eq!(counts["hello"], 3);
    }

    #[test]
    fn ex1_empty_string() {
        let counts = word_count("");
        assert!(counts.is_empty());
    }

    // Exercise 2
    #[test]
    fn ex2_record_new() {
        let r = Record::new("name", "Alice");
        assert_eq!(r.key, "name");
        assert_eq!(r.value, "Alice");
        assert_eq!(r.ttl, None);
    }

    #[test]
    fn ex2_record_with_ttl() {
        let r = Record::new("name", "Alice").with_ttl(3600);
        assert_eq!(r.ttl, Some(3600));
    }

    #[test]
    fn ex2_record_json_roundtrip() {
        let original = Record::new("key", "value").with_ttl(60);
        let json = original.to_json();
        let restored = Record::from_json(&json).unwrap();
        assert_eq!(original, restored);
    }

    #[test]
    fn ex2_record_from_invalid_json() {
        assert!(Record::from_json("not json").is_err());
    }

    // Exercise 3
    #[test]
    fn ex3_valid_email() {
        assert_eq!(
            validate_email("user@Example.COM"),
            Ok("user@example.com".to_string())
        );
    }

    #[test]
    fn ex3_missing_at() {
        assert_eq!(validate_email("invalid"), Err(ValidationError::MissingAt));
    }

    #[test]
    fn ex3_empty_local() {
        assert_eq!(
            validate_email("@example.com"),
            Err(ValidationError::EmptyLocal)
        );
    }

    #[test]
    fn ex3_invalid_domain() {
        assert!(matches!(
            validate_email("user@nodot"),
            Err(ValidationError::InvalidDomain(_))
        ));
    }

    // Exercise 4
    #[test]
    fn ex4_batch_validation() {
        let result = validate_emails(&["good@example.com", "bad", "also@good.org"]);
        assert_eq!(result.valid.len(), 2);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].email, "bad");
    }

    #[test]
    fn ex4_all_valid() {
        let result = validate_emails(&["a@b.com", "c@d.org"]);
        assert_eq!(result.valid.len(), 2);
        assert!(result.errors.is_empty());
    }

    // Exercise 5
    #[test]
    fn ex5_put_and_get() {
        let mut store = KeyValueStore::new();
        let cid = store.put("greeting", "hello");
        let entry = store.get(&cid).unwrap();
        assert_eq!(entry.key, "greeting");
        assert_eq!(entry.value, "hello");
    }

    #[test]
    fn ex5_same_value_same_cid() {
        let mut store = KeyValueStore::new();
        let cid1 = store.put("key1", "same-value");
        let cid2 = store.put("key2", "same-value");
        assert_eq!(cid1, cid2);
        // Note: second put overwrites the first (same CID)
    }

    #[test]
    fn ex5_export_json() {
        let mut store = KeyValueStore::new();
        store.put("name", "Alice");

        let json = store.export();
        let parsed: Vec<Entry> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].key, "name");
    }

    #[test]
    fn ex5_cid_is_hex() {
        let mut store = KeyValueStore::new();
        let cid = store.put("k", "v");
        assert_eq!(cid.len(), 64);
        assert!(cid.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
