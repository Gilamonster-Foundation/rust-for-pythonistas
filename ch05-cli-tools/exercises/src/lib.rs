//! # Chapter 5 Exercises: CLI Tools
//!
//! These exercises build a task manager CLI from scratch.
//! Each exercise adds a new capability.
//!
//! Run tests: `cargo test -p ch05-exercises`

#![allow(unused_variables, dead_code)]

use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;

// ============================================================
// Exercise 1: Define a CLI with Subcommands
// ============================================================
//
// Python version:
// ```python
// parser = argparse.ArgumentParser(description="Task manager")
// sub = parser.add_subparsers(dest="command")
//
// add = sub.add_parser("add")
// add.add_argument("description")
// add.add_argument("--priority", type=int, default=3)
//
// done = sub.add_parser("done")
// done.add_argument("id", type=int)
//
// list = sub.add_parser("list")
// list.add_argument("--status", choices=["all", "open", "done"], default="all")
// ```
//
// Define a TaskCli struct with subcommands: Add, Done, List.
// The Add command takes a description (String) and optional --priority (u8, default 3).
// The Done command takes an id (usize).
// The List command takes an optional --status filter (TaskStatus enum).

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq)]
pub enum TaskStatusFilter {
    All,
    Open,
    Done,
}

#[derive(Parser, Debug)]
#[command(name = "tasks", about = "A simple task manager")]
pub struct TaskCli {
    #[command(subcommand)]
    pub command: TaskCommand,
}

#[derive(Subcommand, Debug)]
pub enum TaskCommand {
    /// Add a new task
    Add {
        /// Task description
        description: String,

        /// Priority (1=highest, 5=lowest)
        #[arg(long, default_value = "3")]
        priority: u8,
    },

    /// Mark a task as done
    Done {
        /// Task ID to mark as done
        id: usize,
    },

    /// List tasks
    List {
        /// Filter by status
        #[arg(long, value_enum, default_value = "all")]
        status: TaskStatusFilter,
    },
}

// ============================================================
// Exercise 2: Data Model
// ============================================================
//
// Python version:
// ```python
// @dataclass
// class Task:
//     id: int
//     description: str
//     priority: int
//     done: bool = False
// ```
//
// Define a Task struct that derives Debug, Clone, Serialize, PartialEq.

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Task {
    pub id: usize,
    pub description: String,
    pub priority: u8,
    pub done: bool,
}

// ============================================================
// Exercise 3: Task Store with Filtering
// ============================================================
//
// Python version:
// ```python
// class TaskStore:
//     def __init__(self):
//         self.tasks = []
//         self.next_id = 1
//
//     def add(self, description, priority=3):
//         task = Task(self.next_id, description, priority)
//         self.next_id += 1
//         self.tasks.append(task)
//         return task.id
//
//     def mark_done(self, id):
//         for task in self.tasks:
//             if task.id == id:
//                 task.done = True
//                 return True
//         return False
//
//     def list(self, status="all"):
//         if status == "all":
//             return sorted(self.tasks, key=lambda t: t.priority)
//         elif status == "open":
//             return sorted([t for t in self.tasks if not t.done], key=lambda t: t.priority)
//         elif status == "done":
//             return sorted([t for t in self.tasks if t.done], key=lambda t: t.priority)
// ```

pub struct TaskStore {
    tasks: Vec<Task>,
    next_id: usize,
}

impl TaskStore {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            next_id: 1,
        }
    }

    pub fn add(&mut self, description: &str, priority: u8) -> usize {
        todo!("Create a task, push it, return the ID")
    }

    /// Mark a task as done. Returns true if the task was found.
    pub fn mark_done(&mut self, id: usize) -> bool {
        todo!("Find task by id, set done=true, return whether it was found")
    }

    /// List tasks filtered by status, sorted by priority (lowest number first).
    pub fn list(&self, filter: TaskStatusFilter) -> Vec<&Task> {
        todo!("Filter by status, sort by priority ascending")
    }

    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }
}

impl Default for TaskStore {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// Exercise 4: Structured Output
// ============================================================
//
// Python version:
// ```python
// def format_tasks(tasks, fmt="text"):
//     if fmt == "json":
//         return json.dumps([t.__dict__ for t in tasks], indent=2)
//     lines = []
//     for t in tasks:
//         status = "x" if t.done else " "
//         lines.append(f"[{status}] #{t.id} (P{t.priority}) {t.description}")
//     return "\n".join(lines)
// ```
//
// Implement format_tasks for text and JSON output.
// Text format: "[x] #1 (P2) Buy milk" or "[ ] #2 (P3) Fix bug"

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq)]
pub enum OutputFormat {
    Text,
    Json,
}

pub fn format_tasks(tasks: &[&Task], format: OutputFormat) -> String {
    todo!("Format tasks as text or JSON")
}

// ============================================================
// Exercise 5: Command Dispatch
// ============================================================
//
// Python version:
// ```python
// def dispatch(store, args):
//     if args.command == "add":
//         id = store.add(args.description, args.priority)
//         return f"Added task #{id}"
//     elif args.command == "done":
//         if store.mark_done(args.id):
//             return f"Marked #{args.id} as done"
//         return f"Task #{args.id} not found"
//     elif args.command == "list":
//         tasks = store.list(args.status)
//         return format_tasks(tasks, "text")
// ```
//
// Implement dispatch using match on the TaskCommand enum.
// Return the output string.

pub fn dispatch(store: &mut TaskStore, command: TaskCommand) -> String {
    todo!("Match on command and dispatch to store methods")
}

// ============================================================
// Tests — do not modify below this line
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Exercise 1 — CLI parsing
    #[test]
    fn ex1_parse_add() {
        let cli = TaskCli::parse_from(["tasks", "add", "Buy milk", "--priority", "1"]);
        match cli.command {
            TaskCommand::Add {
                description,
                priority,
            } => {
                assert_eq!(description, "Buy milk");
                assert_eq!(priority, 1);
            }
            _ => panic!("expected Add"),
        }
    }

    #[test]
    fn ex1_parse_add_default_priority() {
        let cli = TaskCli::parse_from(["tasks", "add", "Something"]);
        match cli.command {
            TaskCommand::Add { priority, .. } => assert_eq!(priority, 3),
            _ => panic!("expected Add"),
        }
    }

    #[test]
    fn ex1_parse_done() {
        let cli = TaskCli::parse_from(["tasks", "done", "42"]);
        match cli.command {
            TaskCommand::Done { id } => assert_eq!(id, 42),
            _ => panic!("expected Done"),
        }
    }

    #[test]
    fn ex1_parse_list_filter() {
        let cli = TaskCli::parse_from(["tasks", "list", "--status", "open"]);
        match cli.command {
            TaskCommand::List { status } => assert_eq!(status, TaskStatusFilter::Open),
            _ => panic!("expected List"),
        }
    }

    // Exercise 3 — store
    #[test]
    fn ex3_add_and_list() {
        let mut store = TaskStore::new();
        store.add("Task A", 3);
        store.add("Task B", 1);

        let all = store.list(TaskStatusFilter::All);
        assert_eq!(all.len(), 2);
        // Should be sorted by priority: B (P1) before A (P3)
        assert_eq!(all[0].description, "Task B");
        assert_eq!(all[1].description, "Task A");
    }

    #[test]
    fn ex3_mark_done() {
        let mut store = TaskStore::new();
        let id = store.add("Task", 3);
        assert!(store.mark_done(id));
        assert!(!store.mark_done(999)); // not found

        let done = store.list(TaskStatusFilter::Done);
        assert_eq!(done.len(), 1);

        let open = store.list(TaskStatusFilter::Open);
        assert_eq!(open.len(), 0);
    }

    // Exercise 4 — output formatting
    #[test]
    fn ex4_text_format() {
        let task = Task {
            id: 1,
            description: "Buy milk".to_string(),
            priority: 2,
            done: false,
        };
        let output = format_tasks(&[&task], OutputFormat::Text);
        assert_eq!(output, "[ ] #1 (P2) Buy milk");
    }

    #[test]
    fn ex4_text_done() {
        let task = Task {
            id: 1,
            description: "Done task".to_string(),
            priority: 1,
            done: true,
        };
        let output = format_tasks(&[&task], OutputFormat::Text);
        assert_eq!(output, "[x] #1 (P1) Done task");
    }

    #[test]
    fn ex4_json_format() {
        let task = Task {
            id: 1,
            description: "Buy milk".to_string(),
            priority: 2,
            done: false,
        };
        let output = format_tasks(&[&task], OutputFormat::Json);
        let parsed: Vec<Task> = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed[0].description, "Buy milk");
    }

    // Exercise 5 — dispatch
    #[test]
    fn ex5_dispatch_add() {
        let mut store = TaskStore::new();
        let result = dispatch(
            &mut store,
            TaskCommand::Add {
                description: "Test".to_string(),
                priority: 2,
            },
        );
        assert_eq!(result, "Added task #1");
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn ex5_dispatch_done() {
        let mut store = TaskStore::new();
        store.add("Task", 3);

        let result = dispatch(&mut store, TaskCommand::Done { id: 1 });
        assert_eq!(result, "Marked #1 as done");

        let result = dispatch(&mut store, TaskCommand::Done { id: 999 });
        assert_eq!(result, "Task #999 not found");
    }

    #[test]
    fn ex5_dispatch_list() {
        let mut store = TaskStore::new();
        store.add("A", 3);
        store.add("B", 1);

        let result = dispatch(
            &mut store,
            TaskCommand::List {
                status: TaskStatusFilter::All,
            },
        );
        // B should come first (P1 < P3)
        assert!(result.starts_with("[ ] #2"));
    }
}
