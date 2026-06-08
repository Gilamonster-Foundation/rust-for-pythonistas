# Chapter 5: CLI Tools

## The Big Idea

In Python, `argparse` (or `click`) parses the command line into a
`Namespace` — a loosely-typed bag of attributes. Whether `args.port` is an
`int` or a `str` depends on whether you remembered `type=int`. Whether it's
in a valid range depends on a check you wrote somewhere downstream — or
didn't. The parser and your program's expectations are two separate things
that you keep in sync by hand.

In Rust, the `clap` crate (derive API) inverts this: **the parser IS a
type**. You define a struct, and the struct *is* the command-line interface.
Field types become argument types. Doc comments become `--help` text.
Subcommands are enum variants. By the time parsing succeeds, every value is
already the right type, in the right range, one of the allowed choices.
Invalid input never makes it past the front door — the rest of your program
can't even *represent* a bad configuration.

This is Chapter 3's lesson (the compiler checks what Python checks at
runtime) applied to a program's outermost boundary: the place where
untrusted strings from a shell become the data your program runs on.

## Python Analogies

### `ArgumentParser` = a derive struct

```python
import argparse

parser = argparse.ArgumentParser(prog="greet", description="Greet someone")
parser.add_argument("name", help="Name of the person to greet")
parser.add_argument("-c", "--count", type=int, default=1,
                    help="Number of times to repeat the greeting")
parser.add_argument("-l", "--loud", action="store_true")
args = parser.parse_args()  # -> Namespace
```

```rust
use clap::Parser;

/// Greet someone
#[derive(Parser)]
struct Greet {
    /// Name of the person to greet
    name: String,

    /// Number of times to repeat the greeting
    #[arg(short, long, default_value_t = 1)]
    count: u8,

    /// Shout the greeting in uppercase
    #[arg(short, long)]
    loud: bool,
}

let args = Greet::parse();  // -> Greet, fully typed
```

**Key insight:** argparse builds the parser with imperative calls and gives
you back an untyped `Namespace`. clap derives the parser *from a type* and
gives you back that type. There is no gap between "what the parser accepts"
and "what your program expects" — they are the same declaration.

### `type=int` = the field's type (but enforced everywhere)

```python
parser.add_argument("--port", type=int, default=8080)
# Forget type=int and args.port is the string "8080".
# Nothing stops --port 99999 or --port -1 either.
```

```rust
/// Port to listen on (1024-65535)
#[arg(long, default_value_t = 8080,
      value_parser = clap::value_parser!(u16).range(1024..=65535))]
port: u16,
```

**Key insight:** `u16` makes negative and oversized ports *unrepresentable*
— not invalid, unrepresentable. The `range()` parser narrows further, and
the rejection happens at parse time with a clear error message. The parser
is the type checker, and validation is part of parsing — not something you
remember to do later.

### `choices=[...]` = `ValueEnum`

```python
parser.add_argument("--level", choices=["debug", "info", "warn", "error"])
# args.level is still a string. Every consumer compares strings,
# and a typo like `if args.level == "wran":` fails silently.
```

```rust
#[derive(ValueEnum, Clone, Copy)]
enum LogLevel { Debug, Info, Warn, Error }

#[arg(long, value_enum, default_value = "info")]
level: LogLevel,
```

**Key insight:** the parser converts `"warn"` into `LogLevel::Warn` once,
at the boundary. Downstream code uses `match` — and the compiler insists
every variant is handled. A typo'd level name isn't a silent bug; it
doesn't compile.

### click groups = subcommand enums

```python
@click.group()
def tasks(): ...

@tasks.command()
@click.argument("title")
@click.option("-p", "--priority", type=click.IntRange(1, 5), default=3)
def add(title, priority): ...

@tasks.command()
@click.argument("id", type=int)
def done(id): ...
```

```rust
#[derive(Subcommand)]
enum TaskCommand {
    /// Add a new task
    Add {
        title: String,
        #[arg(short, long, default_value_t = 3,
              value_parser = clap::value_parser!(u8).range(1..=5))]
        priority: u8,
    },
    /// Mark a task as done
    Done { id: u32 },
}
```

**Key insight:** click scatters a CLI across decorated functions; clap
gathers it into one enum you can read top to bottom. Dispatch is a `match`
— exhaustive, so adding a subcommand forces you to handle it everywhere it
matters. Forgetting one is a compile error, not a runtime surprise.

### Structured output instead of print soup

```python
# Anti-pattern: the output IS the format. Scripts that consume this
# tool end up parsing your prose with regexes.
print(f"task {id}: {title} {'(done)' if done else ''}")

# Better: keep results as data, render at the edge.
task = {"id": id, "title": title, "done": done}
print(json.dumps(task) if args.json else render_text(task))
```

```rust
#[derive(Serialize)]
struct TaskRecord { id: u32, title: String, done: bool }

fn render_tasks(tasks: &[TaskRecord], format: OutputFormat) -> String {
    match format {
        OutputFormat::Text => /* human-readable lines */,
        OutputFormat::Json => serde_json::to_string(tasks).unwrap(),
    }
}
```

**Key insight:** same discipline in both languages — compute data, render
once, at the edge — but Rust makes the shape of the output a `Serialize`
type, so `--json` output is guaranteed parseable. A tool that can speak
JSON is a tool other programs (and scripts, and agents) can build on.

### Help text as a first-class artifact

The doc comments on the struct and its fields *are* the `--help` text. The
same comment serves rustdoc, your teammates reading the source, and the
user at the terminal. Documentation can't drift from behavior because they
are one declaration. In argparse, `help="..."` strings live apart from the
code that uses the values — and quietly rot.

### Exit codes

```python
# argparse exits 2 on bad usage. Your own failures exit with whatever
# you remembered to pass to sys.exit() — often nothing, so a failed
# run exits 0 and the calling shell script merrily continues.
sys.exit(1)
```

```rust
enum CliError {
    Usage(String),    // exit 2: the user invoked the tool incorrectly
    Runtime(String),  // exit 1: the arguments were fine; the work failed
}

fn exit_code<T>(result: &Result<T, CliError>) -> u8 {
    match result {
        Ok(_) => 0,
        Err(CliError::Runtime(_)) => 1,
        Err(CliError::Usage(_)) => 2,
    }
}
```

**Key insight:** Chapter 2's `Result` reaches all the way to the shell. An
enum of failure kinds maps to exit codes in exactly one place, so `&&` and
`set -e` in calling scripts actually mean something.

## Testing CLIs Without Spawning Processes

In Python you'd reach for `subprocess.run([...])` or click's `CliRunner`.
clap parsers have `try_parse_from`, which parses from any slice of strings:

```rust
let args = Greet::try_parse_from(["greet", "Alice", "--count", "2"]).unwrap();
assert_eq!(args.count, 2);
```

Your entire CLI — parsing, validation, dispatch, rendering, exit-code
mapping — is plain functions over plain data, testable in-process. The only
untested line left is `main` calling `Greet::parse()`. Every test in this
chapter works this way; no test spawns a process.

## Summary

| Python | Rust | What Changes |
|--------|------|-------------|
| `ArgumentParser` + `add_argument` | `#[derive(Parser)]` struct | The parser is a type |
| `Namespace` (stringly-typed) | Your struct (fully typed) | No gap between parsed and expected |
| `type=int`, manual range checks | Field types + `value_parser` | Validation at parse time, invalid states unrepresentable |
| `choices=["a", "b"]` | `ValueEnum` enums | Typos become compile errors |
| click groups / `add_subparsers` | `#[derive(Subcommand)]` enums | Exhaustive dispatch via `match` |
| f-string print soup | `Serialize` + `--json` | Output is data; machines can consume it |
| `help="..."` kwargs | Doc comments | Help, docs, and code are one declaration |
| `sys.exit(n)` (if remembered) | `CliError` enum → exit code | Failure taxonomy in one place |
| `CliRunner` / `subprocess` | `try_parse_from` | Test the whole CLI in-process |

## Next Steps

Open `src/lib.rs` to see these concepts in working code, then try the
exercises in `exercises/`.
