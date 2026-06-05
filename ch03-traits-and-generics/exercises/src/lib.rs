//! # Chapter 3 Exercises: Traits & Generics
//!
//! Each exercise shows a Python snippet and asks you to write the Rust
//! equivalent. Replace the `todo!()` markers with working code.
//!
//! Run tests: `cargo test -p ch03-exercises`

// These allows are intentional: exercise stubs have unused parameters
// and fields until the student fills in the todo!() markers.
#![allow(unused_variables, dead_code)]

use std::fmt;

// ============================================================
// Exercise 1: Define and Implement a Trait
// ============================================================
//
// Python version:
// ```python
// class HasArea(Protocol):
//     def area(self) -> float: ...
//
// class Rectangle:
//     def __init__(self, width, height):
//         self.width = width
//         self.height = height
//     def area(self):
//         return self.width * self.height
//
// class Circle:
//     def __init__(self, radius):
//         self.radius = radius
//     def area(self):
//         return 3.14159265 * self.radius ** 2
// ```
//
// 1. Define a trait `HasArea` with a method `fn area(&self) -> f64`
// 2. Implement it for Rectangle and Circle (structs provided below)

pub trait HasArea {
    fn area(&self) -> f64;
}

pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}

pub struct Circle {
    pub radius: f64,
}

impl HasArea for Rectangle {
    fn area(&self) -> f64 {
        todo!("width * height")
    }
}

impl HasArea for Circle {
    fn area(&self) -> f64 {
        todo!("PI * radius^2 — use std::f64::consts::PI")
    }
}

// ============================================================
// Exercise 2: Generic Function with Trait Bound
// ============================================================
//
// Python version:
// ```python
// def largest_area(shapes: list[HasArea]) -> float:
//     return max(shape.area() for shape in shapes)
//
// assert largest_area([Rectangle(3, 4), Circle(1)]) == 12.0
// ```
//
// Write a generic function that finds the largest area in a slice.
// The bound: T must implement HasArea.

pub fn largest_area<T: HasArea>(shapes: &[T]) -> Option<f64> {
    todo!("Return the largest area, or None if the slice is empty")
}

// ============================================================
// Exercise 3: Display Trait (Python's __str__)
// ============================================================
//
// Python version:
// ```python
// class Temperature:
//     def __init__(self, celsius):
//         self.celsius = celsius
//     def __str__(self):
//         return f"{self.celsius}°C"
//     def __repr__(self):
//         return f"Temperature(celsius={self.celsius})"
// ```
//
// Implement Display for Temperature so that:
//   format!("{}", temp) returns "23.5°C"
//
// Debug is already derived for you (__repr__ equivalent).

#[derive(Debug, Clone, Copy)]
pub struct Temperature {
    pub celsius: f64,
}

impl Temperature {
    pub fn new(celsius: f64) -> Self {
        Self { celsius }
    }

    pub fn to_fahrenheit(&self) -> f64 {
        self.celsius * 9.0 / 5.0 + 32.0
    }
}

impl fmt::Display for Temperature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!("Write celsius followed by °C")
    }
}

// ============================================================
// Exercise 4: Operator Overloading
// ============================================================
//
// Python version:
// ```python
// class Money:
//     def __init__(self, cents):
//         self.cents = cents
//     def __add__(self, other):
//         return Money(self.cents + other.cents)
//     def __eq__(self, other):
//         return self.cents == other.cents
//     def __str__(self):
//         return f"${self.cents / 100:.2f}"
// ```
//
// Implement Add and Display for Money. PartialEq is already derived.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Money {
    pub cents: i64,
}

impl Money {
    pub fn new(cents: i64) -> Self {
        Self { cents }
    }

    pub fn from_dollars(dollars: f64) -> Self {
        Self {
            cents: (dollars * 100.0).round() as i64,
        }
    }
}

impl std::ops::Add for Money {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        todo!("Add the cents together")
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!("Format as $X.XX — dollars and cents with 2 decimal places")
    }
}

// ============================================================
// Exercise 5: Trait with Default Method + Dynamic Dispatch
// ============================================================
//
// Python version:
// ```python
// class Renderable:
//     def render(self) -> str:
//         raise NotImplementedError
//     def render_with_border(self) -> str:
//         content = self.render()
//         width = max(len(line) for line in content.split('\n'))
//         border = '+' + '-' * (width + 2) + '+'
//         lines = [f"| {line:<{width}} |" for line in content.split('\n')]
//         return '\n'.join([border] + lines + [border])
//
// class TextBlock(Renderable):
//     def __init__(self, text): self.text = text
//     def render(self): return self.text
//
// class NumberBlock(Renderable):
//     def __init__(self, value): self.value = value
//     def render(self): return str(self.value)
// ```
//
// 1. Implement `render()` for TextBlock and NumberBlock
// 2. The `render_with_border()` default method is provided
// 3. Implement `render_all` to work with mixed types (dynamic dispatch)

pub trait Renderable {
    /// Required: return the content to render.
    fn render(&self) -> String;

    /// Default: wrap render() output in a border.
    fn render_with_border(&self) -> String {
        let content = self.render();
        let width = content.lines().map(|l| l.len()).max().unwrap_or(0);
        let border = format!("+{}+", "-".repeat(width + 2));
        let body: Vec<String> = content
            .lines()
            .map(|line| format!("| {:<width$} |", line))
            .collect();
        format!("{}\n{}\n{}", border, body.join("\n"), border)
    }
}

pub struct TextBlock {
    pub text: String,
}

pub struct NumberBlock {
    pub value: f64,
}

impl Renderable for TextBlock {
    fn render(&self) -> String {
        todo!("Return self.text")
    }
}

impl Renderable for NumberBlock {
    fn render(&self) -> String {
        todo!("Return self.value as a string")
    }
}

/// Render all items in a mixed collection, one per line.
///
/// Python equivalent:
/// ```python
/// def render_all(items: list[Renderable]) -> str:
///     return '\n'.join(item.render() for item in items)
/// ```
///
/// Hint: use `&[Box<dyn Renderable>]` for the parameter type.
pub fn render_all(items: &[Box<dyn Renderable>]) -> String {
    todo!("Join each item's render() output with newlines")
}

// ============================================================
// Tests — do not modify below this line
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Exercise 1
    #[test]
    fn ex1_rectangle_area() {
        let r = Rectangle {
            width: 3.0,
            height: 4.0,
        };
        assert!((r.area() - 12.0).abs() < f64::EPSILON);
    }

    #[test]
    fn ex1_circle_area() {
        let c = Circle { radius: 1.0 };
        assert!((c.area() - std::f64::consts::PI).abs() < 1e-10);
    }

    // Exercise 2
    #[test]
    fn ex2_largest_area() {
        let rects = vec![
            Rectangle {
                width: 2.0,
                height: 3.0,
            },
            Rectangle {
                width: 10.0,
                height: 1.0,
            },
            Rectangle {
                width: 4.0,
                height: 4.0,
            },
        ];
        let largest = largest_area(&rects).unwrap();
        assert!((largest - 16.0).abs() < f64::EPSILON);
    }

    #[test]
    fn ex2_empty_returns_none() {
        let empty: Vec<Rectangle> = vec![];
        assert!(largest_area(&empty).is_none());
    }

    // Exercise 3
    #[test]
    fn ex3_temperature_display() {
        let t = Temperature::new(23.5);
        assert_eq!(format!("{t}"), "23.5\u{00B0}C");
    }

    #[test]
    fn ex3_temperature_debug() {
        let t = Temperature::new(100.0);
        assert_eq!(format!("{t:?}"), "Temperature { celsius: 100.0 }");
    }

    // Exercise 4
    #[test]
    fn ex4_money_add() {
        let a = Money::new(150);
        let b = Money::new(250);
        assert_eq!(a + b, Money::new(400));
    }

    #[test]
    fn ex4_money_display() {
        assert_eq!(format!("{}", Money::new(150)), "$1.50");
        assert_eq!(format!("{}", Money::new(7)), "$0.07");
        assert_eq!(format!("{}", Money::new(1000)), "$10.00");
    }

    #[test]
    fn ex4_money_from_dollars() {
        assert_eq!(Money::from_dollars(9.99), Money::new(999));
    }

    // Exercise 5
    #[test]
    fn ex5_text_render() {
        let t = TextBlock {
            text: "hello".to_string(),
        };
        assert_eq!(t.render(), "hello");
    }

    #[test]
    fn ex5_number_render() {
        let n = NumberBlock { value: 42.0 };
        assert_eq!(n.render(), "42");
    }

    #[test]
    fn ex5_border() {
        let t = TextBlock {
            text: "hi".to_string(),
        };
        let bordered = t.render_with_border();
        assert!(bordered.contains("+----+"));
        assert!(bordered.contains("| hi |"));
    }

    #[test]
    fn ex5_render_all() {
        let items: Vec<Box<dyn Renderable>> = vec![
            Box::new(TextBlock {
                text: "hello".to_string(),
            }),
            Box::new(NumberBlock { value: 42.0 }),
        ];
        assert_eq!(render_all(&items), "hello\n42");
    }
}
