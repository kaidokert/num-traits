//! Parsing an unsigned integer from a byte slice of *arbitrary* length.
//!
//! [`FromBytes`](super::bytes::FromBytes) mirrors core's `from_le_bytes` — the
//! byte count is a type-level constant (`[u8; N]`) and the read is infallible.
//! This is the variable-length cousin: the input is a `&[u8]` whose length the
//! caller doesn't have to match to the target width, so the parse can fail
//! (input wider than the target), which `FromBytes` has no channel to report.
//! Shorter-than-width input is zero-extended; equal is exact; wider errors.
//!
//! Unsigned-only: zero-extension is unambiguous for magnitudes but has no
//! sign-extension analogue, so signed targets are deliberately excluded.
//!
//! **CT tier A (structure-only)** for a fixed-length input: control flow
//! branches only on `bytes.len()` (emptiness, over-width), never on byte
//! *values*, so a constant-length caller is constant-time.

use core::fmt;

/// The reason a [`FromByteSlice`] parse failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteSliceErrorKind {
    /// The input slice was empty. An empty slice is rejected rather than read
    /// as `0`, so a truncated or missing buffer can't masquerade as a valid
    /// zero value.
    Empty,
    /// The input had more bytes than the target type can hold. The value is
    /// never silently truncated.
    Overflow,
}

/// Error parsing an integer from a byte slice.
///
/// Crate-owned (mirrors the [`AsciiParseError`](super::from_ascii::AsciiParseError)
/// pattern) so it is constructible on stable and needs no std type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByteSliceError {
    /// What went wrong.
    pub kind: ByteSliceErrorKind,
}

impl fmt::Display for ByteSliceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self.kind {
            ByteSliceErrorKind::Empty => "cannot parse integer from empty byte slice",
            ByteSliceErrorKind::Overflow => "byte slice too long for target type",
        };
        description.fmt(f)
    }
}

c0nst::c0nst! {
/// Parses an unsigned integer from a byte slice of arbitrary length.
///
/// Input shorter than the target width is zero-extended (leading zeros for
/// big-endian, trailing for little-endian); input wider than the target is
/// rejected with [`ByteSliceErrorKind::Overflow`] — never truncated. An empty
/// slice is [`ByteSliceErrorKind::Empty`], not `0`.
pub c0nst trait FromByteSlice: Sized {
    /// Parses from a big-endian byte slice.
    ///
    /// ```
    /// use const_num_traits::FromByteSlice;
    ///
    /// assert_eq!(<u32 as FromByteSlice>::from_be_slice(&[0x12, 0x34]), Ok(0x1234));
    /// assert_eq!(<u32 as FromByteSlice>::from_be_slice(&[1, 2, 3, 4]), Ok(0x0102_0304));
    /// assert!(<u16 as FromByteSlice>::from_be_slice(&[1, 2, 3]).is_err()); // too wide
    /// assert!(<u32 as FromByteSlice>::from_be_slice(&[]).is_err());        // empty
    /// ```
    fn from_be_slice(bytes: &[u8]) -> Result<Self, ByteSliceError>;

    /// Parses from a little-endian byte slice.
    ///
    /// ```
    /// use const_num_traits::FromByteSlice;
    ///
    /// assert_eq!(<u32 as FromByteSlice>::from_le_slice(&[0x34, 0x12]), Ok(0x1234));
    /// assert_eq!(<u32 as FromByteSlice>::from_le_slice(&[1, 2, 3, 4]), Ok(0x0403_0201));
    /// ```
    fn from_le_slice(bytes: &[u8]) -> Result<Self, ByteSliceError>;
}
}

macro_rules! from_byte_slice_impl {
    ($($t:ty)*) => {$(
        c0nst::c0nst! {
        c0nst impl FromByteSlice for $t {
            fn from_be_slice(bytes: &[u8]) -> Result<Self, ByteSliceError> {
                if bytes.len() == 0 {
                    return Err(ByteSliceError { kind: ByteSliceErrorKind::Empty });
                }
                let width = core::mem::size_of::<$t>();
                if bytes.len() > width {
                    return Err(ByteSliceError { kind: ByteSliceErrorKind::Overflow });
                }
                // right-align into a zero buffer, then reinterpret — avoids the
                // `<< 8` shift that would overflow a single-byte accumulator.
                let mut buf = [0u8; core::mem::size_of::<$t>()];
                let off = width - bytes.len();
                let mut i = 0;
                while i < bytes.len() {
                    buf[off + i] = bytes[i];
                    i += 1;
                }
                Ok(<$t>::from_be_bytes(buf))
            }

            fn from_le_slice(bytes: &[u8]) -> Result<Self, ByteSliceError> {
                if bytes.len() == 0 {
                    return Err(ByteSliceError { kind: ByteSliceErrorKind::Empty });
                }
                let width = core::mem::size_of::<$t>();
                if bytes.len() > width {
                    return Err(ByteSliceError { kind: ByteSliceErrorKind::Overflow });
                }
                let mut buf = [0u8; core::mem::size_of::<$t>()];
                let mut i = 0;
                while i < bytes.len() {
                    buf[i] = bytes[i];
                    i += 1;
                }
                Ok(<$t>::from_le_bytes(buf))
            }
        }
        }
    )*};
}

from_byte_slice_impl!(usize u8 u16 u32 u64 u128);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_width() {
        assert_eq!(
            <u32 as FromByteSlice>::from_be_slice(&[1, 2, 3, 4]),
            Ok(0x0102_0304)
        );
        assert_eq!(
            <u32 as FromByteSlice>::from_le_slice(&[1, 2, 3, 4]),
            Ok(0x0403_0201)
        );
        assert_eq!(<u8 as FromByteSlice>::from_be_slice(&[0xff]), Ok(0xff));
        assert_eq!(<u8 as FromByteSlice>::from_le_slice(&[0xff]), Ok(0xff));
    }

    #[test]
    fn zero_extends_short_input() {
        // BE: fewer bytes are the least-significant end
        assert_eq!(
            <u32 as FromByteSlice>::from_be_slice(&[0x12, 0x34]),
            Ok(0x0000_1234)
        );
        // LE: fewer bytes are the least-significant end too
        assert_eq!(
            <u32 as FromByteSlice>::from_le_slice(&[0x34, 0x12]),
            Ok(0x0000_1234)
        );
        assert_eq!(<u128 as FromByteSlice>::from_be_slice(&[1]), Ok(1));
    }

    #[test]
    fn round_trips_be_le() {
        let v = 0x0102_0304u32;
        assert_eq!(
            <u32 as FromByteSlice>::from_be_slice(&v.to_be_bytes()),
            Ok(v)
        );
        assert_eq!(
            <u32 as FromByteSlice>::from_le_slice(&v.to_le_bytes()),
            Ok(v)
        );
    }

    #[test]
    fn empty_is_error_not_zero() {
        use ByteSliceErrorKind::*;
        assert_eq!(
            <u32 as FromByteSlice>::from_be_slice(&[]).unwrap_err().kind,
            Empty
        );
        assert_eq!(
            <u32 as FromByteSlice>::from_le_slice(&[]).unwrap_err().kind,
            Empty
        );
    }

    #[test]
    fn too_wide_is_overflow() {
        use ByteSliceErrorKind::*;
        assert_eq!(
            <u16 as FromByteSlice>::from_be_slice(&[1, 2, 3])
                .unwrap_err()
                .kind,
            Overflow
        );
        // over-width even when the surplus high bytes are zero: length-based.
        assert_eq!(
            <u16 as FromByteSlice>::from_be_slice(&[0, 1, 2])
                .unwrap_err()
                .kind,
            Overflow
        );
        assert_eq!(
            <u8 as FromByteSlice>::from_le_slice(&[1, 2])
                .unwrap_err()
                .kind,
            Overflow
        );
    }
}
