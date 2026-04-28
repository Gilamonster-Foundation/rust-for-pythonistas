//! # Chapter 6: FFI & PyO3
//!
//! This module demonstrates the "core layer" pattern for Rust code that
//! will be called from Python. Everything here is pure Rust — no PyO3
//! dependency. The design makes adding a PyO3 boundary layer trivial.
//!
//! Run the tests: `cargo test -p ch06-ffi-and-pyo3`

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// 1. Core functions — Python-friendly signatures
// ---------------------------------------------------------------------------

/// Hash bytes and return a hex string.
///
/// This is the kind of function that maps perfectly to Python:
/// - Input: `&[u8]` maps to Python `bytes`
/// - Output: `String` maps to Python `str`
///
/// PyO3 boundary would be:
/// ```text
/// #[pyfunction]
/// fn hash_bytes(data: &[u8]) -> String {
///     hash_bytes_core(data)
/// }
/// ```
pub fn hash_bytes_core(data: &[u8]) -> String {
    blake3::hash(data).to_hex().to_string()
}

/// Parse a JSON string into a structured config.
///
/// Input: `&str` (Python `str`) → Output: `Result<Config, ConfigError>`
///
/// At the boundary, the Result becomes a Python exception:
/// ```text
/// #[pyfunction]
/// fn parse_config(json: &str) -> PyResult<PyConfig> {
///     parse_config_core(json)
///         .map(PyConfig::from)
///         .map_err(|e| PyValueError::new_err(e.to_string()))
/// }
/// ```
pub fn parse_config_core(json: &str) -> Result<Config, ConfigError> {
    serde_json::from_str(json).map_err(|e| ConfigError::InvalidJson(e.to_string()))
}

// ---------------------------------------------------------------------------
// 2. Data types designed for the boundary
// ---------------------------------------------------------------------------

/// A configuration struct that maps cleanly to Python.
///
/// All fields are types that PyO3 auto-converts:
/// - String ↔ str
/// - u32 ↔ int
/// - bool ↔ bool
/// - Vec<String> ↔ list[str]
/// - Option<String> ↔ Optional[str]
///
/// PyO3 boundary would be:
/// ```text
/// #[pyclass]
/// struct PyConfig {
///     #[pyo3(get)]
///     name: String,
///     #[pyo3(get)]
///     version: u32,
///     ...
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    pub name: String,
    pub version: u32,
    pub debug: bool,
    pub tags: Vec<String>,
    pub description: Option<String>,
}

impl Config {
    pub fn new(name: &str, version: u32) -> Self {
        Self {
            name: name.to_string(),
            version,
            debug: false,
            tags: Vec::new(),
            description: None,
        }
    }

    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = Some(desc.to_string());
        self
    }
}

/// Error type for config parsing.
///
/// At the PyO3 boundary, each variant maps to a different Python exception:
/// - InvalidJson → ValueError
/// - MissingField → KeyError
#[derive(Debug, PartialEq)]
pub enum ConfigError {
    InvalidJson(String),
    MissingField(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidJson(msg) => write!(f, "invalid JSON: {msg}"),
            Self::MissingField(field) => write!(f, "missing field: {field}"),
        }
    }
}

// ---------------------------------------------------------------------------
// 3. Builder pattern — Pythonic object construction
// ---------------------------------------------------------------------------

/// A content-addressed document with a builder pattern.
///
/// Python users expect keyword arguments:
/// ```python
/// doc = Document(title="Hello", body="World", tags=["greeting"])
/// ```
///
/// Rust doesn't have keyword args, but the builder pattern serves
/// the same purpose — and PyO3 can map `#[new]` with keyword args:
/// ```text
/// #[pymethods]
/// impl PyDocument {
///     #[new]
///     #[pyo3(signature = (title, body, tags=vec![]))]
///     fn new(title: String, body: String, tags: Vec<String>) -> Self { ... }
/// }
/// ```
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Document {
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
}

impl Document {
    pub fn new(title: &str, body: &str) -> Self {
        Self {
            title: title.to_string(),
            body: body.to_string(),
            tags: Vec::new(),
        }
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Content ID — the hash of the serialized document.
    pub fn content_id(&self) -> String {
        let bytes = serde_json::to_vec(self).expect("document serialization should not fail");
        hash_bytes_core(&bytes)
    }
}

// ---------------------------------------------------------------------------
// 4. A service layer — the core business logic
// ---------------------------------------------------------------------------

/// A document store that Python can interact with.
///
/// At the boundary, this becomes a `#[pyclass]` with `#[pymethods]`.
/// The core logic stays here, testable without Python.
pub struct DocumentStore {
    documents: std::collections::HashMap<String, Document>,
}

impl DocumentStore {
    pub fn new() -> Self {
        Self {
            documents: std::collections::HashMap::new(),
        }
    }

    /// Store a document, return its content ID.
    pub fn put(&mut self, doc: Document) -> String {
        let cid = doc.content_id();
        self.documents.insert(cid.clone(), doc);
        cid
    }

    /// Retrieve a document by content ID.
    pub fn get(&self, cid: &str) -> Option<&Document> {
        self.documents.get(cid)
    }

    /// Search documents by title substring.
    pub fn search(&self, query: &str) -> Vec<(&str, &Document)> {
        let query_lower = query.to_lowercase();
        self.documents
            .iter()
            .filter(|(_, doc)| doc.title.to_lowercase().contains(&query_lower))
            .map(|(cid, doc)| (cid.as_str(), doc))
            .collect()
    }

    /// List all content IDs.
    pub fn list_ids(&self) -> Vec<&str> {
        self.documents.keys().map(|s| s.as_str()).collect()
    }

    /// Number of documents.
    pub fn len(&self) -> usize {
        self.documents.len()
    }

    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }

    /// Export all documents as JSON (for Python interop).
    ///
    /// This pattern is common: Rust returns JSON, Python deserializes.
    /// It's simpler than converting each field individually.
    pub fn export_json(&self) -> String {
        let entries: Vec<_> = self
            .documents
            .iter()
            .map(|(cid, doc)| {
                serde_json::json!({
                    "cid": cid,
                    "title": doc.title,
                    "body": doc.body,
                    "tags": doc.tags,
                })
            })
            .collect();
        serde_json::to_string_pretty(&entries).expect("export should not fail")
    }
}

impl Default for DocumentStore {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// 5. Batch operations — processing lists
// ---------------------------------------------------------------------------

/// Hash multiple items and return their CIDs.
///
/// Python equivalent:
/// ```python
/// def hash_all(items: list[bytes]) -> list[str]:
///     return [hash_bytes(item) for item in items]
/// ```
///
/// At the boundary: Vec<Vec<u8>> (Python list[bytes]) → Vec<String> (list[str])
pub fn hash_all(items: &[&[u8]]) -> Vec<String> {
    items.iter().map(|item| hash_bytes_core(item)).collect()
}

/// Verify a batch of (cid, data) pairs.
///
/// Returns the indices of any items that fail verification.
/// Python can use this to report which items are corrupted.
pub fn verify_batch(pairs: &[(String, Vec<u8>)]) -> Vec<usize> {
    pairs
        .iter()
        .enumerate()
        .filter_map(|(i, (cid, data))| {
            let actual = hash_bytes_core(data);
            if actual != *cid {
                Some(i)
            } else {
                None
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Core functions

    #[test]
    fn hash_bytes_returns_hex() {
        let hash = hash_bytes_core(b"hello");
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn parse_config_valid_json() {
        let json = r#"{"name":"app","version":1,"debug":false,"tags":[],"description":null}"#;
        let config = parse_config_core(json).unwrap();
        assert_eq!(config.name, "app");
        assert_eq!(config.version, 1);
    }

    #[test]
    fn parse_config_invalid_json() {
        let result = parse_config_core("not json");
        assert!(result.is_err());
    }

    // Data types

    #[test]
    fn config_builder() {
        let config = Config::new("app", 1)
            .with_debug(true)
            .with_tags(vec!["prod".to_string()])
            .with_description("My app");

        assert_eq!(config.name, "app");
        assert!(config.debug);
        assert_eq!(config.tags, vec!["prod"]);
        assert_eq!(config.description, Some("My app".to_string()));
    }

    #[test]
    fn config_roundtrip_json() {
        let original = Config::new("test", 2).with_debug(true);
        let json = serde_json::to_string(&original).unwrap();
        let restored: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(original, restored);
    }

    // Document + content addressing

    #[test]
    fn document_content_id_deterministic() {
        let doc1 = Document::new("Hello", "World");
        let doc2 = Document::new("Hello", "World");
        assert_eq!(doc1.content_id(), doc2.content_id());
    }

    #[test]
    fn document_different_content_different_id() {
        let doc1 = Document::new("Hello", "World");
        let doc2 = Document::new("Hello", "Changed");
        assert_ne!(doc1.content_id(), doc2.content_id());
    }

    // Document store

    #[test]
    fn store_put_and_get() {
        let mut store = DocumentStore::new();
        let doc = Document::new("Test", "Content");
        let cid = store.put(doc.clone());
        assert_eq!(store.get(&cid), Some(&doc));
    }

    #[test]
    fn store_deduplicates() {
        let mut store = DocumentStore::new();
        let cid1 = store.put(Document::new("A", "B"));
        let cid2 = store.put(Document::new("A", "B"));
        assert_eq!(cid1, cid2);
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn store_search() {
        let mut store = DocumentStore::new();
        store.put(Document::new("Rust Guide", "Learn Rust"));
        store.put(Document::new("Python Guide", "Learn Python"));
        store.put(Document::new("Cooking 101", "Learn Cooking"));

        let results = store.search("guide");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn store_export_json() {
        let mut store = DocumentStore::new();
        store.put(Document::new("Test", "Data"));

        let json = store.export_json();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0]["title"], "Test");
    }

    // Batch operations

    #[test]
    fn hash_all_items() {
        let items: Vec<&[u8]> = vec![b"a", b"b", b"c"];
        let hashes = hash_all(&items);
        assert_eq!(hashes.len(), 3);
        assert_ne!(hashes[0], hashes[1]);
    }

    #[test]
    fn verify_batch_all_valid() {
        let data = vec![b"hello".to_vec(), b"world".to_vec()];
        let pairs: Vec<(String, Vec<u8>)> = data
            .iter()
            .map(|d| (hash_bytes_core(d), d.clone()))
            .collect();

        assert!(verify_batch(&pairs).is_empty());
    }

    #[test]
    fn verify_batch_detects_corruption() {
        let pairs = vec![
            (hash_bytes_core(b"hello"), b"hello".to_vec()),
            ("bad_hash".to_string(), b"world".to_vec()),
            (hash_bytes_core(b"foo"), b"foo".to_vec()),
        ];

        let bad = verify_batch(&pairs);
        assert_eq!(bad, vec![1]);
    }
}
