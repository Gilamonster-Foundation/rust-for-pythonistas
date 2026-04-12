# CLAUDE.md — rust-for-pythonistas

## Safety Rules (non-negotiable)

This is a **public repository** under the Gilamonster Foundation org.
Every commit is visible to the world. Follow these rules absolutely:

1. **No private repo references.** Never mention `kyln`, `kyln-scm`,
   `gilabot`, `gilamonster` (as a project name), or any `hartsock/*` repo
   by name or URL. Discuss concepts abstractly instead.

2. **No private infrastructure.** No internal IPs, hostnames (`gnuc`,
   `server1`, `geforcenuc`), or filesystem paths (`/home/hartsock`).

3. **No secrets.** No API keys, tokens, passwords, or credentials. Ever.

4. **No personal identifiers beyond the git commit author.** The git
   author is `Shawn Hartsock <hartsock@users.noreply.github.com>` — that's
   fine. But don't embed personal details, work context, or private
   project names in code or prose.

5. **Concepts, not implementations.** When the course builds toward
   advanced ideas (content-addressable data, data provenance, CLI context),
   teach the *concept* from first principles. Don't say "as implemented
   in project X" — build it fresh in the exercises.

## Content Rules

- Every chapter maps Rust concepts to Python patterns the reader already knows
- Code must compile and tests must pass (`cargo test --workspace`)
- Exercises use `todo!()` markers — tests should fail until solved
- Zero warnings: `cargo clippy --workspace -- -D warnings`

## Build

```bash
cargo test --workspace           # Run all tests (ch examples pass, exercises fail with todo!)
cargo clippy --workspace -- -D warnings   # Lint
cargo fmt --all -- --check       # Format check
```
