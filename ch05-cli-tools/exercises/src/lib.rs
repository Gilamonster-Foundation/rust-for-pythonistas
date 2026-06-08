//! # Chapter 5 Exercises: CLI Tools
//!
//! Each exercise shows a Python snippet and asks you to write the Rust
//! equivalent. Replace the `todo!()` markers with working code.
//!
//! Run tests: `cargo test -p ch05-exercises`

// These allows are intentional: exercise stubs have unused parameters
// and fields until the student fills in the todo!() markers.
#![allow(unused_variables, dead_code)]

use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;

// ============================================================
// Exercise 1: From Parsed Args to Output
// ============================================================
//
// Python version:
// ```python
// import argparse
//
// parser = argparse.ArgumentParser(prog="shout")
// parser.add_argument("words", nargs="+")
// parser.add_argument("-r", "--repeat", type=int, default=1)
// args = parser.parse_args()
//
// line = " ".join(args.words).upper() + "!"
// print("\n".join([line] * args.repeat))
// ```
//
// The parser struct is provided — read it carefully; the field types and
// attributes ARE the argument definitions. Implement `run_shout`:
// join the words with spaces, uppercase them, append "!", and repeat
// the line `repeat` times joined by newlines.

/// Shout some words.
#[derive(Parser, Debug)]
#[command(name = "shout")]
pub struct Shout {
    /// Words to shout (at least one)
    #[arg(required = true)]
    pub words: Vec<String>,

    /// Number of times to repeat the line
    #[arg(short, long, default_value_t = 1)]
    pub repeat: u8,
}

pub fn run_shout(args: &Shout) -> String {
    todo!("Join words with spaces, uppercase, add '!', repeat with newlines")
}

// ============================================================
// Exercise 2: The Builder API
// ============================================================
//
// Python version:
// ```python
// parser = argparse.ArgumentParser(prog="copy")
// parser.add_argument("source", help="File to copy")
// parser.add_argument("dest", help="Where to copy it")
// parser.add_argument("-f", "--force", action="store_true",
//                     help="Overwrite the destination if it exists")
// ```
//
// The derive API is sugar over clap's builder API — which looks a lot
// like argparse. Build the same parser by hand:
// - a required positional arg "source"
// - a required positional arg "dest"
// - a "force" flag with short 'f' and long "force"
//
// Hints (use full paths, no extra imports needed):
//   clap::Command::new("copy")
//   .arg(clap::Arg::new("source").required(true))
//   flags use .action(clap::ArgAction::SetTrue)

pub fn build_copy_command() -> clap::Command {
    todo!("Build the `copy` command with source, dest, and --force")
}

// ============================================================
// Exercise 3: Subcommand Dispatch
// ============================================================
//
// Python version (click):
// ```python
// @click.group()
// def notes(): ...
//
// @notes.command()
// @click.argument("text")
// def add(text):
//     click.echo(f"added note: {text}")
//
// @notes.command(name="list")
// @click.option("--limit", type=int)
// def list_(limit):
//     if limit is None:
//         click.echo("listing all notes")
//     else:
//         click.echo(f"listing {limit} notes")
//
// @notes.command()
// @click.argument("id", type=int)
// def delete(id):
//     click.echo(f"deleted note {id}")
// ```
//
// The enum is provided. Implement `describe` with a match over the
// variants, producing exactly the strings click would echo above.

#[derive(Parser, Debug)]
#[command(name = "notes")]
pub struct NotesCli {
    #[command(subcommand)]
    pub command: NotesCommand,
}

#[derive(Subcommand, Debug, PartialEq)]
pub enum NotesCommand {
    /// Add a note
    Add {
        /// The note text
        text: String,
    },
    /// List notes
    List {
        /// Show at most this many notes
        #[arg(long)]
        limit: Option<u32>,
    },
    /// Delete a note
    Delete {
        /// Note id to delete
        id: u32,
    },
}

pub fn describe(command: &NotesCommand) -> String {
    todo!("Match each variant: 'added note: TEXT', 'listing all notes' / 'listing N notes', 'deleted note ID'")
}

// ============================================================
// Exercise 4: Structured Output
// ============================================================
//
// Python version:
// ```python
// def render(packages, as_json):
//     if as_json:
//         return json.dumps(packages)   # list of dicts
//     lines = []
//     for p in packages:
//         suffix = " (installed)" if p["installed"] else ""
//         lines.append(f"{p['name']} {p['version']}{suffix}")
//     return "\n".join(lines)
// ```
//
// Keep results as data; render once, at the edge. Implement `render`:
// - Text:  one line per package: "NAME VERSION" plus " (installed)"
//          if installed, joined with newlines
// - Json:  serde_json::to_string of the whole slice

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq)]
pub enum OutputFormat {
    Text,
    Json,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub installed: bool,
}

impl Package {
    pub fn new(name: &str, version: &str, installed: bool) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            installed,
        }
    }
}

pub fn render(packages: &[Package], format: OutputFormat) -> String {
    todo!("Text: lines like 'name version (installed)'; Json: serde_json::to_string")
}

// ============================================================
// Exercise 5: Exit Codes
// ============================================================
//
// Python version:
// ```python
// import sys
//
// PROTECTED = {"/", ""}
//
// def main():
//     args = parser.parse_args()        # argparse exits 2 on bad usage
//     if args.target in PROTECTED:
//         print("refusing to remove protected path", file=sys.stderr)
//         sys.exit(1)                   # runtime failure
//     verb = "would remove" if args.dry_run else "removed"
//     print(f"{verb} {args.target}")
//     sys.exit(0)
// ```
//
// Implement `run_remove` and `exit_code`:
// - Parse with `Remove::try_parse_from(argv)`; on a parse error return
//   Err(CliError::Usage) (don't worry about --help here)
// - If target is "/" or "" return Err(CliError::Refused)
// - Otherwise Ok("would remove TARGET") when dry_run, Ok("removed TARGET")
//   when not
// - exit_code: Ok = 0, Refused = 1, Usage = 2

/// Remove a file (pretend).
#[derive(Parser, Debug)]
#[command(name = "remove")]
pub struct Remove {
    /// Path to remove
    pub target: String,

    /// Show what would happen without doing it
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Debug, PartialEq)]
pub enum CliError {
    /// Bad arguments — exit 2.
    Usage,
    /// Refused to operate on a protected path — exit 1.
    Refused,
}

pub fn run_remove(argv: &[&str]) -> Result<String, CliError> {
    todo!("Parse argv, refuse protected targets, honor --dry-run")
}

pub fn exit_code(result: &Result<String, CliError>) -> u8 {
    todo!("Ok = 0, Refused = 1, Usage = 2")
}

// ============================================================
// Tests — do not modify below this line
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Exercise 1
    #[test]
    fn ex1_shout_one_word() {
        let args = Shout::try_parse_from(["shout", "hello"]).unwrap();
        assert_eq!(run_shout(&args), "HELLO!");
    }

    #[test]
    fn ex1_shout_joins_words() {
        let args = Shout::try_parse_from(["shout", "hello", "world"]).unwrap();
        assert_eq!(run_shout(&args), "HELLO WORLD!");
    }

    #[test]
    fn ex1_shout_repeats() {
        let args = Shout::try_parse_from(["shout", "-r", "3", "go"]).unwrap();
        assert_eq!(run_shout(&args), "GO!\nGO!\nGO!");
    }

    // Exercise 2
    #[test]
    fn ex2_copy_parses_positionals() {
        let matches = build_copy_command()
            .try_get_matches_from(["copy", "a.txt", "b.txt"])
            .unwrap();
        assert_eq!(
            matches.get_one::<String>("source").map(String::as_str),
            Some("a.txt")
        );
        assert_eq!(
            matches.get_one::<String>("dest").map(String::as_str),
            Some("b.txt")
        );
        assert!(!matches.get_flag("force"));
    }

    #[test]
    fn ex2_copy_requires_dest() {
        assert!(build_copy_command()
            .try_get_matches_from(["copy", "only-source.txt"])
            .is_err());
    }

    #[test]
    fn ex2_copy_force_flag() {
        let long = build_copy_command()
            .try_get_matches_from(["copy", "a", "b", "--force"])
            .unwrap();
        assert!(long.get_flag("force"));

        let short = build_copy_command()
            .try_get_matches_from(["copy", "a", "b", "-f"])
            .unwrap();
        assert!(short.get_flag("force"));
    }

    // Exercise 3
    #[test]
    fn ex3_add_describes() {
        let cli = NotesCli::try_parse_from(["notes", "add", "buy milk"]).unwrap();
        assert_eq!(describe(&cli.command), "added note: buy milk");
    }

    #[test]
    fn ex3_list_without_limit() {
        let cli = NotesCli::try_parse_from(["notes", "list"]).unwrap();
        assert_eq!(describe(&cli.command), "listing all notes");
    }

    #[test]
    fn ex3_list_with_limit() {
        let cli = NotesCli::try_parse_from(["notes", "list", "--limit", "5"]).unwrap();
        assert_eq!(describe(&cli.command), "listing 5 notes");
    }

    #[test]
    fn ex3_delete_describes() {
        let cli = NotesCli::try_parse_from(["notes", "delete", "12"]).unwrap();
        assert_eq!(describe(&cli.command), "deleted note 12");
    }

    // Exercise 4
    #[test]
    fn ex4_text_output() {
        let packages = vec![
            Package::new("serde", "1.0.200", true),
            Package::new("clap", "4.5.0", false),
        ];
        assert_eq!(
            render(&packages, OutputFormat::Text),
            "serde 1.0.200 (installed)\nclap 4.5.0"
        );
    }

    #[test]
    fn ex4_json_output_is_parseable() {
        let packages = vec![Package::new("serde", "1.0.200", true)];
        let json = render(&packages, OutputFormat::Json);

        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed[0]["name"], "serde");
        assert_eq!(parsed[0]["version"], "1.0.200");
        assert_eq!(parsed[0]["installed"], true);
    }

    #[test]
    fn ex4_json_empty_list() {
        assert_eq!(render(&[], OutputFormat::Json), "[]");
    }

    // Exercise 5
    #[test]
    fn ex5_remove_success() {
        let result = run_remove(&["remove", "old.log"]);
        assert_eq!(result, Ok("removed old.log".to_string()));
        assert_eq!(exit_code(&result), 0);
    }

    #[test]
    fn ex5_dry_run() {
        let result = run_remove(&["remove", "old.log", "--dry-run"]);
        assert_eq!(result, Ok("would remove old.log".to_string()));
    }

    #[test]
    fn ex5_refuses_protected_path() {
        let result = run_remove(&["remove", "/"]);
        assert_eq!(result, Err(CliError::Refused));
        assert_eq!(exit_code(&result), 1);
    }

    #[test]
    fn ex5_usage_error() {
        let result = run_remove(&["remove"]); // missing target
        assert_eq!(result, Err(CliError::Usage));
        assert_eq!(exit_code(&result), 2);
    }
}
