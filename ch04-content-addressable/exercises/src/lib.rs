//! # Chapter 4 Exercises: Content-Addressable Data
//!
//! These exercises build a mini content-addressable system from scratch.
//! Each exercise builds on the previous one.
//!
//! Run tests: `cargo test -p ch04-exercises`

#![allow(unused_variables, dead_code)]

use serde::Serialize;

// ============================================================
// Exercise 1: Hash Function
// ============================================================
//
// Python version:
// ```python
// import hashlib
// def hash_content(data: bytes) -> str:
//     return hashlib.blake2b(data, digest_size=32).hexdigest()
// ```
//
// Use blake3 to hash bytes and return a hex string.

pub fn hash_content(data: &[u8]) -> String {
    todo!("Hash the data with blake3 and return the hex string")
}

// ============================================================
// Exercise 2: A ContentAddressable Trait
// ============================================================
//
// Python version:
// ```python
// class ContentAddressable:
//     def canonical_bytes(self) -> bytes:
//         raise NotImplementedError
//
//     def content_id(self) -> str:
//         return hash_content(self.canonical_bytes())
// ```
//
// Define a trait with:
// - A required method `canonical_bytes(&self) -> Vec<u8>`
// - A default method `content_id(&self) -> String` that hashes the bytes

pub trait ContentAddressable {
    /// Required: return the canonical byte representation of this value.
    fn canonical_bytes(&self) -> Vec<u8>;

    /// Default: hash the canonical bytes to produce a content identifier.
    fn content_id(&self) -> String {
        todo!("Call canonical_bytes, pass to hash_content")
    }
}

// ============================================================
// Exercise 3: Implement ContentAddressable for a Config Struct
// ============================================================
//
// Python version:
// ```python
// @dataclass(frozen=True)
// class Config:
//     name: str
//     version: int
//     debug: bool
//
//     def canonical_bytes(self):
//         return json.dumps(
//             {"name": self.name, "version": self.version, "debug": self.debug},
//             sort_keys=True, separators=(',', ':')
//         ).encode()
// ```
//
// Implement ContentAddressable for Config using serde_json serialization.

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Config {
    pub name: String,
    pub version: u32,
    pub debug: bool,
}

impl Config {
    pub fn new(name: &str, version: u32, debug: bool) -> Self {
        Self {
            name: name.to_string(),
            version,
            debug,
        }
    }
}

impl ContentAddressable for Config {
    fn canonical_bytes(&self) -> Vec<u8> {
        todo!("Serialize self to JSON bytes using serde_json::to_vec")
    }
}

// ============================================================
// Exercise 4: Content-Addressed Cache
// ============================================================
//
// Python version:
// ```python
// class ContentCache:
//     def __init__(self):
//         self._cache = {}
//
//     def get_or_compute(self, key_obj, compute_fn):
//         cid = key_obj.content_id()
//         if cid not in self._cache:
//             self._cache[cid] = compute_fn(key_obj)
//         return self._cache[cid]
// ```
//
// Build a cache that uses content IDs as keys. If two different Config
// objects have the same content, the computation should only run once.

pub struct ContentCache {
    cache: std::collections::HashMap<String, String>,
}

impl Default for ContentCache {
    fn default() -> Self {
        Self::new()
    }
}

impl ContentCache {
    pub fn new() -> Self {
        Self {
            cache: std::collections::HashMap::new(),
        }
    }

    /// Get a cached result or compute it.
    ///
    /// If an entry with the same content_id already exists, return it.
    /// Otherwise, call `compute` with the value, store the result, and return it.
    pub fn get_or_compute<T: ContentAddressable>(
        &mut self,
        value: &T,
        compute: impl FnOnce(&T) -> String,
    ) -> String {
        todo!("Check if content_id is in cache; if not, compute and store")
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

// ============================================================
// Exercise 5: Merkle-Style Chaining
// ============================================================
//
// Python version:
// ```python
// class Snapshot:
//     def __init__(self, data, parent_cid=None):
//         self.data = data
//         self.parent_cid = parent_cid
//
//     def canonical_bytes(self):
//         obj = {"data": self.data, "parent": self.parent_cid}
//         return json.dumps(obj, sort_keys=True, separators=(',', ':')).encode()
//
//     def content_id(self):
//         return hash_content(self.canonical_bytes())
// ```
//
// A Snapshot includes a reference to its parent's content ID, creating
// a chain. Changing any snapshot changes all descendant CIDs.
// This is the same principle behind Git commits and blockchain blocks.
//
// Implement ContentAddressable for Snapshot.

#[derive(Debug, Clone, Serialize)]
pub struct Snapshot {
    pub data: String,
    pub parent_cid: Option<String>,
}

impl Snapshot {
    pub fn root(data: &str) -> Self {
        Self {
            data: data.to_string(),
            parent_cid: None,
        }
    }

    pub fn child(data: &str, parent: &impl ContentAddressable) -> Self {
        Self {
            data: data.to_string(),
            parent_cid: Some(parent.content_id()),
        }
    }
}

impl ContentAddressable for Snapshot {
    fn canonical_bytes(&self) -> Vec<u8> {
        todo!("Serialize self to JSON bytes")
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
    fn ex1_hash_deterministic() {
        assert_eq!(hash_content(b"hello"), hash_content(b"hello"));
    }

    #[test]
    fn ex1_hash_different_input() {
        assert_ne!(hash_content(b"hello"), hash_content(b"world"));
    }

    #[test]
    fn ex1_hash_is_hex() {
        let h = hash_content(b"test");
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(h.len(), 64); // BLAKE3 produces 256-bit = 64 hex chars
    }

    // Exercise 2 + 3
    #[test]
    fn ex3_config_content_id_deterministic() {
        let cfg = Config::new("app", 1, false);
        assert_eq!(cfg.content_id(), cfg.content_id());
    }

    #[test]
    fn ex3_same_config_same_id() {
        let cfg1 = Config::new("app", 1, false);
        let cfg2 = Config::new("app", 1, false);
        assert_eq!(cfg1.content_id(), cfg2.content_id());
    }

    #[test]
    fn ex3_different_config_different_id() {
        let cfg1 = Config::new("app", 1, false);
        let cfg2 = Config::new("app", 2, false);
        assert_ne!(cfg1.content_id(), cfg2.content_id());
    }

    #[test]
    fn ex3_debug_flag_changes_id() {
        let cfg1 = Config::new("app", 1, false);
        let cfg2 = Config::new("app", 1, true);
        assert_ne!(cfg1.content_id(), cfg2.content_id());
    }

    // Exercise 4
    #[test]
    fn ex4_cache_computes_once() {
        let mut cache = ContentCache::new();
        let cfg = Config::new("app", 1, false);

        let mut call_count = 0;

        let result1 = cache.get_or_compute(&cfg, |c| {
            call_count += 1;
            format!("computed-{}", c.name)
        });

        // Same content, different object — should hit cache
        let cfg2 = Config::new("app", 1, false);
        let result2 = cache.get_or_compute(&cfg2, |c| {
            call_count += 1;
            format!("computed-{}", c.name)
        });

        assert_eq!(result1, "computed-app");
        assert_eq!(result1, result2);
        assert_eq!(call_count, 1); // computed only once!
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn ex4_cache_different_content() {
        let mut cache = ContentCache::new();

        cache.get_or_compute(&Config::new("a", 1, false), |c| c.name.clone());
        cache.get_or_compute(&Config::new("b", 1, false), |c| c.name.clone());

        assert_eq!(cache.len(), 2);
    }

    // Exercise 5
    #[test]
    fn ex5_root_snapshot() {
        let root = Snapshot::root("initial");
        assert!(root.parent_cid.is_none());
        let cid = root.content_id();
        assert_eq!(cid.len(), 64);
    }

    #[test]
    fn ex5_child_includes_parent_cid() {
        let root = Snapshot::root("initial");
        let child = Snapshot::child("update", &root);
        assert_eq!(child.parent_cid, Some(root.content_id()));
    }

    #[test]
    fn ex5_chain_integrity() {
        let root = Snapshot::root("v1");
        let child = Snapshot::child("v2", &root);
        let grandchild = Snapshot::child("v3", &child);

        // Changing the root changes the entire chain
        let alt_root = Snapshot::root("v1-tampered");
        let alt_child = Snapshot::child("v2", &alt_root);
        let alt_grandchild = Snapshot::child("v3", &alt_child);

        assert_ne!(root.content_id(), alt_root.content_id());
        assert_ne!(child.content_id(), alt_child.content_id());
        assert_ne!(grandchild.content_id(), alt_grandchild.content_id());
    }

    #[test]
    fn ex5_same_data_different_parent_different_cid() {
        let root1 = Snapshot::root("a");
        let root2 = Snapshot::root("b");

        let child1 = Snapshot::child("same-data", &root1);
        let child2 = Snapshot::child("same-data", &root2);

        // Same child data but different parents = different CIDs
        assert_ne!(child1.content_id(), child2.content_id());
    }
}
