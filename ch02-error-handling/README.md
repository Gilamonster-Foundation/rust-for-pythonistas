# Chapter 2: Error Handling

## The Big Idea

Python uses exceptions for errors and `None` for missing values. Both are
invisible in function signatures — you only discover them by reading docs
(or hitting them at runtime). Rust makes errors and absence *part of the
type system*. A function that can fail returns `Result<T, E>`. A function
that might have nothing returns `Option<T>`. The compiler won't let you
ignore either one.

This isn't just syntax sugar — it changes how you think about error paths.
In Python, error handling is something you bolt on after the fact. In Rust,
it's part of the design from the start.

## Python Analogies

### `Option<T>` = The problem `None` was trying to solve

```python
# Python: None is a valid value for any variable
def find_user(user_id):
    if user_id in database:
        return database[user_id]
    return None

user = find_user(42)
print(user.name)  # AttributeError if user is None — runtime crash!
```

```rust
// Rust: Option<T> forces you to handle the None case
fn find_user(user_id: u64) -> Option<User> {
    database.get(&user_id).cloned()
}

let user = find_user(42);
// user.name  // compile error! user is Option<User>, not User

// You must unwrap it explicitly:
match user {
    Some(u) => println!("{}", u.name),
    None => println!("User not found"),
}
```

**Key insight:** Python's `None` is a billion-dollar mistake (Tony Hoare's
words). Any variable can be `None`, and nothing forces you to check. Rust's
`Option<T>` is a type — if a function returns `Option<User>`, you *must*
handle the `None` case before you can use the `User`. The compiler enforces
what Python hopes you'll remember.

### `Result<T, E>` = `try/except` but visible in the signature

```python
# Python: you can't tell from the signature that this function raises
def parse_config(path):
    with open(path) as f:          # might raise FileNotFoundError
        data = json.load(f)        # might raise JSONDecodeError
    return Config(**data)           # might raise TypeError
# Caller has to guess what to catch (or read the source)
```

```rust
// Rust: the signature tells you this function can fail, and how
fn parse_config(path: &str) -> Result<Config, ConfigError> {
    let content = std::fs::read_to_string(path)?;  // propagates io::Error
    let data: Value = serde_json::from_str(&content)?;  // propagates json Error
    Config::from_value(data)  // returns Result<Config, ConfigError>
}
// Caller knows exactly what can go wrong — it's in the type
```

**Key insight:** Python's exception system is powerful but invisible. Any
function can raise anything. Rust's `Result<T, E>` makes failure a
first-class part of the return type. You can't accidentally ignore an error
because the compiler won't let you use the success value without handling
the error case first.

### The `?` operator = Python's implicit exception propagation, but explicit

```python
# Python: exceptions propagate automatically up the call stack
def load_settings():
    config = parse_config("settings.json")  # if this raises, it bubbles up
    return config.settings                   # caller never sees this line
```

```rust
// Rust: the ? operator propagates errors explicitly
fn load_settings() -> Result<Settings, ConfigError> {
    let config = parse_config("settings.json")?;  // ? = "if Err, return it"
    Ok(config.settings)
}
```

**Key insight:** In Python, every function call is an implicit `?` — errors
always propagate unless you catch them. In Rust, propagation is opt-in with
`?`. This means you can see *exactly* which calls in a function might cause
it to return early. No hidden control flow.

### LBYL vs EAFP — Rust chooses neither (it chooses types)

Python has two schools of error handling:

```python
# LBYL: Look Before You Leap
if os.path.exists(path):
    with open(path) as f:
        data = f.read()
# Problem: file could be deleted between the check and the open (TOCTOU race)

# EAFP: Easier to Ask Forgiveness than Permission
try:
    with open(path) as f:
        data = f.read()
except FileNotFoundError:
    data = default_data
# Better, but you have to know which exception to catch
```

```rust
// Rust: the type system handles it — no LBYL/EAFP debate needed
match std::fs::read_to_string(path) {
    Ok(data) => process(data),
    Err(e) if e.kind() == ErrorKind::NotFound => use_default(),
    Err(e) => return Err(e.into()),  // propagate unexpected errors
}
```

**Key insight:** LBYL has race conditions. EAFP has invisible error types.
Rust's `match` on `Result` gives you exhaustive handling without either
problem — and the compiler tells you if you missed a case.

### Custom error types = Custom exception classes

```python
# Python custom exceptions
class ValidationError(Exception):
    def __init__(self, field, message):
        self.field = field
        self.message = message
        super().__init__(f"{field}: {message}")
```

```rust
// Rust custom errors — they're just enums
#[derive(Debug)]
enum ValidationError {
    MissingField(String),
    InvalidValue { field: String, message: String },
    TooLong { field: String, max: usize, actual: usize },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingField(name) => write!(f, "missing field: {name}"),
            Self::InvalidValue { field, message } => write!(f, "{field}: {message}"),
            Self::TooLong { field, max, actual } =>
                write!(f, "{field}: too long ({actual} > {max})"),
        }
    }
}
```

**Key insight:** Python exceptions are classes in a hierarchy. Rust errors
are enums with variants. The `match` statement on a Rust error enum is
exhaustive — the compiler ensures you handle every variant. Python's
`except` blocks are best-effort — you can always miss one.

## Summary

| Python | Rust | What Changes |
|--------|------|-------------|
| `None` (any variable) | `Option<T>` | Absence is a type, not a surprise |
| `try/except` | `Result<T, E>` | Errors visible in function signatures |
| Implicit propagation | `?` operator | You see where errors can escape |
| LBYL / EAFP debate | `match` on Result | Exhaustive handling, no race conditions |
| Exception class hierarchy | Error enums | Compiler checks exhaustiveness |
| `assert` / `raise` for bugs | `panic!` / `unreachable!` | Unrecoverable = crash, recoverable = Result |

## The Panic Distinction

One more thing Python developers need to know: Rust separates
*recoverable* errors from *unrecoverable* ones.

- **Recoverable**: file not found, invalid input, network timeout → `Result<T, E>`
- **Unrecoverable**: index out of bounds, violated invariant → `panic!`

In Python, both are exceptions. In Rust, a `panic!` is a program bug — it
means something happened that *should never happen*. A `Result::Err` is an
expected failure — the system is working correctly by reporting it.

Don't use `panic!` for things users might do wrong. Don't use `Result` for
things that indicate bugs. This distinction makes Rust programs much more
predictable than Python programs, where `KeyError` might mean "bad user
input" or "bug in your code" depending on context.

## Next Steps

Open `src/lib.rs` to see these concepts in working code, then try the
exercises in `exercises/`.
