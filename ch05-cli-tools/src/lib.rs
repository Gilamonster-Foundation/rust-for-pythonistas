//! # Chapter 5: CLI Tools
//!
//! This module demonstrates building CLI tools in Rust using clap for
//! argument parsing and serde for structured output.
//!
//! Run the tests: `cargo test -p ch05-cli-tools`

use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// 1. Declarative argument parsing with clap derive
// ---------------------------------------------------------------------------

/// A note-taking CLI — manages a list of notes with tags.
///
/// Python equivalent:
/// ```python
/// parser = argparse.ArgumentParser(description="A simple note-taking tool")
/// subparsers = parser.add_subparsers(dest="command")
///
/// add = subparsers.add_parser("add")
/// add.add_argument("text", help="The note content")
/// add.add_argument("--tag", "-t", action="append", default=[])
///
/// list = subparsers.add_parser("list")
/// list.add_argument("--tag", help="Filter by tag")
/// list.add_argument("--format", choices=["text", "json"], default="text")
///
/// search = subparsers.add_parser("search")
/// search.add_argument("query")
/// ```
#[derive(Parser, Debug)]
#[command(name = "notes", about = "A simple note-taking tool")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Add a new note
    Add {
        /// The note content
        text: String,

        /// Tags for the note (can be repeated)
        #[arg(long, short)]
        tag: Vec<String>,
    },

    /// List all notes
    List {
        /// Filter by tag
        #[arg(long)]
        tag: Option<String>,

        /// Output format
        #[arg(long, value_enum, default_value = "text")]
        format: OutputFormat,
    },

    /// Search notes by content
    Search {
        /// Search query (case-insensitive substring match)
        query: String,
    },
}

// ---------------------------------------------------------------------------
// 2. Output format as an enum
// ---------------------------------------------------------------------------

/// Output format — derived from clap's ValueEnum.
///
/// Python equivalent:
/// ```python
/// choices=["text", "json"]
/// ```
///
/// In Rust, this is a type — you can't pass an invalid format.
#[derive(ValueEnum, Debug, Clone, Copy, PartialEq)]
pub enum OutputFormat {
    Text,
    Json,
}

// ---------------------------------------------------------------------------
// 3. The data model — serializable for structured output
// ---------------------------------------------------------------------------

/// A single note with content and tags.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Note {
    pub id: usize,
    pub text: String,
    pub tags: Vec<String>,
}

/// An in-memory note store.
///
/// Python equivalent:
/// ```python
/// class NoteStore:
///     def __init__(self):
///         self.notes = []
///         self.next_id = 1
/// ```
pub struct NoteStore {
    notes: Vec<Note>,
    next_id: usize,
}

impl NoteStore {
    pub fn new() -> Self {
        Self {
            notes: Vec::new(),
            next_id: 1,
        }
    }

    /// Add a note and return its ID.
    pub fn add(&mut self, text: &str, tags: Vec<String>) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.notes.push(Note {
            id,
            text: text.to_string(),
            tags,
        });
        id
    }

    /// List all notes, optionally filtered by tag.
    pub fn list(&self, tag_filter: Option<&str>) -> Vec<&Note> {
        self.notes
            .iter()
            .filter(|note| {
                tag_filter
                    .map(|tag| note.tags.iter().any(|t| t == tag))
                    .unwrap_or(true)
            })
            .collect()
    }

    /// Search notes by substring (case-insensitive).
    pub fn search(&self, query: &str) -> Vec<&Note> {
        let query_lower = query.to_lowercase();
        self.notes
            .iter()
            .filter(|note| note.text.to_lowercase().contains(&query_lower))
            .collect()
    }

    /// Total number of notes.
    pub fn len(&self) -> usize {
        self.notes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.notes.is_empty()
    }
}

impl Default for NoteStore {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// 4. Structured output — one function, multiple formats
// ---------------------------------------------------------------------------

/// Format notes for output.
///
/// Python equivalent:
/// ```python
/// def format_notes(notes, fmt):
///     if fmt == "json":
///         return json.dumps([n.__dict__ for n in notes], indent=2)
///     return "\n".join(f"[{n.id}] {n.text} {n.tags}" for n in notes)
/// ```
///
/// The Rust version uses serde Serialize — any type that implements
/// Serialize gets JSON output for free.
pub fn format_notes(notes: &[&Note], format: OutputFormat) -> String {
    match format {
        OutputFormat::Text => notes
            .iter()
            .map(|note| {
                let tags = if note.tags.is_empty() {
                    String::new()
                } else {
                    format!(" [{}]", note.tags.join(", "))
                };
                format!("#{}: {}{}", note.id, note.text, tags)
            })
            .collect::<Vec<_>>()
            .join("\n"),
        OutputFormat::Json => {
            serde_json::to_string_pretty(notes).expect("note serialization should not fail")
        }
    }
}

// ---------------------------------------------------------------------------
// 5. Command dispatch with exhaustive matching
// ---------------------------------------------------------------------------

/// Process a command and return the output string.
///
/// Python equivalent:
/// ```python
/// def dispatch(store, args):
///     if args.command == "add":
///         id = store.add(args.text, args.tag)
///         return f"Added note #{id}"
///     elif args.command == "list":
///         notes = store.list(args.tag)
///         return format_notes(notes, args.format)
///     elif args.command == "search":
///         results = store.search(args.query)
///         return format_notes(results, "text")
/// ```
///
/// The Rust match is exhaustive — add a new Command variant and the
/// compiler will tell you to handle it here.
pub fn dispatch(store: &mut NoteStore, command: Command) -> String {
    match command {
        Command::Add { text, tag } => {
            let id = store.add(&text, tag);
            format!("Added note #{id}")
        }
        Command::List { tag, format } => {
            let notes = store.list(tag.as_deref());
            format_notes(&notes, format)
        }
        Command::Search { query } => {
            let results = store.search(&query);
            format_notes(&results, OutputFormat::Text)
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // CLI parsing tests — verify the derive macros work

    #[test]
    fn parse_add_command() {
        let cli = Cli::parse_from(["notes", "add", "Hello world", "--tag", "greeting"]);
        match cli.command {
            Command::Add { text, tag } => {
                assert_eq!(text, "Hello world");
                assert_eq!(tag, vec!["greeting"]);
            }
            _ => panic!("expected Add command"),
        }
    }

    #[test]
    fn parse_add_multiple_tags() {
        let cli = Cli::parse_from(["notes", "add", "Note", "-t", "a", "-t", "b"]);
        match cli.command {
            Command::Add { tag, .. } => {
                assert_eq!(tag, vec!["a", "b"]);
            }
            _ => panic!("expected Add command"),
        }
    }

    #[test]
    fn parse_list_default_format() {
        let cli = Cli::parse_from(["notes", "list"]);
        match cli.command {
            Command::List { format, tag } => {
                assert_eq!(format, OutputFormat::Text);
                assert!(tag.is_none());
            }
            _ => panic!("expected List command"),
        }
    }

    #[test]
    fn parse_list_json_format() {
        let cli = Cli::parse_from(["notes", "list", "--format", "json"]);
        match cli.command {
            Command::List { format, .. } => {
                assert_eq!(format, OutputFormat::Json);
            }
            _ => panic!("expected List command"),
        }
    }

    #[test]
    fn parse_search() {
        let cli = Cli::parse_from(["notes", "search", "hello"]);
        match cli.command {
            Command::Search { query } => {
                assert_eq!(query, "hello");
            }
            _ => panic!("expected Search command"),
        }
    }

    // Store tests

    #[test]
    fn store_add_and_list() {
        let mut store = NoteStore::new();
        store.add("First note", vec![]);
        store.add("Second note", vec!["important".to_string()]);

        assert_eq!(store.len(), 2);
        assert_eq!(store.list(None).len(), 2);
    }

    #[test]
    fn store_filter_by_tag() {
        let mut store = NoteStore::new();
        store.add("Buy milk", vec!["shopping".to_string()]);
        store.add("Fix bug", vec!["work".to_string()]);
        store.add("Buy eggs", vec!["shopping".to_string()]);

        let shopping = store.list(Some("shopping"));
        assert_eq!(shopping.len(), 2);
        assert!(shopping
            .iter()
            .all(|n| n.tags.contains(&"shopping".to_string())));
    }

    #[test]
    fn store_search_case_insensitive() {
        let mut store = NoteStore::new();
        store.add("Hello World", vec![]);
        store.add("Goodbye world", vec![]);
        store.add("No match", vec![]);

        let results = store.search("world");
        assert_eq!(results.len(), 2);
    }

    // Output format tests

    #[test]
    fn format_text_output() {
        let note = Note {
            id: 1,
            text: "Hello".to_string(),
            tags: vec!["greeting".to_string()],
        };
        let output = format_notes(&[&note], OutputFormat::Text);
        assert_eq!(output, "#1: Hello [greeting]");
    }

    #[test]
    fn format_text_no_tags() {
        let note = Note {
            id: 1,
            text: "Hello".to_string(),
            tags: vec![],
        };
        let output = format_notes(&[&note], OutputFormat::Text);
        assert_eq!(output, "#1: Hello");
    }

    #[test]
    fn format_json_output() {
        let note = Note {
            id: 1,
            text: "Hello".to_string(),
            tags: vec!["greeting".to_string()],
        };
        let output = format_notes(&[&note], OutputFormat::Json);
        let parsed: Vec<Note> = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].text, "Hello");
    }

    // Dispatch tests

    #[test]
    fn dispatch_add() {
        let mut store = NoteStore::new();
        let result = dispatch(
            &mut store,
            Command::Add {
                text: "Test note".to_string(),
                tag: vec!["test".to_string()],
            },
        );
        assert_eq!(result, "Added note #1");
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn dispatch_list() {
        let mut store = NoteStore::new();
        store.add("Note A", vec![]);
        store.add("Note B", vec![]);

        let result = dispatch(
            &mut store,
            Command::List {
                tag: None,
                format: OutputFormat::Text,
            },
        );
        assert!(result.contains("#1: Note A"));
        assert!(result.contains("#2: Note B"));
    }

    #[test]
    fn dispatch_search() {
        let mut store = NoteStore::new();
        store.add("Buy groceries", vec![]);
        store.add("Fix the car", vec![]);

        let result = dispatch(
            &mut store,
            Command::Search {
                query: "buy".to_string(),
            },
        );
        assert!(result.contains("Buy groceries"));
        assert!(!result.contains("Fix the car"));
    }
}
