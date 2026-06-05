//! # Chapter 4: Content-Addressable Data
//!
//! This module builds the concept of content-addressable data from first
//! principles: hashing, deterministic serialization, and a trait that
//! makes "data that carries its own proof of integrity" a property of
//! the type system.
//!
//! Run the tests: `cargo test -p ch04-content-addressable`

use serde::Serialize;

// ---------------------------------------------------------------------------
// 1. Basic hashing with BLAKE3
// ---------------------------------------------------------------------------

/// Hash raw bytes and return the hex string.
///
/// Python equivalent:
/// ```python
/// import hashlib
/// def hash_bytes(data: bytes) -> str:
///     return hashlib.sha256(data).hexdigest()
/// ```
///
/// We use BLAKE3 instead of SHA-256: faster, parallelizable, same security
/// guarantees for integrity checking.
pub fn hash_bytes(data: &[u8]) -> String {
    blake3::hash(data).to_hex().to_string()
}

// ---------------------------------------------------------------------------
// 2. Deterministic serialization with serde
// ---------------------------------------------------------------------------

/// A document with a title and body.
///
/// The `#[derive(Serialize)]` makes this serializable with serde.
/// Unlike Python dicts, struct field order is fixed at compile time,
/// so serialization is deterministic without sort_keys=True.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Document {
    pub title: String,
    pub body: String,
}

impl Document {
    pub fn new(title: &str, body: &str) -> Self {
        Self {
            title: title.to_string(),
            body: body.to_string(),
        }
    }
}

/// Serialize to canonical JSON bytes (compact, no trailing newline).
///
/// Python equivalent:
/// ```python
/// def canonical_json(obj) -> bytes:
///     return json.dumps(obj, sort_keys=True, separators=(',', ':')).encode()
/// ```
pub fn canonical_json<T: Serialize + ?Sized>(value: &T) -> Vec<u8> {
    serde_json::to_vec(value).expect("serialization should not fail for valid types")
}

// ---------------------------------------------------------------------------
// 3. The ContentAddressable trait
// ---------------------------------------------------------------------------

/// A trait for data that can identify itself by its content.
///
/// Any type that implements `Serialize` can become content-addressable.
/// The content ID is a BLAKE3 hash of the canonical JSON representation.
///
/// This is a default-method trait (like Chapter 3): you get `content_id()`
/// for free just by implementing Serialize and opting in.
///
/// Python equivalent:
/// ```python
/// class ContentAddressable:
///     def content_id(self) -> str:
///         canonical = json.dumps(self.__dict__, sort_keys=True, separators=(',', ':'))
///         return hashlib.blake3(canonical.encode()).hexdigest()
/// ```
pub trait ContentAddressable: Serialize {
    /// Return the content-based identifier for this value.
    ///
    /// Same content always produces the same ID. Different content
    /// (with overwhelming probability) produces a different ID.
    fn content_id(&self) -> String {
        let bytes = canonical_json(self);
        hash_bytes(&bytes)
    }

    /// Check whether two values have the same content, by comparing
    /// their content IDs.
    fn same_content(&self, other: &impl ContentAddressable) -> bool {
        self.content_id() == other.content_id()
    }
}

// Implement ContentAddressable for Document — one line!
impl ContentAddressable for Document {}

// ---------------------------------------------------------------------------
// 4. Content-addressed storage (a simple in-memory store)
// ---------------------------------------------------------------------------

/// A simple content-addressed store: values are stored by their content ID.
///
/// Python equivalent:
/// ```python
/// class ContentStore:
///     def __init__(self):
///         self._store = {}
///
///     def put(self, data: bytes) -> str:
///         cid = hash_bytes(data)
///         self._store[cid] = data
///         return cid
///
///     def get(self, cid: str) -> bytes | None:
///         return self._store.get(cid)
/// ```
///
/// The Rust version uses generics + trait bounds: the store only accepts
/// types that are ContentAddressable. The type system enforces this.
pub struct ContentStore {
    store: std::collections::HashMap<String, Vec<u8>>,
}

impl ContentStore {
    pub fn new() -> Self {
        Self {
            store: std::collections::HashMap::new(),
        }
    }

    /// Store a value and return its content ID.
    /// If the same content already exists, this is a no-op (deduplication!).
    pub fn put<T: ContentAddressable>(&mut self, value: &T) -> String {
        let cid = value.content_id();
        let bytes = canonical_json(value);
        self.store.entry(cid.clone()).or_insert(bytes);
        cid
    }

    /// Retrieve raw bytes by content ID.
    pub fn get(&self, cid: &str) -> Option<&[u8]> {
        self.store.get(cid).map(|v| v.as_slice())
    }

    /// Check if content exists without retrieving it.
    pub fn contains(&self, cid: &str) -> bool {
        self.store.contains_key(cid)
    }

    /// Number of unique items stored.
    pub fn len(&self) -> usize {
        self.store.len()
    }

    /// Whether the store is empty.
    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }
}

impl Default for ContentStore {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// 5. Verified retrieval — data that proves itself
// ---------------------------------------------------------------------------

/// Retrieve and verify: fetch by CID, re-hash, confirm integrity.
///
/// This is the key insight of content-addressed systems: the address
/// IS the checksum. If someone gives you a CID and data, you can verify
/// independently that the data matches — no trust required.
pub fn verify_content(claimed_cid: &str, data: &[u8]) -> bool {
    let actual_cid = hash_bytes(data);
    actual_cid == claimed_cid
}

// ---------------------------------------------------------------------------
// 6. Composable content addressing — hashing a tree
// ---------------------------------------------------------------------------

/// A collection of documents, itself content-addressable.
///
/// When a collection is content-addressable, its CID depends on ALL
/// of its children. Change one document and the collection's CID changes.
/// This is how Git works: a tree hash includes all its blob hashes.
#[derive(Debug, Clone, Serialize)]
pub struct DocumentCollection {
    pub name: String,
    pub documents: Vec<Document>,
}

impl DocumentCollection {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            documents: Vec::new(),
        }
    }

    pub fn add(&mut self, doc: Document) {
        self.documents.push(doc);
    }
}

impl ContentAddressable for DocumentCollection {}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Basic hashing

    #[test]
    fn hash_is_deterministic() {
        let data = b"hello, world";
        assert_eq!(hash_bytes(data), hash_bytes(data));
    }

    #[test]
    fn different_data_different_hash() {
        assert_ne!(hash_bytes(b"hello"), hash_bytes(b"world"));
    }

    // Deterministic serialization

    #[test]
    fn serialization_is_deterministic() {
        let doc = Document::new("Hello", "World");
        assert_eq!(canonical_json(&doc), canonical_json(&doc));
    }

    #[test]
    fn same_content_same_bytes() {
        let doc1 = Document::new("Hello", "World");
        let doc2 = Document::new("Hello", "World");
        assert_eq!(canonical_json(&doc1), canonical_json(&doc2));
    }

    // ContentAddressable trait

    #[test]
    fn content_id_is_deterministic() {
        let doc = Document::new("Test", "Content");
        assert_eq!(doc.content_id(), doc.content_id());
    }

    #[test]
    fn same_content_same_id() {
        let doc1 = Document::new("Hello", "World");
        let doc2 = Document::new("Hello", "World");
        assert_eq!(doc1.content_id(), doc2.content_id());
    }

    #[test]
    fn different_content_different_id() {
        let doc1 = Document::new("Hello", "World");
        let doc2 = Document::new("Hello", "Changed");
        assert_ne!(doc1.content_id(), doc2.content_id());
    }

    #[test]
    fn same_content_check() {
        let doc1 = Document::new("Hello", "World");
        let doc2 = Document::new("Hello", "World");
        assert!(doc1.same_content(&doc2));
    }

    #[test]
    fn different_content_check() {
        let doc1 = Document::new("Hello", "World");
        let doc2 = Document::new("Goodbye", "World");
        assert!(!doc1.same_content(&doc2));
    }

    // Content store

    #[test]
    fn store_and_retrieve() {
        let mut store = ContentStore::new();
        let doc = Document::new("Test", "Data");
        let cid = store.put(&doc);

        assert!(store.contains(&cid));
        assert!(store.get(&cid).is_some());
    }

    #[test]
    fn store_deduplicates() {
        let mut store = ContentStore::new();
        let doc1 = Document::new("Hello", "World");
        let doc2 = Document::new("Hello", "World");

        let cid1 = store.put(&doc1);
        let cid2 = store.put(&doc2);

        assert_eq!(cid1, cid2);
        assert_eq!(store.len(), 1); // only stored once!
    }

    #[test]
    fn store_different_content_separately() {
        let mut store = ContentStore::new();
        store.put(&Document::new("A", "1"));
        store.put(&Document::new("B", "2"));
        assert_eq!(store.len(), 2);
    }

    // Verified retrieval

    #[test]
    fn verify_valid_content() {
        let data = canonical_json(&Document::new("Test", "Data"));
        let cid = hash_bytes(&data);
        assert!(verify_content(&cid, &data));
    }

    #[test]
    fn verify_tampered_content() {
        let data = canonical_json(&Document::new("Test", "Data"));
        let cid = hash_bytes(&data);

        let mut tampered = data.clone();
        tampered[0] = b'X';
        assert!(!verify_content(&cid, &tampered));
    }

    // Composable content addressing

    #[test]
    fn collection_cid_includes_all_documents() {
        let mut col1 = DocumentCollection::new("docs");
        col1.add(Document::new("A", "1"));
        col1.add(Document::new("B", "2"));

        let mut col2 = DocumentCollection::new("docs");
        col2.add(Document::new("A", "1"));
        col2.add(Document::new("B", "2"));

        assert_eq!(col1.content_id(), col2.content_id());
    }

    #[test]
    fn collection_cid_changes_with_any_document() {
        let mut col1 = DocumentCollection::new("docs");
        col1.add(Document::new("A", "1"));
        col1.add(Document::new("B", "2"));

        let mut col2 = DocumentCollection::new("docs");
        col2.add(Document::new("A", "1"));
        col2.add(Document::new("B", "CHANGED"));

        assert_ne!(col1.content_id(), col2.content_id());
    }

    #[test]
    fn collection_cid_sensitive_to_order() {
        let mut col1 = DocumentCollection::new("docs");
        col1.add(Document::new("A", "1"));
        col1.add(Document::new("B", "2"));

        let mut col2 = DocumentCollection::new("docs");
        col2.add(Document::new("B", "2"));
        col2.add(Document::new("A", "1"));

        // Order matters! Different order = different CID
        assert_ne!(col1.content_id(), col2.content_id());
    }
}
