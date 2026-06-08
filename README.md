# Rust for Pythonistas

**Learn Rust through Python analogies — from ownership to content-addressable data.**

This is a hands-on course for Python developers who want to learn Rust. Rather
than starting from scratch, each chapter maps Rust concepts to Python patterns
you already know, then shows you what Rust makes possible beyond that.

The course gradually builds toward real-world systems concepts:
content-addressable data (where data carries its own proof of integrity),
data provenance (knowing where your data came from and that it hasn't been
tampered with), and CLI tools that carry their own context instead of
demanding the user carry it for them.

**Where this is headed: Python ↔ Rust interop.** That is this course's
specific domain — and the Gilamonster Foundation's contribution focus. The
destination is not "rewrite it in Rust"; it is knowing how to reach for Rust
*from* Python: binding Rust crates into Python packages, escaping the GIL
with real threads, and borrowing Rust's type system to harden Python APIs —
so the part of your codebase that needs to be fast or correct can be, without
giving up everything that makes Python productive.

## Prerequisites

- Comfortable reading and writing Python
- Rust toolchain installed ([rustup.rs](https://rustup.rs))
- Curiosity about systems programming

## Chapters

| # | Title | Python Concept | Rust Concept |
|---|-------|---------------|--------------|
| 1 | **Ownership** | Reference counting, `del`, `with` blocks | Ownership, borrowing, lifetimes, RAII |
| 2 | **Error Handling** | `try/except`, `None`, `LBYL vs EAFP` | `Result<T,E>`, `Option<T>`, `?` operator |
| 3 | **Traits & Generics** | ABCs, Protocols, duck typing | Traits, generics, trait bounds |
| 4 | **Content-Addressable Data** | `hashlib`, `json.dumps(sort_keys=True)` | BLAKE3, serde, CIDs, deterministic serialization |
| 5 | **CLI Tools** | `argparse`, `click` | `clap`, structured output, self-documenting tools |
| 6 | **FFI & PyO3** | `ctypes`, C extensions, wheels | PyO3 bindings, `maturin`, packaging Rust as a Python module |
| 7 | **PyO3 House Style** | "how should this feel from Python?" | feature-gated bindings, error mapping to exceptions, zero-copy data exchange |
| 8 | **Escaping the GIL** | `threading` vs `multiprocessing`, the GIL | `Send`/`Sync`, `rayon`, releasing the GIL across FFI calls |

## Course Roadmap

The arc continues toward the interop boundary — each planned chapter takes
one thing Python developers wish they had and shows how Rust provides it
*to* Python, not instead of it:

| # | Title | Python Concept | Rust Concept |
|---|-------|---------------|--------------|
| 9 | **Types That Travel** | type hints, `mypy`, runtime validation | newtypes, exhaustive enums, type-safe IDs surfacing as Python types |
| 10 | **Case Study: Speed Where It Counts** | profiling a real hot path | swapping it for a published Rust crate and measuring the win |

Chapter 10 lands when the Foundation's own crates (the Hermes-Thoon line)
are published — the case study will use real, released code rather than a
toy benchmark.

## How to Use This Course

Each chapter is a Cargo crate with:

- **`src/lib.rs`** — Annotated examples with Python comparisons in comments
- **`exercises/`** — "Translate this Python" challenges with test suites
- **Chapter README** — Concept explanations and mental model bridges

### Run examples

```bash
# Run all tests in a chapter
cargo test -p ch01-ownership

# Run the exercises for a chapter
cargo test -p ch01-exercises
```

### Work through exercises

1. Read the chapter's `src/lib.rs` for the concepts
2. Open `exercises/src/lib.rs` — each exercise has a Python snippet and a
   Rust skeleton with `todo!()` markers
3. Fill in the Rust code
4. Run `cargo test -p ch01-exercises` to check your work

## Philosophy

> "Computer Science has as much to do with Computers as Astronomy does with
> Telescopes." — Edsger W. Dijkstra

Rust is a telescope. This course isn't about Rust syntax — it's about the
things Rust lets you *see*: where memory lives, who owns data, how errors
propagate, and how to build systems where data carries its own proof of
integrity.

## License

Apache License 2.0 — see [LICENSE](LICENSE) for details.
