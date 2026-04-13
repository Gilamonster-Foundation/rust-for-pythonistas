//! # Chapter 2: Error Handling
//!
//! This module demonstrates Rust's error handling through examples that
//! map to familiar Python patterns.
//!
//! Run the tests: `cargo test -p ch02-error-handling`

use std::fmt;
use std::num::ParseIntError;

// ---------------------------------------------------------------------------
// 1. Option<T> — Rust's answer to None
// ---------------------------------------------------------------------------

/// Look up a value in a simple in-memory "database."
///
/// Python equivalent:
/// ```python
/// def find_port(service_name):
///     ports = {"http": 80, "https": 443, "ssh": 22}
///     return ports.get(service_name)  # returns None if missing
/// ```
///
/// The difference: Python returns None (any variable can be None).
/// Rust returns Option<u16> — the type *tells* you it might be absent.
pub fn find_port(service: &str) -> Option<u16> {
    match service {
        "http" => Some(80),
        "https" => Some(443),
        "ssh" => Some(22),
        _ => None,
    }
}

/// Chaining Option operations with `map` and `and_then`.
///
/// Python equivalent:
/// ```python
/// def port_as_string(service):
///     port = find_port(service)
///     if port is not None:
///         return f":{port}"
///     return None
/// ```
///
/// Rust's combinators let you avoid nested if-let / match blocks.
pub fn port_as_string(service: &str) -> Option<String> {
    find_port(service).map(|p| format!(":{p}"))
}

/// Using `unwrap_or` — like Python's `value if value is not None else default`.
///
/// Python equivalent:
/// ```python
/// def port_or_default(service):
///     return find_port(service) or 8080
/// ```
pub fn port_or_default(service: &str) -> u16 {
    find_port(service).unwrap_or(8080)
}

// ---------------------------------------------------------------------------
// 2. Result<T, E> — Errors as values, not exceptions
// ---------------------------------------------------------------------------

/// A custom error type — like a Python exception class, but an enum.
///
/// Python equivalent:
/// ```python
/// class ParseError(Exception): pass
/// class OutOfRange(Exception):
///     def __init__(self, value, min_val, max_val): ...
/// ```
#[derive(Debug, PartialEq)]
pub enum PortError {
    /// The input string couldn't be parsed as a number.
    NotANumber(String),
    /// The number is outside the valid port range.
    OutOfRange { value: i64, min: u16, max: u16 },
}

impl fmt::Display for PortError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotANumber(s) => write!(f, "not a valid number: {s}"),
            Self::OutOfRange { value, min, max } => {
                write!(f, "port {value} out of range ({min}..{max})")
            }
        }
    }
}

/// Convert a ParseIntError into our custom PortError.
///
/// This is like Python's `raise PortError(...) from original_exception`.
/// The `From` trait lets the `?` operator do this conversion automatically.
impl From<ParseIntError> for PortError {
    fn from(e: ParseIntError) -> Self {
        Self::NotANumber(e.to_string())
    }
}

/// Parse a string as a valid port number (1-65535).
///
/// Python equivalent:
/// ```python
/// def parse_port(s):
///     try:
///         value = int(s)
///     except ValueError:
///         raise ParseError(f"not a valid number: {s}")
///     if not (1 <= value <= 65535):
///         raise OutOfRange(value, 1, 65535)
///     return value
/// ```
///
/// The Rust version returns Result instead of raising — the caller can see
/// from the type signature that this function can fail.
pub fn parse_port(s: &str) -> Result<u16, PortError> {
    let value: i64 = s
        .parse()
        .map_err(|_| PortError::NotANumber(s.to_string()))?;

    if !(1..=65535).contains(&value) {
        return Err(PortError::OutOfRange {
            value,
            min: 1,
            max: 65535,
        });
    }

    Ok(value as u16)
}

// ---------------------------------------------------------------------------
// 3. The ? operator — explicit propagation
// ---------------------------------------------------------------------------

/// A network address: host + port.
#[derive(Debug, PartialEq)]
pub struct Address {
    pub host: String,
    pub port: u16,
}

/// Parse "host:port" into an Address.
///
/// Python equivalent:
/// ```python
/// def parse_address(s):
///     if ':' not in s:
///         raise ParseError("missing ':'")
///     host, port_str = s.rsplit(':', 1)
///     port = parse_port(port_str)  # raises on bad port — propagates!
///     return Address(host=host, port=port)
/// ```
///
/// Notice each `?` in the Rust version. They mark *exactly* where the
/// function might return early with an error. No hidden control flow.
pub fn parse_address(s: &str) -> Result<Address, PortError> {
    let (host, port_str) = s
        .rsplit_once(':')
        .ok_or_else(|| PortError::NotANumber("missing ':' separator".to_string()))?;

    let port = parse_port(port_str)?; // ? propagates PortError

    Ok(Address {
        host: host.to_string(),
        port,
    })
}

// ---------------------------------------------------------------------------
// 4. Combining Option and Result
// ---------------------------------------------------------------------------

/// Look up a service port, falling back to parsing a custom port string.
///
/// Python equivalent:
/// ```python
/// def resolve_port(service_or_number):
///     port = find_port(service_or_number)
///     if port is not None:
///         return port
///     return parse_port(service_or_number)  # might raise
/// ```
///
/// This shows how Option and Result interact: Option for "might not exist"
/// and Result for "might fail with a specific error."
pub fn resolve_port(service_or_number: &str) -> Result<u16, PortError> {
    // If it's a known service, use that
    if let Some(port) = find_port(service_or_number) {
        return Ok(port);
    }
    // Otherwise try to parse as a number
    parse_port(service_or_number)
}

// ---------------------------------------------------------------------------
// 5. Iterating with Results — collect into Result
// ---------------------------------------------------------------------------

/// Parse multiple port strings, failing on the first bad one.
///
/// Python equivalent:
/// ```python
/// def parse_all_ports(strings):
///     return [parse_port(s) for s in strings]  # raises on first bad one
/// ```
///
/// Rust's iterator + collect can gather Results into a single Result<Vec>.
/// This is one of those "Rust lets you express something cleanly that
/// Python can't" moments.
pub fn parse_all_ports(strings: &[&str]) -> Result<Vec<u16>, PortError> {
    strings.iter().map(|s| parse_port(s)).collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Option tests

    #[test]
    fn option_some() {
        assert_eq!(find_port("http"), Some(80));
    }

    #[test]
    fn option_none() {
        assert_eq!(find_port("gopher"), None);
    }

    #[test]
    fn option_map() {
        assert_eq!(port_as_string("https"), Some(":443".to_string()));
        assert_eq!(port_as_string("gopher"), None);
    }

    #[test]
    fn option_unwrap_or() {
        assert_eq!(port_or_default("http"), 80);
        assert_eq!(port_or_default("gopher"), 8080);
    }

    // Result tests

    #[test]
    fn result_ok() {
        assert_eq!(parse_port("443"), Ok(443));
    }

    #[test]
    fn result_not_a_number() {
        assert_eq!(
            parse_port("abc"),
            Err(PortError::NotANumber("abc".to_string()))
        );
    }

    #[test]
    fn result_out_of_range() {
        assert_eq!(
            parse_port("99999"),
            Err(PortError::OutOfRange {
                value: 99999,
                min: 1,
                max: 65535,
            })
        );
    }

    #[test]
    fn result_zero_is_invalid() {
        assert_eq!(
            parse_port("0"),
            Err(PortError::OutOfRange {
                value: 0,
                min: 1,
                max: 65535,
            })
        );
    }

    // ? operator tests

    #[test]
    fn address_parse_ok() {
        assert_eq!(
            parse_address("localhost:8080"),
            Ok(Address {
                host: "localhost".to_string(),
                port: 8080,
            })
        );
    }

    #[test]
    fn address_bad_port_propagates() {
        assert!(parse_address("localhost:abc").is_err());
    }

    #[test]
    fn address_missing_colon() {
        assert!(parse_address("localhost").is_err());
    }

    // Option + Result interop

    #[test]
    fn resolve_known_service() {
        assert_eq!(resolve_port("ssh"), Ok(22));
    }

    #[test]
    fn resolve_numeric_port() {
        assert_eq!(resolve_port("3000"), Ok(3000));
    }

    #[test]
    fn resolve_bad_string() {
        assert!(resolve_port("not_a_service").is_err());
    }

    // Collecting Results

    #[test]
    fn collect_all_ok() {
        assert_eq!(parse_all_ports(&["80", "443", "22"]), Ok(vec![80, 443, 22]));
    }

    #[test]
    fn collect_fails_on_first_bad() {
        let result = parse_all_ports(&["80", "bad", "443"]);
        assert!(result.is_err());
    }
}
