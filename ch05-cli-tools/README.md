# Chapter 5: CLI Tools

## The Big Idea

Python's `argparse` and `click` are good at parsing command-line arguments.
But most CLI tools stop there — they parse args, do a thing, and print
text. The *user* is expected to carry context between commands: remember
which server they connected to, pipe output through `jq`, and mentally
track what state the tool left behind.

Good CLI tools do more. They carry their own context: structured output
that other tools can consume, self-documenting help that teaches domain
concepts (not just flag names), and predictable behavior that composes
with other tools in a pipeline.

Rust's `clap` library, combined with serde for structured output and
the type system for enforcing correct usage, makes it natural to build
this kind of tool.

## Python Analogies

### Argument parsing = `argparse` / `click`

```python
import argparse

parser = argparse.ArgumentParser(description="Manage documents")
subparsers = parser.add_subparsers(dest="command")

add_parser = subparsers.add_parser("add", help="Add a document")
add_parser.add_argument("title", help="Document title")
add_parser.add_argument("--tag", action="append", help="Tags")

list_parser = subparsers.add_parser("list", help="List documents")
list_parser.add_argument("--format", choices=["text", "json"], default="text")

args = parser.parse_args()
```

```rust
use clap::Parser;

#[derive(Parser)]
#[command(about = "Manage documents")]
enum Cli {
    /// Add a document
    Add {
        /// Document title
        title: String,
        /// Tags for the document
        #[arg(long)]
        tag: Vec<String>,
    },
    /// List documents
    List {
        /// Output format
        #[arg(long, default_value = "text")]
        format: OutputFormat,
    },
}
```

**Key insight:** Python argument parsers are imperative — you call methods
to build up the parser. Rust's clap with derive macros is *declarative* —
you define a struct or enum and the parser is generated from the type.
The documentation, validation, and help text all come from the type
definition.

### Structured output = "don't make humans parse text"

```python
# Bad: text output that's hard to parse
def list_docs_text(docs):
    for doc in docs:
        print(f"{doc.title} ({len(doc.tags)} tags)")

# Better: structured output that tools can consume
def list_docs_json(docs):
    import json
    print(json.dumps([{"title": d.title, "tags": d.tags} for d in docs]))
```

```rust
use serde::Serialize;

#[derive(Serialize)]
struct DocOutput {
    title: String,
    tags: Vec<String>,
}

// One type, multiple output formats
fn output(docs: &[DocOutput], format: OutputFormat) {
    match format {
        OutputFormat::Text => {
            for doc in docs {
                println!("{} ({} tags)", doc.title, doc.tags.len());
            }
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(docs).unwrap());
        }
    }
}
```

**Key insight:** When your output types implement `Serialize`, you get
JSON (and YAML, TOML, etc.) for free. The same data structure serves
both human-readable and machine-readable output. No separate code paths
that can drift apart.

### Subcommands = Enums (exhaustive matching)

```python
# Python: subcommands are strings — typos cause runtime errors
if args.command == "add":
    handle_add(args)
elif args.command == "list":
    handle_list(args)
elif args.command == "remove":
    handle_remove(args)
# What if we add "update" and forget to add the elif? Silent bug.
```

```rust
// Rust: subcommands are enum variants — the compiler checks exhaustiveness
match cli {
    Cli::Add { title, tag } => handle_add(title, tag),
    Cli::List { format } => handle_list(format),
    Cli::Remove { id } => handle_remove(id),
    // If we add Cli::Update, the compiler forces us to handle it here
}
```

**Key insight:** Python subcommand dispatch is a chain of string
comparisons. Miss one and you get a silent bug. Rust enum matching is
exhaustive — add a new variant and the compiler tells you every place
that needs to handle it.

### Exit codes = Structured error reporting

```python
import sys

def main():
    try:
        result = do_work()
        print(result)
    except FileNotFoundError:
        print("Error: file not found", file=sys.stderr)
        sys.exit(1)
    except PermissionError:
        print("Error: permission denied", file=sys.stderr)
        sys.exit(2)
```

```rust
// Rust: exit codes can be an enum too
#[derive(Debug)]
enum ExitCode {
    Success = 0,
    NotFound = 1,
    PermissionDenied = 2,
}

// Or use Result as main's return type
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = do_work()?;  // errors propagate automatically
    println!("{result}");
    Ok(())
}
```

## Design Principles for Good CLI Tools

1. **Self-documenting**: Help text teaches the domain, not just the flags.
   "Add a document to the content-addressed store" is better than
   "Add a document."

2. **Structured output**: Always support `--format json` (or similar).
   Human-readable is the default; machine-readable is available.

3. **Composable**: Output that other tools can consume. Input from stdin
   when appropriate. Exit codes that scripts can check.

4. **Predictable**: Same input = same output. No hidden state changes
   that surprise the user. Make side effects visible.

5. **Context-carrying**: The tool knows what it needs. Instead of making
   the user provide 15 flags, carry configuration, know reasonable
   defaults, and explain what it's doing.

## Summary

| Python | Rust | What Changes |
|--------|------|-------------|
| `argparse` / `click` | `clap` derive | Declarative from types |
| String-based dispatch | Enum matching | Exhaustive, no silent bugs |
| Print text + `jq` | `serde_json` + format flag | One type, multiple formats |
| Manual exit codes | Result-based main | Error propagation built in |
| Flat flag lists | Nested subcommand enums | Type-safe command trees |

## Next Steps

Open `src/lib.rs` to see these concepts in working code, then try the
exercises in `exercises/`.
