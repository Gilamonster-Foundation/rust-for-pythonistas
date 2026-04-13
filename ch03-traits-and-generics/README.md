# Chapter 3: Traits & Generics

## The Big Idea

Python has several ways to define "interfaces": abstract base classes (ABCs),
Protocols (structural typing), and plain duck typing. They all work at
runtime — you find out something doesn't implement the right method when
your program crashes.

Rust has **traits**: named sets of behaviors that types can implement. Like
Python Protocols, they describe *what a type can do*. Unlike Python
Protocols, the compiler checks them *before your code runs*. Combined with
generics, traits let you write code that works with any type meeting certain
requirements — and the compiler generates specialized, zero-cost versions
for each concrete type.

## Python Analogies

### Traits = Protocols (but checked at compile time)

```python
# Python Protocol (PEP 544) — structural typing
from typing import Protocol

class Drawable(Protocol):
    def draw(self) -> str: ...

class Circle:
    def draw(self) -> str:
        return "○"

class Square:
    def draw(self) -> str:
        return "□"

def render(shape: Drawable) -> str:
    return shape.draw()  # type checker warns if .draw() is missing
                         # but the code still RUNS — crash at runtime
```

```rust
// Rust trait — the compiler enforces it
trait Drawable {
    fn draw(&self) -> String;
}

struct Circle;
struct Square;

impl Drawable for Circle {
    fn draw(&self) -> String { "○".to_string() }
}

impl Drawable for Square {
    fn draw(&self) -> String { "□".to_string() }
}

fn render(shape: &dyn Drawable) -> String {
    shape.draw()  // compile error if .draw() is missing — guaranteed safe
}
```

**Key insight:** Python Protocols are advisory — mypy can warn you, but the
code runs regardless. Rust traits are mandatory — if a type claims to
implement a trait, the compiler verifies every required method exists and
has the right signature.

### Generics = Type parameters (but monomorphized)

```python
# Python generic with TypeVar
from typing import TypeVar, List

T = TypeVar('T')

def first(items: List[T]) -> T | None:
    return items[0] if items else None

# At runtime, T is erased — it's just a hint for type checkers
```

```rust
// Rust generic — the compiler generates specialized code for each type
fn first<T>(items: &[T]) -> Option<&T> {
    items.first()
}

// When you call first::<i32>(&numbers), the compiler generates a
// version of `first` specifically for i32. No runtime cost.
// When you call first::<String>(&names), it generates another version.
```

**Key insight:** Python generics are erased at runtime — `List[int]` and
`List[str]` are the same object. Rust generics are *monomorphized* — the
compiler generates specialized machine code for each concrete type. This
means generic Rust code runs as fast as hand-written specialized code.

### Trait bounds = "T must support these operations"

```python
# Python: you just hope T has the right methods
def largest(items: List[T]) -> T:
    return max(items)  # assumes T supports comparison — crashes if not
```

```rust
// Rust: the bound says exactly what T must support
fn largest<T: PartialOrd>(items: &[T]) -> Option<&T> {
    items.iter().reduce(|a, b| if a >= b { a } else { b })
}
// Won't compile if you call it with a type that can't be compared
```

**Key insight:** Python's duck typing means "try it and see." Rust's trait
bounds mean "prove it before we start." The compiler error message tells
you *exactly* which trait is missing.

### Default methods = Mixin behavior

```python
# Python: mixins or default method implementations
class Describable:
    def name(self) -> str:
        raise NotImplementedError

    def describe(self) -> str:
        return f"I am {self.name()}"  # default implementation using name()
```

```rust
// Rust: traits can have default methods
trait Describable {
    fn name(&self) -> &str;  // required — implementors must provide this

    fn describe(&self) -> String {  // default — implementors get this for free
        format!("I am {}", self.name())
    }
}
```

**Key insight:** This is exactly like Python mixins, but with a guarantee:
the required methods are enforced at compile time. No `NotImplementedError`
at runtime.

### `impl Trait` = "something that implements this"

```python
# Python: you write the Protocol in the type hint
def make_shape() -> Drawable:
    return Circle()  # caller knows it's Drawable, not necessarily Circle
```

```rust
// Rust: impl Trait in return position
fn make_shape() -> impl Drawable {
    Circle  // caller knows it's Drawable, compiler knows it's Circle
}
```

### Deriving = Auto-generating trait implementations

```python
# Python: @dataclass auto-generates __eq__, __repr__, __hash__, etc.
from dataclasses import dataclass

@dataclass
class Point:
    x: float
    y: float
# Automatically gets __eq__, __repr__, __init__, etc.
```

```rust
// Rust: #[derive] auto-generates trait implementations
#[derive(Debug, Clone, PartialEq)]
struct Point {
    x: f64,
    y: f64,
}
// Automatically gets Debug (like __repr__), Clone (like copy.deepcopy),
// and PartialEq (like __eq__)
```

**Key insight:** Python's `@dataclass` is a single decorator that generates
multiple methods. Rust's `#[derive]` lets you pick exactly which traits to
auto-implement. You get fine-grained control over what your type can do.

### Operator overloading = `__add__`, `__mul__`, etc.

```python
class Vector:
    def __init__(self, x, y):
        self.x = x
        self.y = y

    def __add__(self, other):
        return Vector(self.x + other.x, self.y + other.y)

    def __repr__(self):
        return f"Vector({self.x}, {self.y})"
```

```rust
use std::ops::Add;

#[derive(Debug, Clone, Copy)]
struct Vector {
    x: f64,
    y: f64,
}

impl Add for Vector {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
```

**Key insight:** Same concept, different mechanism. Python uses magic
methods (`__add__`). Rust uses trait implementations (`impl Add`). The
Rust version is more explicit about what the output type is (it could be
different from the input types).

## Static vs Dynamic Dispatch

One concept that doesn't exist in Python: Rust lets you choose between
*static dispatch* (generics, resolved at compile time) and *dynamic
dispatch* (trait objects, resolved at runtime).

```rust
// Static dispatch — compiler generates specialized code for each type
// Fast (no indirection), but can't mix types in a collection
fn draw_static(shape: &impl Drawable) -> String {
    shape.draw()
}

// Dynamic dispatch — uses a vtable pointer at runtime
// Slight overhead, but can mix different types in a collection
fn draw_dynamic(shape: &dyn Drawable) -> String {
    shape.draw()
}

// Why this matters: you can have a Vec of mixed shapes
let shapes: Vec<Box<dyn Drawable>> = vec![
    Box::new(Circle),
    Box::new(Square),
];
```

Python always does dynamic dispatch (everything goes through `__dict__`
lookup). Rust makes you choose — and the default (generics/static) is
zero-cost.

## Summary

| Python | Rust | What Changes |
|--------|------|-------------|
| ABCs / Protocols | Traits | Compile-time enforcement |
| `TypeVar` / generics | `<T>` generics | Monomorphized — zero runtime cost |
| Duck typing | Trait bounds | Explicit requirements, clear error messages |
| Mixins / default methods | Default trait methods | Same idea, enforced by compiler |
| `@dataclass` | `#[derive(...)]` | Pick exactly which behaviors to generate |
| `__add__`, `__repr__` | `impl Add`, `impl Display` | Operators are traits |
| Always dynamic dispatch | Static or dynamic dispatch | You choose the trade-off |

## Next Steps

Open `src/lib.rs` to see these concepts in working code, then try the
exercises in `exercises/`.
