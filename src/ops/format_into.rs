//! Exact trait mirror of the unstable `format_into` inherent method
//! (`int_format_into`): decimal emission into a [`NumBuffer`] without going
//! through `core::fmt`.
//!
//! **Requires the `nightly-std` feature.** This module names
//! `core::fmt::NumBuffer` / `NumBufferTrait`, which are `#[unstable]` and
//! cannot exist on stable, so it is gated behind `nightly-std` (= `nightly`
//! plus the `int_format_into` lang feature). Keeping it separate from plain
//! `nightly` means const-traits-only builds are not exposed to churn in
//! `int_format_into`. Unlike the rest of the crate — where `nightly` only
//! upgrades *const-ness* and never changes which items exist — this is a
//! deliberately additive item with no stable counterpart: there is no
//! stable signature to diverge from, so the cleanest choice is to mirror
//! std exactly rather than invent a crate-owned parallel buffer API.
//!
//! It is a plain (non-`const`) trait, because the inherent `format_into` is
//! not `const fn` even on nightly. For compile-time decimal emission, use
//! the integer arithmetic traits directly.
//!
//! **CT tier C (CT-hostile)**: digit loops are data-dependent, like all
//! string conversion.

pub use core::fmt::{NumBuffer, NumBufferTrait};

/// Formats an integer as decimal ASCII into a [`NumBuffer`], the generic
/// (trait) form of the inherent `format_into` method.
///
/// The [`NumBufferTrait`] supertrait carries the per-type buffer size as
/// `BUF_SIZE`, mirroring std.
pub trait FormatInto: NumBufferTrait + Sized {
    /// Writes the decimal representation of `self` into `buf` and returns
    /// the `&str` spanning it.
    ///
    /// Mirrors the inherent
    /// [`u32::format_into`](https://doc.rust-lang.org/std/primitive.u32.html#method.format_into)
    /// exactly, including the by-value receiver and the `-` sign for
    /// negative signed values.
    fn format_into(self, buf: &mut NumBuffer<Self>) -> &str;
}

macro_rules! format_into_impl {
    ($($t:ty)*) => {$(
        impl FormatInto for $t {
            #[inline]
            fn format_into(self, buf: &mut NumBuffer<Self>) -> &str {
                // resolves to the inherent method (inherent wins over trait)
                <$t>::format_into(self, buf)
            }
        }
    )*};
}

format_into_impl!(u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats() {
        let mut buf = NumBuffer::new();
        assert_eq!(FormatInto::format_into(0u8, &mut buf), "0");
        assert_eq!(FormatInto::format_into(255u8, &mut buf), "255");

        let mut sbuf = NumBuffer::new();
        assert_eq!(FormatInto::format_into(-1i8, &mut sbuf), "-1");
        assert_eq!(FormatInto::format_into(i8::MIN, &mut sbuf), "-128");

        let mut big = NumBuffer::new();
        assert_eq!(
            FormatInto::format_into(i128::MIN, &mut big),
            "-170141183460469231731687303715884105728"
        );
        let mut ubig = NumBuffer::new();
        assert_eq!(
            FormatInto::format_into(u128::MAX, &mut ubig),
            "340282366920938463463374607431768211455"
        );
    }

    // generic over the trait — the point of having it at all
    fn render<T: FormatInto>(x: T, buf: &mut NumBuffer<T>) -> &str {
        x.format_into(buf)
    }

    #[test]
    fn generic_use() {
        let mut buf = NumBuffer::new();
        assert_eq!(render(42u32, &mut buf), "42");
        let mut buf = NumBuffer::new();
        assert_eq!(render(-7i64, &mut buf), "-7");
    }

    #[test]
    fn roundtrips_with_from_ascii() {
        use crate::ops::from_ascii::FromAscii;
        let mut buf = NumBuffer::new();
        for x in [-128i64, -1, 0, 1, 42, i64::MAX, i64::MIN] {
            let s = FormatInto::format_into(x, &mut buf);
            assert_eq!(<i64 as FromAscii>::from_ascii(s.as_bytes()), Ok(x));
        }
    }
}
