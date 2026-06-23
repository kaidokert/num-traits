//! Integer parsing from ASCII byte strings, mirroring the still-unstable
//! `from_ascii` / `from_ascii_radix` inherent methods (`int_from_ascii`).
//!
//! Parsing from `&[u8]` instead of `&str` avoids dragging core's UTF-8
//! validation machinery into the binary — valuable on embedded targets —
//! and is exactly what protocol parsers have in hand anyway.
//!
//! The bodies are hand-rolled (std's are unstable, and its `ParseIntError`
//! can't be constructed by other crates), which buys something std doesn't
//! offer yet: the impls are **const-callable on nightly**, so byte-string
//! constants can be parsed at compile time.
//!
//! **CT tier C (CT-hostile)**: data-dependent loop and early-exit parsing,
//! like all string conversion.

use core::fmt;

/// The reason an ASCII parse failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsciiErrorKind {
    /// The input was empty.
    Empty,
    /// A byte was not a valid digit in the requested radix (this includes a
    /// bare or misplaced sign, and a `-` on an unsigned type).
    InvalidDigit,
    /// The value was too large for the type.
    PosOverflow,
    /// The value was too small for the type.
    NegOverflow,
}

/// Error parsing an integer from an ASCII byte string.
///
/// This is this crate's own error type (std's `ParseIntError` is opaque and
/// can't be constructed here), mirroring `IntErrorKind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AsciiParseError {
    /// What went wrong.
    pub kind: AsciiErrorKind,
}

impl fmt::Display for AsciiParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self.kind {
            AsciiErrorKind::Empty => "cannot parse integer from empty byte string",
            AsciiErrorKind::InvalidDigit => "invalid digit found in byte string",
            AsciiErrorKind::PosOverflow => "number too large to fit in target type",
            AsciiErrorKind::NegOverflow => "number too small to fit in target type",
        };
        description.fmt(f)
    }
}

c0nst::c0nst! {
/// Parses an integer from an ASCII byte string, without any UTF-8
/// machinery.
pub c0nst trait FromAscii: Sized {
    /// Parses an integer from base-10 ASCII digits with an optional leading
    /// sign (`+`, and `-` for signed types).
    ///
    /// ```
    /// use const_num_traits::FromAscii;
    ///
    /// assert_eq!(<u32 as FromAscii>::from_ascii(b"1234"), Ok(1234));
    /// assert_eq!(<i32 as FromAscii>::from_ascii(b"-99"), Ok(-99));
    /// assert!(<u8 as FromAscii>::from_ascii(b"300").is_err());
    /// ```
    fn from_ascii(src: &[u8]) -> Result<Self, AsciiParseError>;

    /// Parses an integer from ASCII digits in the given radix, with an
    /// optional leading sign. Digits beyond 9 are `a`-`z` / `A`-`Z`.
    ///
    /// # Panics
    ///
    /// Panics if `radix` is not in the range `2..=36`, matching the inherent
    /// `from_str_radix` methods.
    ///
    /// ```
    /// use const_num_traits::FromAscii;
    ///
    /// assert_eq!(<u32 as FromAscii>::from_ascii_radix(b"DeadBeef", 16), Ok(0xdead_beef));
    /// assert_eq!(<i8 as FromAscii>::from_ascii_radix(b"-80", 16), Ok(i8::MIN));
    /// ```
    fn from_ascii_radix(src: &[u8], radix: u32) -> Result<Self, AsciiParseError>;
}
}

// digit value of an ASCII byte, or u32::MAX as a sentinel for "not a digit"
const fn ascii_digit(b: u8) -> u32 {
    match b {
        b'0'..=b'9' => (b - b'0') as u32,
        b'a'..=b'z' => (b - b'a') as u32 + 10,
        b'A'..=b'Z' => (b - b'A') as u32 + 10,
        _ => u32::MAX,
    }
}

macro_rules! from_ascii_impl {
    // $signed: whether a leading `-` is accepted
    ($signed:literal, $($t:ty)*) => {$(
        c0nst::c0nst! {
        c0nst impl FromAscii for $t {
            #[inline]
            fn from_ascii(src: &[u8]) -> Result<Self, AsciiParseError> {
                <Self as FromAscii>::from_ascii_radix(src, 10)
            }

            fn from_ascii_radix(src: &[u8], radix: u32) -> Result<Self, AsciiParseError> {
                assert!(
                    radix >= 2 && radix <= 36,
                    "from_ascii_radix: radix must lie in the range `[2, 36]`"
                );
                if src.is_empty() {
                    return Err(AsciiParseError { kind: AsciiErrorKind::Empty });
                }
                let (positive, start) = match src[0] {
                    b'+' => (true, 1),
                    b'-' if $signed => (false, 1),
                    _ => (true, 0),
                };
                if src.len() == start {
                    // a bare sign, like std's `from_str_radix("+")`
                    return Err(AsciiParseError { kind: AsciiErrorKind::InvalidDigit });
                }
                let overflow_kind = if positive {
                    AsciiErrorKind::PosOverflow
                } else {
                    AsciiErrorKind::NegOverflow
                };
                let mut result: $t = 0;
                let mut i = start;
                while i < src.len() {
                    let d = ascii_digit(src[i]);
                    if d >= radix {
                        return Err(AsciiParseError { kind: AsciiErrorKind::InvalidDigit });
                    }
                    result = match <$t>::checked_mul(result, radix as $t) {
                        Some(v) => v,
                        None => return Err(AsciiParseError { kind: overflow_kind }),
                    };
                    let step = if positive {
                        <$t>::checked_add(result, d as $t)
                    } else {
                        <$t>::checked_sub(result, d as $t)
                    };
                    result = match step {
                        Some(v) => v,
                        None => return Err(AsciiParseError { kind: overflow_kind }),
                    };
                    i += 1;
                }
                Ok(result)
            }
        }
        }
    )*};
}

from_ascii_impl!(false, usize u8 u16 u32 u64 u128);
from_ascii_impl!(true, isize i8 i16 i32 i64 i128);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        assert_eq!(<u32 as FromAscii>::from_ascii(b"0"), Ok(0));
        assert_eq!(<u32 as FromAscii>::from_ascii(b"+42"), Ok(42));
        assert_eq!(<i32 as FromAscii>::from_ascii(b"-42"), Ok(-42));
        assert_eq!(<u32 as FromAscii>::from_ascii_radix(b"ff", 16), Ok(255));
        assert_eq!(<u32 as FromAscii>::from_ascii_radix(b"FF", 16), Ok(255));
        assert_eq!(<u32 as FromAscii>::from_ascii_radix(b"101", 2), Ok(5));
        assert_eq!(<u32 as FromAscii>::from_ascii_radix(b"z", 36), Ok(35));
        assert_eq!(<i8 as FromAscii>::from_ascii(b"-128"), Ok(i8::MIN));
        assert_eq!(<i8 as FromAscii>::from_ascii(b"127"), Ok(i8::MAX));
    }

    #[test]
    fn errors() {
        use AsciiErrorKind::*;
        let kind = |r: Result<u8, AsciiParseError>| r.unwrap_err().kind;
        assert_eq!(kind(<u8 as FromAscii>::from_ascii(b"")), Empty);
        assert_eq!(kind(<u8 as FromAscii>::from_ascii(b"+")), InvalidDigit);
        assert_eq!(kind(<u8 as FromAscii>::from_ascii(b"-1")), InvalidDigit); // unsigned
        assert_eq!(kind(<u8 as FromAscii>::from_ascii(b"12x")), InvalidDigit);
        assert_eq!(kind(<u8 as FromAscii>::from_ascii(b"1 2")), InvalidDigit);
        assert_eq!(kind(<u8 as FromAscii>::from_ascii(b"256")), PosOverflow);
        assert_eq!(
            kind(<u8 as FromAscii>::from_ascii_radix(b"9", 8)),
            InvalidDigit
        );
        assert_eq!(
            <i8 as FromAscii>::from_ascii(b"-129").unwrap_err().kind,
            NegOverflow
        );
        assert_eq!(
            <i8 as FromAscii>::from_ascii(b"--1").unwrap_err().kind,
            InvalidDigit
        );
    }

    #[test]
    #[should_panic(expected = "radix must lie in the range")]
    fn bad_radix() {
        let _ = <u8 as FromAscii>::from_ascii_radix(b"1", 37);
    }

    #[test]
    fn agrees_with_from_str_radix() {
        // for valid UTF-8 inputs the two parsers must agree on Ok values
        // and on whether an error occurred
        let inputs: &[&str] = &[
            "0", "1", "127", "128", "255", "256", "-1", "-128", "-129", "+5", "ff", "FF", "7f",
            "deadbeef", "", "+", "-", "12_3", "0x10",
        ];
        for radix in [2u32, 8, 10, 16, 36] {
            for s in inputs {
                let ours = <i32 as FromAscii>::from_ascii_radix(s.as_bytes(), radix);
                let std = i32::from_str_radix(s, radix);
                assert_eq!(ours.is_ok(), std.is_ok(), "{s:?} radix {radix}");
                if let (Ok(a), Ok(b)) = (ours, std) {
                    assert_eq!(a, b, "{s:?} radix {radix}");
                }
            }
        }
    }
}
