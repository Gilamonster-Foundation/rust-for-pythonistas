//! # Chapter 5: CLI Tools
//!
//! This module maps Python's `argparse`/`click` patterns to Rust's `clap`
//! (derive API). The central idea: in clap, the parser IS a type. Parsing
//! and validation happen in one step, and the rest of your program receives
//! real typed values instead of a stringly-typed namespace.
//!
//! Run the tests: `cargo test -p ch05-cli-tools`

use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use serde::Serialize;

// ---------------------------------------------------------------------------
// 1. Your first parser — argparse ArgumentParser vs clap derive
// ---------------------------------------------------------------------------

/// Greet someone from the command line.
///
/// Python equivalent:
/// ```python
/// import argparse
///
/// parser = argparse.ArgumentParser(prog="greet", description="Greet someone")
/// parser.add_argument("name", help="Name of the person to greet")
/// parser.add_argument("-c", "--count", type=int, default=1,
///                     help="Number of times to repeat the greeting")
/// parser.add_argument("-l", "--loud", action="store_true",
///                     help="Shout the greeting in uppercase")
/// args = parser.parse_args()
/// ```
///
/// In clap's derive API, the parser IS the struct. Each field becomes an
/// argument, the field type becomes the argument type, and the doc comments
/// become the `--help` text. No separate `add_argument` calls — the struct
/// definition is the single source of truth.
#[derive(Parser, Debug, PartialEq)]
#[command(name = "greet", version = "1.0.0", about = "Greet someone")]
pub struct Greet {
    /// Name of the person to greet
    pub name: String,

    /// Number of times to repeat the greeting
    #[arg(short, long, default_value_t = 1)]
    pub count: u8,

    /// Shout the greeting in uppercase
    #[arg(short, long)]
    pub loud: bool,
}

/// Build the greeting from parsed arguments.
///
/// Notice the signature: this takes `&Greet`, not "whatever came off the
/// command line". By the time this function runs, `count` is already a u8
/// and `loud` is already a bool. There is nothing left to validate.
pub fn greeting(args: &Greet) -> String {
    let line = format!("Hello, {}!", args.name);
    let line = if args.loud { line.to_uppercase() } else { line };
    vec![line; args.count as usize].join("\n")
}

// ---------------------------------------------------------------------------
// 2. Typed arguments — the parser IS the type checker
// ---------------------------------------------------------------------------

/// Serve files from a directory.
///
/// Python equivalent:
/// ```python
/// parser = argparse.ArgumentParser(prog="serve")
/// parser.add_argument("--port", type=int, default=8080)
/// parser.add_argument("root", nargs="?", default=".")
/// args = parser.parse_args()
///
/// # argparse gives you a Namespace. args.port is an int *if* you remembered
/// # type=int — otherwise it's a string and you find out deep in your code.
/// # And nothing stops port=99999 or port=-1 from sliding through.
/// if not (1024 <= args.port <= 65535):
///     parser.error("port out of range")   # manual, easy to forget
/// ```
///
/// In Rust the field type does the work. `u16` already makes 99999 and -1
/// unrepresentable; the `range()` value parser narrows it further to the
/// unprivileged ports. Invalid input is rejected at the front door — your
/// program logic never sees it.
#[derive(Parser, Debug, PartialEq)]
#[command(name = "serve", about = "Serve files from a directory")]
pub struct Serve {
    /// Port to listen on (1024-65535)
    #[arg(
        short,
        long,
        default_value_t = 8080,
        value_parser = clap::value_parser!(u16).range(1024..=65535)
    )]
    pub port: u16,

    /// Directory to serve files from
    #[arg(default_value = ".")]
    pub root: std::path::PathBuf,
}

// ---------------------------------------------------------------------------
// 3. Choices become enums — invalid states are unrepresentable
// ---------------------------------------------------------------------------

/// Log levels as a real enum.
///
/// Python equivalent:
/// ```python
/// parser.add_argument("--level", choices=["debug", "info", "warn", "error"],
///                     default="info")
/// # ...but args.level is still a *string*. Every consumer compares
/// # strings, and a typo like `if args.level == "wran":` fails silently.
/// ```
///
/// With `ValueEnum`, the parser converts "warn" into `LogLevel::Warn` at
/// parse time. Downstream code matches on the enum — and the compiler
/// rejects a match arm for a level that doesn't exist.
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    /// Numeric severity, for threshold comparisons.
    ///
    /// A match on an enum must be exhaustive: add a `Trace` variant later
    /// and this function stops compiling until you handle it. Python's
    /// string comparisons would just silently never match.
    pub fn severity(&self) -> u8 {
        match self {
            LogLevel::Debug => 0,
            LogLevel::Info => 1,
            LogLevel::Warn => 2,
            LogLevel::Error => 3,
        }
    }
}

/// Configure logging verbosity.
#[derive(Parser, Debug, PartialEq)]
#[command(name = "logdemo")]
pub struct Logging {
    /// Minimum level to print
    #[arg(long, value_enum, default_value = "info")]
    pub level: LogLevel,
}

// ---------------------------------------------------------------------------
// 4. Subcommands — click groups vs clap subcommand enums
// ---------------------------------------------------------------------------

/// A tiny task tracker.
///
/// Python equivalent (click):
/// ```python
/// @click.group()
/// @click.option("--json", "as_json", is_flag=True)
/// def tasks(as_json): ...
///
/// @tasks.command()
/// @click.argument("title")
/// @click.option("-p", "--priority", type=click.IntRange(1, 5), default=3)
/// def add(title, priority): ...
///
/// @tasks.command(name="list")
/// @click.option("--all", "show_all", is_flag=True)
/// def list_(show_all): ...
///
/// @tasks.command()
/// @click.argument("id", type=int)
/// def done(id): ...
/// ```
///
/// click models subcommands as decorated functions discovered at runtime.
/// clap models them as an enum: each subcommand is a variant, each variant's
/// fields are that subcommand's arguments. The full command surface of your
/// CLI is one type you can read top to bottom.
#[derive(Parser, Debug)]
#[command(name = "tasks", about = "A tiny task tracker")]
pub struct TasksCli {
    /// Emit machine-readable JSON instead of human-readable text
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: TaskCommand,
}

/// One variant per subcommand. Dispatch is a `match` — and it must be
/// exhaustive, so adding a subcommand forces you to handle it everywhere.
#[derive(Subcommand, Debug, PartialEq)]
pub enum TaskCommand {
    /// Add a new task
    Add {
        /// Short description of the task
        title: String,

        /// Priority from 1 (highest) to 5 (lowest)
        #[arg(
            short,
            long,
            default_value_t = 3,
            value_parser = clap::value_parser!(u8).range(1..=5)
        )]
        priority: u8,
    },
    /// List tasks
    List {
        /// Include completed tasks
        #[arg(long)]
        all: bool,
    },
    /// Mark a task as done
    Done {
        /// Task id to complete
        id: u32,
    },
}

/// Dispatch a parsed subcommand to a human-readable result.
///
/// Python equivalent: click calls the decorated function for you. In clap
/// you match on the enum — more explicit, and the compiler guarantees no
/// subcommand is forgotten.
pub fn dispatch(command: &TaskCommand) -> String {
    match command {
        TaskCommand::Add { title, priority } => {
            format!("added {title:?} at priority {priority}")
        }
        TaskCommand::List { all } => {
            if *all {
                "listing all tasks".to_string()
            } else {
                "listing open tasks".to_string()
            }
        }
        TaskCommand::Done { id } => format!("completed task {id}"),
    }
}

// ---------------------------------------------------------------------------
// 5. Structured output — a --json flag instead of print soup
// ---------------------------------------------------------------------------

/// Output formats a tool can speak.
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq)]
pub enum OutputFormat {
    /// Human-readable lines
    Text,
    /// Machine-readable JSON
    Json,
}

/// One task, as data — not as a pre-formatted string.
///
/// Python equivalent:
/// ```python
/// # The anti-pattern: print soup. Output IS the format.
/// print(f"task {id}: {title} {'(done)' if done else ''}")
///
/// # The fix: keep results as data, choose the rendering at the edge.
/// task = {"id": id, "title": title, "done": done}
/// print(json.dumps(task) if args.json else render_text(task))
/// ```
///
/// The same discipline in Rust: compute a `Vec<TaskRecord>`, then render it
/// once, at the edge, in the format the caller asked for. Scripts and other
/// programs consume `--json`; humans get text. One source of truth.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TaskRecord {
    pub id: u32,
    pub title: String,
    pub done: bool,
}

/// Render tasks in the requested format.
pub fn render_tasks(tasks: &[TaskRecord], format: OutputFormat) -> String {
    match format {
        OutputFormat::Text => tasks
            .iter()
            .map(|t| {
                let mark = if t.done { "x" } else { " " };
                format!("[{mark}] {} {}", t.id, t.title)
            })
            .collect::<Vec<_>>()
            .join("\n"),
        OutputFormat::Json => {
            serde_json::to_string(tasks).expect("serialization should not fail for valid types")
        }
    }
}

// ---------------------------------------------------------------------------
// 6. Help text as a first-class artifact
// ---------------------------------------------------------------------------

/// Render the long help for the `greet` tool.
///
/// Python equivalent:
/// ```python
/// parser.format_help()   # help strings passed as help="..." kwargs
/// ```
///
/// In clap derive, the doc comments you wrote on the struct and its fields
/// ARE the help text. Documentation and behavior live in one place, so they
/// cannot drift apart — the same comment serves rustdoc, your teammates,
/// and `--help`.
pub fn greet_help() -> String {
    Greet::command().render_long_help().to_string()
}

// ---------------------------------------------------------------------------
// 7. Exit codes — errors a shell can see
// ---------------------------------------------------------------------------

/// Errors a CLI run can produce, separated by who got it wrong.
///
/// Python equivalent:
/// ```python
/// # argparse calls sys.exit(2) on bad usage. Your own failures are
/// # whatever you remember to pass to sys.exit() — often nothing, so a
/// # failed run exits 0 and the calling script merrily continues.
/// ```
///
/// Convention: 0 = success, 1 = the operation failed, 2 = the user invoked
/// the tool incorrectly. Encoding the distinction in an enum means the
/// mapping to exit codes happens in exactly one place.
#[derive(Debug, PartialEq)]
pub enum CliError {
    /// The arguments were invalid — exit code 2.
    Usage(String),
    /// The arguments were fine but the operation failed — exit code 1.
    Runtime(String),
}

/// Run the greet tool end to end: parse, validate, produce output.
///
/// Testable by construction: `try_parse_from` parses from a slice instead
/// of the real process arguments, so the whole pipeline runs inside a unit
/// test — no subprocess required. (`Greet::parse()` is the same parser
/// reading the real `argv`; you'd call that in `main`.)
pub fn run_greet(argv: &[&str]) -> Result<String, CliError> {
    let args = match Greet::try_parse_from(argv) {
        Ok(args) => args,
        // --help and --version surface as "errors" from try_parse_from,
        // but they are successful runs: print and exit 0.
        Err(e)
            if e.kind() == clap::error::ErrorKind::DisplayHelp
                || e.kind() == clap::error::ErrorKind::DisplayVersion =>
        {
            return Ok(e.to_string());
        }
        Err(e) => return Err(CliError::Usage(e.to_string())),
    };

    // A rule the type system can't express — checked after parsing,
    // reported as a runtime failure rather than a usage error.
    if args.name.trim().is_empty() {
        return Err(CliError::Runtime("name must not be blank".to_string()));
    }

    Ok(greeting(&args))
}

/// Map a run result to a process exit code.
///
/// In `main` you would end with `std::process::exit(exit_code(&result))`.
pub fn exit_code<T>(result: &Result<T, CliError>) -> u8 {
    match result {
        Ok(_) => 0,
        Err(CliError::Runtime(_)) => 1,
        Err(CliError::Usage(_)) => 2,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Your first parser

    #[test]
    fn greet_parses_positional_and_defaults() {
        let args = Greet::try_parse_from(["greet", "Alice"]).unwrap();
        assert_eq!(args.name, "Alice");
        assert_eq!(args.count, 1);
        assert!(!args.loud);
    }

    #[test]
    fn greet_parses_short_and_long_flags() {
        let short = Greet::try_parse_from(["greet", "Bob", "-c", "2", "-l"]).unwrap();
        let long = Greet::try_parse_from(["greet", "Bob", "--count", "2", "--loud"]).unwrap();
        assert_eq!(short, long);
        assert_eq!(short.count, 2);
        assert!(short.loud);
    }

    #[test]
    fn greet_missing_name_is_a_parse_error() {
        let err = Greet::try_parse_from(["greet"]).unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn greet_rejects_non_numeric_count() {
        // argparse without type=int would happily hand you the string "lots".
        assert!(Greet::try_parse_from(["greet", "Alice", "--count", "lots"]).is_err());
    }

    #[test]
    fn greeting_repeats_and_shouts() {
        let args = Greet::try_parse_from(["greet", "Ada", "-c", "2", "--loud"]).unwrap();
        assert_eq!(greeting(&args), "HELLO, ADA!\nHELLO, ADA!");
    }

    // The parser is the type checker

    #[test]
    fn serve_defaults_are_typed() {
        let args = Serve::try_parse_from(["serve"]).unwrap();
        assert_eq!(args.port, 8080); // a real u16, not a string
        assert_eq!(args.root, std::path::PathBuf::from("."));
    }

    #[test]
    fn serve_accepts_valid_port() {
        let args = Serve::try_parse_from(["serve", "--port", "3000", "/srv"]).unwrap();
        assert_eq!(args.port, 3000);
        assert_eq!(args.root, std::path::PathBuf::from("/srv"));
    }

    #[test]
    fn serve_rejects_out_of_range_port() {
        // 99999 doesn't fit in u16, and 80 is below the declared range.
        assert!(Serve::try_parse_from(["serve", "--port", "99999"]).is_err());
        assert!(Serve::try_parse_from(["serve", "--port", "80"]).is_err());
    }

    #[test]
    fn serve_rejects_non_numeric_port() {
        assert!(Serve::try_parse_from(["serve", "--port", "http"]).is_err());
    }

    // Choices become enums

    #[test]
    fn level_parses_into_enum_variant() {
        let args = Logging::try_parse_from(["logdemo", "--level", "warn"]).unwrap();
        assert_eq!(args.level, LogLevel::Warn);
    }

    #[test]
    fn level_default_is_info() {
        let args = Logging::try_parse_from(["logdemo"]).unwrap();
        assert_eq!(args.level, LogLevel::Info);
    }

    #[test]
    fn level_rejects_unknown_choice() {
        assert!(Logging::try_parse_from(["logdemo", "--level", "loud"]).is_err());
    }

    #[test]
    fn severity_orders_levels() {
        assert!(LogLevel::Error.severity() > LogLevel::Debug.severity());
    }

    // Subcommands

    #[test]
    fn add_subcommand_parses() {
        let cli = TasksCli::try_parse_from(["tasks", "add", "write tests", "-p", "1"]).unwrap();
        assert_eq!(
            cli.command,
            TaskCommand::Add {
                title: "write tests".to_string(),
                priority: 1,
            }
        );
    }

    #[test]
    fn add_priority_validated_at_parse_time() {
        // click.IntRange equivalent — but enforced by the type's parser.
        assert!(TasksCli::try_parse_from(["tasks", "add", "x", "--priority", "9"]).is_err());
    }

    #[test]
    fn list_subcommand_parses() {
        let cli = TasksCli::try_parse_from(["tasks", "list", "--all"]).unwrap();
        assert_eq!(cli.command, TaskCommand::List { all: true });
    }

    #[test]
    fn done_subcommand_parses_typed_id() {
        let cli = TasksCli::try_parse_from(["tasks", "done", "42"]).unwrap();
        assert_eq!(cli.command, TaskCommand::Done { id: 42 });
    }

    #[test]
    fn global_json_flag_works_after_subcommand() {
        let cli = TasksCli::try_parse_from(["tasks", "list", "--json"]).unwrap();
        assert!(cli.json);
    }

    #[test]
    fn dispatch_covers_every_subcommand() {
        let cmd = TaskCommand::Add {
            title: "ship it".to_string(),
            priority: 2,
        };
        assert_eq!(dispatch(&cmd), "added \"ship it\" at priority 2");
        assert_eq!(
            dispatch(&TaskCommand::List { all: false }),
            "listing open tasks"
        );
        assert_eq!(dispatch(&TaskCommand::Done { id: 7 }), "completed task 7");
    }

    // Structured output

    #[test]
    fn render_text_is_human_readable() {
        let tasks = vec![
            TaskRecord {
                id: 1,
                title: "write chapter".to_string(),
                done: true,
            },
            TaskRecord {
                id: 2,
                title: "review chapter".to_string(),
                done: false,
            },
        ];
        let text = render_tasks(&tasks, OutputFormat::Text);
        assert_eq!(text, "[x] 1 write chapter\n[ ] 2 review chapter");
    }

    #[test]
    fn render_json_is_machine_readable() {
        let tasks = vec![TaskRecord {
            id: 1,
            title: "write chapter".to_string(),
            done: true,
        }];
        let json = render_tasks(&tasks, OutputFormat::Json);

        // Another program can parse this back — that's the point.
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed[0]["id"], 1);
        assert_eq!(parsed[0]["title"], "write chapter");
        assert_eq!(parsed[0]["done"], true);
    }

    // Help text as a first-class artifact

    #[test]
    fn doc_comments_become_help_text() {
        let help = greet_help();
        assert!(help.contains("Name of the person to greet"));
        assert!(help.contains("Number of times to repeat the greeting"));
        assert!(help.contains("--loud"));
    }

    #[test]
    fn subcommands_appear_in_help() {
        let help = TasksCli::command().render_long_help().to_string();
        assert!(help.contains("Add a new task"));
        assert!(help.contains("Mark a task as done"));
    }

    // Exit codes

    #[test]
    fn successful_run_exits_zero() {
        let result = run_greet(&["greet", "Alice"]);
        assert_eq!(result, Ok("Hello, Alice!".to_string()));
        assert_eq!(exit_code(&result), 0);
    }

    #[test]
    fn usage_error_exits_two() {
        let result = run_greet(&["greet", "--count", "nope", "Alice"]);
        assert!(matches!(result, Err(CliError::Usage(_))));
        assert_eq!(exit_code(&result), 2);
    }

    #[test]
    fn runtime_error_exits_one() {
        let result = run_greet(&["greet", "   "]);
        assert_eq!(
            result,
            Err(CliError::Runtime("name must not be blank".to_string()))
        );
        assert_eq!(exit_code(&result), 1);
    }

    #[test]
    fn help_is_success_not_failure() {
        let result = run_greet(&["greet", "--help"]);
        assert!(result.is_ok());
        assert_eq!(exit_code(&result), 0);
        assert!(result.unwrap().contains("Greet someone"));
    }
}
