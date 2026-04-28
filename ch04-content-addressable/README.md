# Chapter 4: Content-Addressable Data

## The Big Idea

Most data is **location-addressed**: you find it by *where* it lives.
A file path, a URL, a database row ID — these all say "go to this place
and get whatever's there." The problem: the content at that location can
change without the address changing. You got a file from `/data/config.json`
— but is it the *same* file you got yesterday?

**Content-addressed** data flips this: the address *is derived from the
content*. You hash the data, and the hash becomes the name. If the content
changes, the name changes. If two files have the same name, they have the
same content. The data carries its own proof of integrity.

This isn't a Rust-specific idea — Git uses it (commits, trees, and blobs
are all content-addressed). Docker image layers use it. IPFS uses it.
But Rust's type system makes it particularly clean to express, because you
can use traits to make content-addressability a *property of the type
itself*, not something bolted on after the fact.

## Python Analogies

### Hashing = `hashlib`

```python
import hashlib

data = b"hello, world"
digest = hashlib.sha256(data).hexdigest()
print(digest)  # deterministic: same input always gives same output
```

```rust
let data = b"hello, world";
let hash = blake3::hash(data);
println!("{}", hash.to_hex());  // same idea, but BLAKE3 is faster
```

**Why BLAKE3?** SHA-256 is fine for most things, but BLAKE3 is:
- Faster (designed for modern CPUs, parallelizable)
- Just as secure for integrity checking
- Simpler API (no finalization step)

### Deterministic serialization = `json.dumps(sort_keys=True)`

```python
import json

# Problem: dict ordering affects the hash
data1 = {"b": 2, "a": 1}
data2 = {"a": 1, "b": 2}

# Python 3.7+ preserves insertion order, so these differ:
json.dumps(data1)  # '{"b": 2, "a": 1}'
json.dumps(data2)  # '{"a": 1, "b": 2}'

# Fix: sort keys for deterministic output
json.dumps(data1, sort_keys=True)  # '{"a": 1, "b": 2}'
json.dumps(data2, sort_keys=True)  # '{"a": 1, "b": 2}'  — same!
```

```rust
use serde::Serialize;
use serde_json;

#[derive(Serialize)]
struct Data {
    a: i32,
    b: i32,
}

// Rust structs have a fixed field order — no sorting needed.
// serde_json always serializes fields in declaration order.
let data = Data { a: 1, b: 2 };
let json = serde_json::to_string(&data).unwrap();
// Always: {"a":1,"b":2} — deterministic by construction
```

**Key insight:** Python dicts are inherently unordered (conceptually),
so you need `sort_keys=True` to get deterministic output. Rust structs
have a fixed field order defined at compile time. Determinism comes from
the type system, not from runtime flags.

### Content identifiers = "the hash IS the name"

```python
import hashlib, json

def content_id(obj):
    """Generate a content-based identifier for any JSON-serializable object."""
    canonical = json.dumps(obj, sort_keys=True, separators=(',', ':'))
    return hashlib.sha256(canonical.encode()).hexdigest()

doc1 = {"title": "Hello", "body": "World"}
doc2 = {"title": "Hello", "body": "World"}
doc3 = {"title": "Hello", "body": "Changed"}

assert content_id(doc1) == content_id(doc2)  # same content = same id
assert content_id(doc1) != content_id(doc3)  # different content = different id
```

In Rust, we can make this a *trait* — a property of the type:

```rust
trait ContentAddressable: Serialize {
    fn content_id(&self) -> String {
        let bytes = serde_json::to_vec(self).unwrap();
        blake3::hash(&bytes).to_hex().to_string()
    }
}
```

Any type that implements `Serialize` can become `ContentAddressable` with
a single line. The trait carries the behavior, and the type system ensures
you can't call `content_id()` on something that can't be serialized.

### Immutability = "frozen dataclass"

```python
from dataclasses import dataclass

@dataclass(frozen=True)
class Document:
    title: str
    body: str
    # Can't modify after creation — mutations return new instances
```

```rust
// Rust values are immutable by default. No frozen flag needed.
#[derive(Debug, Clone, Serialize)]
struct Document {
    title: String,
    body: String,
}
// You'd need `mut` to modify — and the type system tracks it
```

## Why Content-Addressability Matters

Content-addressed data gives you powerful properties for free:

1. **Integrity**: If the hash matches, the data is uncorrupted. No trust
   required — the data proves itself.

2. **Deduplication**: Same content = same hash. Store it once, reference
   it by hash from anywhere.

3. **Caching**: Hash hasn't changed? Don't recompute. The hash is a
   perfect cache key because it captures *exactly* what the content is.

4. **Provenance**: Chain of hashes = audit trail. You can prove that data
   at point B came from data at point A, because the hashes link them.

5. **Concurrency safety**: Content-addressed data is inherently immutable.
   You never update in place — you create a new version with a new hash.
   No locks needed.

These aren't theoretical benefits. Git uses content-addressing for its
entire object store. Docker uses it for image layers. Package managers
use it for dependency resolution. Any system that needs to answer "is
this the same data I saw before?" benefits from content-addressing.

## Summary

| Python | Rust | What Changes |
|--------|------|-------------|
| `hashlib.sha256()` | `blake3::hash()` | Faster, simpler API |
| `json.dumps(sort_keys=True)` | `serde_json::to_vec()` | Deterministic by struct definition |
| Manual hash functions | `ContentAddressable` trait | Hash behavior attached to the type |
| `@dataclass(frozen=True)` | Default immutability | No runtime flag needed |
| Content ID as a string | Content ID as a type | Type system tracks what's addressable |

## Next Steps

Open `src/lib.rs` to see these concepts in working code, then try the
exercises in `exercises/`.
