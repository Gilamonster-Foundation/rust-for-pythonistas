//! # Chapter 3: Traits & Generics
//!
//! This module demonstrates Rust's trait system through examples that
//! map to familiar Python patterns.
//!
//! Run the tests: `cargo test -p ch03-traits-and-generics`

use std::fmt;

// ---------------------------------------------------------------------------
// 1. Defining and implementing traits
// ---------------------------------------------------------------------------

/// A trait for things that can summarize themselves in one line.
///
/// Python equivalent:
/// ```python
/// class Summarizable(Protocol):
///     def summary(self) -> str: ...
/// ```
pub trait Summarizable {
    fn summary(&self) -> String;
}

/// A blog post.
#[derive(Debug, Clone)]
pub struct BlogPost {
    pub title: String,
    pub author: String,
    pub word_count: usize,
}

/// A code snippet.
#[derive(Debug, Clone)]
pub struct CodeSnippet {
    pub language: String,
    pub lines: usize,
}

impl Summarizable for BlogPost {
    fn summary(&self) -> String {
        format!(
            "\"{}\" by {} ({} words)",
            self.title, self.author, self.word_count
        )
    }
}

impl Summarizable for CodeSnippet {
    fn summary(&self) -> String {
        format!("{} snippet ({} lines)", self.language, self.lines)
    }
}

// ---------------------------------------------------------------------------
// 2. Trait bounds and generics
// ---------------------------------------------------------------------------

/// Print the summary of anything Summarizable.
///
/// Python equivalent:
/// ```python
/// def print_summary(item: Summarizable) -> str:
///     return f">> {item.summary()}"
/// ```
///
/// The `impl Summarizable` syntax is sugar for `<T: Summarizable>`.
/// The compiler generates a specialized version for each type you call
/// this with — no vtable, no runtime cost.
pub fn format_summary(item: &impl Summarizable) -> String {
    format!(">> {}", item.summary())
}

/// Find the item with the longest summary.
///
/// This shows a more complex trait bound: T must be both Summarizable
/// and Clone (because we need to return an owned copy).
///
/// Python equivalent:
/// ```python
/// def longest_summary(items: list[Summarizable]) -> Summarizable:
///     return max(items, key=lambda x: len(x.summary()))
/// ```
pub fn longest_summary<T: Summarizable + Clone>(items: &[T]) -> Option<T> {
    items
        .iter()
        .max_by_key(|item| item.summary().len())
        .cloned()
}

// ---------------------------------------------------------------------------
// 3. Default methods
// ---------------------------------------------------------------------------

/// A trait with a required method and a default method.
///
/// Python equivalent:
/// ```python
/// class Labeled:
///     def label(self) -> str:
///         raise NotImplementedError
///
///     def display_label(self) -> str:
///         return f"[{self.label()}]"  # default uses label()
/// ```
pub trait Labeled {
    /// Required — implementors must provide this.
    fn label(&self) -> &str;

    /// Default — implementors get this for free, but can override it.
    fn display_label(&self) -> String {
        format!("[{}]", self.label())
    }
}

#[derive(Debug)]
pub struct Tag {
    name: String,
}

impl Tag {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

impl Labeled for Tag {
    fn label(&self) -> &str {
        &self.name
    }
    // display_label() uses the default implementation
}

#[derive(Debug)]
pub struct Priority {
    level: u8,
    name: String,
}

impl Priority {
    pub fn new(level: u8, name: &str) -> Self {
        Self {
            level,
            name: name.to_string(),
        }
    }
}

impl Labeled for Priority {
    fn label(&self) -> &str {
        &self.name
    }

    // Override the default to include the level
    fn display_label(&self) -> String {
        format!("[P{}:{}]", self.level, self.name)
    }
}

// ---------------------------------------------------------------------------
// 4. Deriving common traits
// ---------------------------------------------------------------------------

/// A 2D point with derived traits.
///
/// Python equivalent:
/// ```python
/// @dataclass(frozen=True)
/// class Point:
///     x: float
///     y: float
///     # Gets __eq__, __repr__, __hash__ automatically
/// ```
///
/// Rust's #[derive] is more granular — you pick exactly which traits.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn distance_to(&self, other: &Self) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

/// Display is the trait behind `format!("{}", point)`.
/// It's like Python's `__str__`.
impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

// ---------------------------------------------------------------------------
// 5. Operator overloading via traits
// ---------------------------------------------------------------------------

/// Implement Add for Point — like Python's `__add__`.
impl std::ops::Add for Point {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

/// Implement Sub for Point — like Python's `__sub__`.
impl std::ops::Sub for Point {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

// ---------------------------------------------------------------------------
// 6. Dynamic dispatch with trait objects
// ---------------------------------------------------------------------------

/// Render a mixed collection of Summarizable items.
///
/// Python equivalent:
/// ```python
/// def render_feed(items: list[Summarizable]) -> list[str]:
///     return [f"- {item.summary()}" for item in items]
/// ```
///
/// In Rust, mixing different concrete types in one Vec requires dynamic
/// dispatch: `Box<dyn Trait>` or `&dyn Trait`.
pub fn render_feed(items: &[Box<dyn Summarizable>]) -> Vec<String> {
    items
        .iter()
        .map(|item| format!("- {}", item.summary()))
        .collect()
}

// ---------------------------------------------------------------------------
// 7. Multiple trait bounds — the "where" clause
// ---------------------------------------------------------------------------

/// Format a labeled, summarizable item.
///
/// Python equivalent:
/// ```python
/// def card(item):
///     # Assumes item has both .label() and .summary()
///     return f"{item.display_label()} {item.summary()}"
/// ```
///
/// The `where` clause is the same as inline bounds but more readable
/// when you have multiple constraints.
pub fn card<T>(item: &T) -> String
where
    T: Labeled + Summarizable,
{
    format!("{} {}", item.display_label(), item.summary())
}

/// A type that implements both Labeled and Summarizable.
#[derive(Debug, Clone)]
pub struct Article {
    pub section: String,
    pub title: String,
    pub word_count: usize,
}

impl Labeled for Article {
    fn label(&self) -> &str {
        &self.section
    }
}

impl Summarizable for Article {
    fn summary(&self) -> String {
        format!("{} ({} words)", self.title, self.word_count)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Trait basics

    #[test]
    fn summarize_blog_post() {
        let post = BlogPost {
            title: "Ownership in Rust".to_string(),
            author: "Alice".to_string(),
            word_count: 1500,
        };
        assert_eq!(
            post.summary(),
            "\"Ownership in Rust\" by Alice (1500 words)"
        );
    }

    #[test]
    fn summarize_code_snippet() {
        let snippet = CodeSnippet {
            language: "Rust".to_string(),
            lines: 42,
        };
        assert_eq!(snippet.summary(), "Rust snippet (42 lines)");
    }

    // Generic functions with trait bounds

    #[test]
    fn format_summary_works() {
        let post = BlogPost {
            title: "Hello".to_string(),
            author: "Bob".to_string(),
            word_count: 100,
        };
        assert_eq!(format_summary(&post), ">> \"Hello\" by Bob (100 words)");
    }

    #[test]
    fn longest_summary_finds_longest() {
        let snippets = vec![
            CodeSnippet {
                language: "Python".to_string(),
                lines: 10,
            },
            CodeSnippet {
                language: "Rust".to_string(),
                lines: 1000,
            },
        ];
        let longest = longest_summary(&snippets).unwrap();
        assert_eq!(longest.language, "Rust");
    }

    #[test]
    fn longest_summary_empty_returns_none() {
        let empty: Vec<CodeSnippet> = vec![];
        assert!(longest_summary(&empty).is_none());
    }

    // Default methods

    #[test]
    fn tag_uses_default_display_label() {
        let tag = Tag::new("urgent");
        assert_eq!(tag.display_label(), "[urgent]");
    }

    #[test]
    fn priority_overrides_display_label() {
        let p = Priority::new(1, "critical");
        assert_eq!(p.display_label(), "[P1:critical]");
    }

    // Derived traits

    #[test]
    fn point_equality() {
        let a = Point::new(1.0, 2.0);
        let b = Point::new(1.0, 2.0);
        assert_eq!(a, b);
    }

    #[test]
    fn point_debug() {
        let p = Point::new(3.0, 4.0);
        assert_eq!(format!("{:?}", p), "Point { x: 3.0, y: 4.0 }");
    }

    #[test]
    fn point_display() {
        let p = Point::new(3.0, 4.0);
        assert_eq!(format!("{p}"), "(3, 4)");
    }

    #[test]
    fn point_copy() {
        let a = Point::new(1.0, 2.0);
        let b = a; // Copy, not move
        assert_eq!(a, b); // both still valid
    }

    // Operator overloading

    #[test]
    fn point_add() {
        let a = Point::new(1.0, 2.0);
        let b = Point::new(3.0, 4.0);
        assert_eq!(a + b, Point::new(4.0, 6.0));
    }

    #[test]
    fn point_sub() {
        let a = Point::new(5.0, 7.0);
        let b = Point::new(2.0, 3.0);
        assert_eq!(a - b, Point::new(3.0, 4.0));
    }

    #[test]
    fn point_distance() {
        let a = Point::new(0.0, 0.0);
        let b = Point::new(3.0, 4.0);
        assert!((a.distance_to(&b) - 5.0).abs() < f64::EPSILON);
    }

    // Dynamic dispatch

    #[test]
    fn render_mixed_feed() {
        let items: Vec<Box<dyn Summarizable>> = vec![
            Box::new(BlogPost {
                title: "Hello".to_string(),
                author: "Alice".to_string(),
                word_count: 100,
            }),
            Box::new(CodeSnippet {
                language: "Rust".to_string(),
                lines: 50,
            }),
        ];
        let feed = render_feed(&items);
        assert_eq!(feed.len(), 2);
        assert!(feed[0].starts_with("- \"Hello\""));
        assert!(feed[1].starts_with("- Rust snippet"));
    }

    // Multiple trait bounds

    #[test]
    fn article_card() {
        let article = Article {
            section: "Tech".to_string(),
            title: "Why Traits Matter".to_string(),
            word_count: 2000,
        };
        assert_eq!(card(&article), "[Tech] Why Traits Matter (2000 words)");
    }
}
