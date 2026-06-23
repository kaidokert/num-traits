//! Integer-to-integer conversions: sign reinterpretation, absolute
//! difference, value-preserving widening, lossy truncation, and the generic
//! cast family, mirroring `cast_signed` / `cast_unsigned` / `unsigned_abs` /
//! `abs_diff` / `widen` / `truncate` /
//! `{checked,wrapping,saturating,strict}_cast` / `clamp_magnitude` on the
//! primitive integer types.
//!
//! The checked/saturating/strict *sign-cast* and *truncate* variants found
//! in std are NOT mirrored as methods here: they are exactly
//! the generic cast traits ([`CheckedCast`], [`SaturatingCast`],
//! [`StrictCast`]) applied to the counterpart type, so the sign-cast and
//! truncate traits stay one-method (the Tier-A bit-pattern operations) and
//! the generic casts are the single home for value-checked conversion.
//!
//! Stability in std (as of nightly 2026): `unsigned_abs` (1.51) and
//! `abs_diff` (1.60) are delegated; `cast_signed`/`cast_unsigned` (1.87),
//! `widen`/`truncate` (`integer_widen_truncate`), the generic `*_cast`
//! family (`integer_casts`) and `clamp_magnitude` are newer than the
//! crate's MSRV or still nightly-only, and are hand-rolled with the same
//! semantics.
//!
//! **CT tiers**: [`CastSigned`]/[`CastUnsigned`], [`Truncate`], [`Widen`],
//! [`WrappingCast`], [`AbsDiff`] and [`UnsignedAbs`] are Tier A
//! (bit-pattern / borrow-free operations); [`CheckedCast`],
//! [`SaturatingCast`] and [`ClampMagnitude`] are Tier B; [`StrictCast`] is
//! Tier C (data-dependent panic).

c0nst::c0nst! {
/// Computes the absolute difference of two values.
pub c0nst trait AbsDiff: Sized {
    /// The result type: `Self` for unsigned types, the unsigned counterpart
    /// for signed types (so the difference always fits).
    type Output;

    /// Computes the absolute difference between `self` and `other` without
    /// the possibility of overflow.
    ///
    /// ```
    /// use const_num_traits::AbsDiff;
    ///
    /// assert_eq!(AbsDiff::abs_diff(100u8, 120), 20);
    /// assert_eq!(AbsDiff::abs_diff(i8::MIN, i8::MAX), 255u8);
    /// ```
    fn abs_diff(self, other: Self) -> Self::Output;
}
}

macro_rules! abs_diff_impl {
    ($($t:ty => $out:ty;)*) => {$(
        c0nst::c0nst! {
        c0nst impl AbsDiff for $t {
            type Output = $out;

            #[inline]
            fn abs_diff(self, other: Self) -> $out {
                <$t>::abs_diff(self, other)
            }
        }
        }
    )*};
}

abs_diff_impl! {
    u8 => u8; u16 => u16; u32 => u32; u64 => u64; usize => usize; u128 => u128;
    i8 => u8; i16 => u16; i32 => u32; i64 => u64; isize => usize; i128 => u128;
}

c0nst::c0nst! {
/// Computes the absolute value as the unsigned counterpart type, which
/// cannot overflow (unlike `abs`, which overflows on `MIN`).
pub c0nst trait UnsignedAbs: Sized {
    /// The unsigned counterpart type (e.g. `u32` for `i32`).
    type Unsigned;

    /// Computes the absolute value of `self` without any wrapping or
    /// panicking.
    ///
    /// ```
    /// use const_num_traits::UnsignedAbs;
    ///
    /// assert_eq!(UnsignedAbs::unsigned_abs(-100i8), 100u8);
    /// assert_eq!(UnsignedAbs::unsigned_abs(i8::MIN), 128u8);
    /// ```
    fn unsigned_abs(self) -> Self::Unsigned;
}
}

c0nst::c0nst! {
/// Clamps the magnitude (absolute value) of a signed integer.
pub c0nst trait ClampMagnitude: Sized {
    /// The unsigned counterpart type used for the limit.
    type Unsigned;
    /// The (owned) result type.
    type Output;

    /// Clamps `self` to the symmetric range `[-limit, limit]`. Limits that
    /// don't fit in `Self` leave the value unchanged.
    ///
    /// ```
    /// use const_num_traits::ClampMagnitude;
    ///
    /// assert_eq!(ClampMagnitude::clamp_magnitude(120i8, 100), 100);
    /// assert_eq!(ClampMagnitude::clamp_magnitude(-120i8, 100), -100);
    /// assert_eq!(ClampMagnitude::clamp_magnitude(-120i8, 200), -120);
    /// ```
    fn clamp_magnitude(self, limit: Self::Unsigned) -> Self::Output;
}
}

macro_rules! unsigned_abs_impl {
    ($($t:ty => $u:ty;)*) => {$(
        c0nst::c0nst! {
        c0nst impl UnsignedAbs for $t {
            type Unsigned = $u;

            #[inline]
            fn unsigned_abs(self) -> $u {
                <$t>::unsigned_abs(self)
            }
        }
        }

        c0nst::c0nst! {
        c0nst impl ClampMagnitude for $t {
            type Unsigned = $u;
            type Output = $t;

            // still unstable in std (and not yet const there)
            #[inline]
            fn clamp_magnitude(self, limit: $u) -> $t {
                if limit <= <$t>::MAX as $u {
                    let lim = limit as $t;
                    if self < -lim {
                        -lim
                    } else if self > lim {
                        lim
                    } else {
                        self
                    }
                } else {
                    self
                }
            }
        }
        }
    )*};
}

unsigned_abs_impl! {
    i8 => u8; i16 => u16; i32 => u32; i64 => u64; isize => usize; i128 => u128;
}

c0nst::c0nst! {
/// Reinterprets an unsigned integer's bits as its signed counterpart.
///
/// For value-checked conversion to the signed counterpart, use
/// [`CheckedCast`] / [`SaturatingCast`] / [`StrictCast`] with the signed
/// target type.
pub c0nst trait CastSigned: Sized {
    /// The signed counterpart type (e.g. `i32` for `u32`).
    type Signed;

    /// Returns the bit pattern of `self` reinterpreted as its signed
    /// counterpart (two's complement: values above `Signed::MAX` become
    /// negative).
    ///
    /// ```
    /// use const_num_traits::CastSigned;
    ///
    /// assert_eq!(CastSigned::cast_signed(255u8), -1i8);
    /// assert_eq!(CastSigned::cast_signed(127u8), 127i8);
    /// ```
    fn cast_signed(self) -> Self::Signed;
}
}

macro_rules! cast_signed_impl {
    ($($t:ty => $s:ty;)*) => {$(
        c0nst::c0nst! {
        c0nst impl CastSigned for $t {
            type Signed = $s;

            #[inline]
            fn cast_signed(self) -> $s {
                self as $s
            }
        }
        }
    )*};
}

cast_signed_impl! {
    u8 => i8; u16 => i16; u32 => i32; u64 => i64; usize => isize; u128 => i128;
}

c0nst::c0nst! {
/// Reinterprets a signed integer's bits as its unsigned counterpart.
///
/// For value-checked conversion to the unsigned counterpart, use
/// [`CheckedCast`] / [`SaturatingCast`] / [`StrictCast`] with the unsigned
/// target type.
pub c0nst trait CastUnsigned: Sized {
    /// The unsigned counterpart type (e.g. `u32` for `i32`).
    type Unsigned;

    /// Returns the bit pattern of `self` reinterpreted as its unsigned
    /// counterpart (two's complement: negative values become large).
    ///
    /// ```
    /// use const_num_traits::CastUnsigned;
    ///
    /// assert_eq!(CastUnsigned::cast_unsigned(-1i8), 255u8);
    /// assert_eq!(CastUnsigned::cast_unsigned(127i8), 127u8);
    /// ```
    fn cast_unsigned(self) -> Self::Unsigned;
}
}

macro_rules! cast_unsigned_impl {
    ($($t:ty => $u:ty;)*) => {$(
        c0nst::c0nst! {
        c0nst impl CastUnsigned for $t {
            type Unsigned = $u;

            #[inline]
            fn cast_unsigned(self) -> $u {
                self as $u
            }
        }
        }
    )*};
}

cast_unsigned_impl! {
    i8 => u8; i16 => u16; i32 => u32; i64 => u64; isize => usize; i128 => u128;
}

c0nst::c0nst! {
/// Widens to an integer of the same signedness and the same size or larger,
/// always preserving the value.
pub c0nst trait Widen<T>: Sized {
    /// Widens `self` to the target type, preserving its value. Implemented
    /// for the same pairs as std's `widen` (including the identity
    /// conversion; `usize`/`isize` only participate where the conversion is
    /// valid on every platform).
    ///
    /// ```
    /// use const_num_traits::Widen;
    ///
    /// let x: u32 = Widen::widen(200u8);
    /// assert_eq!(x, 200);
    /// let y: i128 = Widen::widen(-5i64);
    /// assert_eq!(y, -5);
    /// ```
    fn widen(self) -> T;
}
}

macro_rules! widen_impl {
    ($($from:ty => $($to:ty),+;)*) => {$($(
        c0nst::c0nst! {
        c0nst impl Widen<$to> for $from {
            #[inline]
            fn widen(self) -> $to {
                self as $to
            }
        }
        }
    )+)*};
}

// same pairs as core's `impl_widen!`
widen_impl! {
    u8 => u8, u16, u32, u64, u128, usize;
    u16 => u16, u32, u64, u128, usize;
    u32 => u32, u64, u128;
    u64 => u64, u128;
    u128 => u128;
    usize => usize;

    i8 => i8, i16, i32, i64, i128, isize;
    i16 => i16, i32, i64, i128, isize;
    i32 => i32, i64, i128;
    i64 => i64, i128;
    i128 => i128;
    isize => isize;
}

c0nst::c0nst! {
/// Truncates to an integer of the same signedness and the same size or
/// smaller, discarding high-order bits.
///
/// For bounds-checked or saturating narrowing, use [`CheckedCast`] /
/// [`SaturatingCast`] / [`StrictCast`] — on same-signedness pairs they have
/// exactly the semantics of std's `checked_truncate` /
/// `saturating_truncate`.
pub c0nst trait Truncate<T>: Sized {
    /// Truncates `self` to the target type, discarding high-order bits.
    /// Implemented for the same pairs as std's `truncate` (including the
    /// identity conversion).
    ///
    /// ```
    /// use const_num_traits::Truncate;
    ///
    /// let x: u8 = Truncate::truncate(0x1234u16);
    /// assert_eq!(x, 0x34);
    /// ```
    fn truncate(self) -> T;
}
}

macro_rules! truncate_impl {
    ($($from:ty => $($to:ty),+;)*) => {$($(
        c0nst::c0nst! {
        c0nst impl Truncate<$to> for $from {
            #[inline]
            fn truncate(self) -> $to {
                self as $to
            }
        }
        }
    )+)*};
}

// same pairs as core's `impl_truncate!`
truncate_impl! {
    u8 => u8;
    u16 => u16, u8;
    u32 => u32, u16, u8;
    u64 => u64, u32, u16, u8;
    u128 => u128, u64, u32, u16, u8;
    usize => usize, u16, u8;

    i8 => i8;
    i16 => i16, i8;
    i32 => i32, i16, i8;
    i64 => i64, i32, i16, i8;
    i128 => i128, i64, i32, i16, i8;
    isize => isize, i16, i8;
}

c0nst::c0nst! {
/// Fallible value-preserving conversion between any two integer types
/// (the trait counterpart of std's generic `checked_cast`).
pub c0nst trait CheckedCast<T>: Sized {
    /// Converts `self` to the target type, returning `None` if the value
    /// doesn't fit.
    ///
    /// ```
    /// use const_num_traits::CheckedCast;
    ///
    /// let ok: Option<i8> = CheckedCast::checked_cast(100u16);
    /// let no: Option<i8> = CheckedCast::checked_cast(300u16);
    /// assert_eq!(ok, Some(100));
    /// assert_eq!(no, None);
    /// ```
    fn checked_cast(self) -> Option<T>;
}
}

c0nst::c0nst! {
/// Value-preserving conversion between any two integer types that panics on
/// overflow, even in release builds (std's generic `strict_cast`).
pub c0nst trait StrictCast<T>: Sized {
    /// Converts `self` to the target type, panicking if the value doesn't
    /// fit.
    fn strict_cast(self) -> T;
}
}

c0nst::c0nst! {
/// Wrapping conversion between any two integer types
/// (std's generic `wrapping_cast` — the semantics of `as`).
pub c0nst trait WrappingCast<T>: Sized {
    /// Converts `self` to the target type, wrapping around at the boundary
    /// of the type.
    ///
    /// ```
    /// use const_num_traits::WrappingCast;
    ///
    /// let x: u8 = WrappingCast::wrapping_cast(300u16);
    /// assert_eq!(x, 44);
    /// ```
    fn wrapping_cast(self) -> T;
}
}

c0nst::c0nst! {
/// Saturating conversion between any two integer types
/// (std's generic `saturating_cast`).
pub c0nst trait SaturatingCast<T>: Sized {
    /// Converts `self` to the target type, saturating at the target's
    /// numeric bounds instead of wrapping.
    ///
    /// ```
    /// use const_num_traits::SaturatingCast;
    ///
    /// let x: u8 = SaturatingCast::saturating_cast(300u16);
    /// let y: u8 = SaturatingCast::saturating_cast(-5i32);
    /// assert_eq!(x, 255);
    /// assert_eq!(y, 0);
    /// ```
    fn saturating_cast(self) -> T;
}
}

macro_rules! cast_pair_impl {
    ($from:ty => $($to:ty),*) => {$(
        c0nst::c0nst! {
        c0nst impl CheckedCast<$to> for $from {
            // The round-trip check catches truncation; the sign comparison
            // catches same-width reinterpretation (e.g. 200u8 -> -56i8).
            #[inline]
            #[allow(unused_comparisons)]
            fn checked_cast(self) -> Option<$to> {
                let r = self as $to;
                if (r as $from) == self && ((r < 0) == (self < 0)) {
                    Some(r)
                } else {
                    None
                }
            }
        }
        }

        c0nst::c0nst! {
        c0nst impl StrictCast<$to> for $from {
            #[inline]
            #[track_caller]
            fn strict_cast(self) -> $to {
                match CheckedCast::<$to>::checked_cast(self) {
                    Some(v) => v,
                    None => panic!("attempt to cast integer with overflow"),
                }
            }
        }
        }

        c0nst::c0nst! {
        c0nst impl WrappingCast<$to> for $from {
            #[inline]
            fn wrapping_cast(self) -> $to {
                self as $to
            }
        }
        }

        c0nst::c0nst! {
        c0nst impl SaturatingCast<$to> for $from {
            #[inline]
            #[allow(unused_comparisons)]
            fn saturating_cast(self) -> $to {
                match CheckedCast::<$to>::checked_cast(self) {
                    Some(v) => v,
                    None => {
                        if self < 0 {
                            <$to>::MIN
                        } else {
                            <$to>::MAX
                        }
                    }
                }
            }
        }
        }
    )*};
}

macro_rules! cast_all_impl {
    ($($from:ty),*) => {$(
        cast_pair_impl!($from => u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);
    )*};
}

cast_all_impl!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abs_diff_unsigned_abs() {
        assert_eq!(AbsDiff::abs_diff(100u8, 120), 20);
        assert_eq!(AbsDiff::abs_diff(120u8, 100), 20);
        assert_eq!(AbsDiff::abs_diff(i8::MIN, i8::MAX), 255u8);
        assert_eq!(AbsDiff::abs_diff(-10i32, 10), 20u32);
        assert_eq!(UnsignedAbs::unsigned_abs(i128::MIN), 1u128 << 127);
        assert_eq!(UnsignedAbs::unsigned_abs(-5isize), 5usize);
    }

    #[test]
    fn clamp_magnitude() {
        assert_eq!(ClampMagnitude::clamp_magnitude(120i8, 100), 100);
        assert_eq!(ClampMagnitude::clamp_magnitude(-120i8, 100), -100);
        assert_eq!(ClampMagnitude::clamp_magnitude(50i8, 100), 50);
        // limit doesn't fit in i8: unchanged
        assert_eq!(ClampMagnitude::clamp_magnitude(-120i8, 200), -120);
        assert_eq!(ClampMagnitude::clamp_magnitude(i8::MIN, 127), -127);
    }

    #[test]
    fn sign_casts() {
        assert_eq!(CastSigned::cast_signed(255u8), -1i8);
        assert_eq!(CastSigned::cast_signed(127u8), 127i8);
        assert_eq!(CastUnsigned::cast_unsigned(-1i8), 255u8);
        assert_eq!(CastUnsigned::cast_unsigned(5i8), 5u8);
        // the value-checked variants live in the generic casts:
        let c: Option<i8> = CheckedCast::checked_cast(255u8);
        assert_eq!(c, None);
        let s: i8 = SaturatingCast::saturating_cast(255u8);
        assert_eq!(s, 127);
        let c: Option<u8> = CheckedCast::checked_cast(-1i8);
        assert_eq!(c, None);
        let s: u8 = SaturatingCast::saturating_cast(-1i8);
        assert_eq!(s, 0);
    }

    #[test]
    #[should_panic(expected = "attempt to cast integer with overflow")]
    fn strict_cast_panics() {
        let _: u8 = StrictCast::strict_cast(-1i8);
    }

    #[test]
    fn widen_truncate() {
        let w: u128 = Widen::widen(u8::MAX);
        assert_eq!(w, 255);
        let w: i64 = Widen::widen(i32::MIN);
        assert_eq!(w, i32::MIN as i64);
        let t: u8 = Truncate::truncate(0xABCDu16);
        assert_eq!(t, 0xCD);
        // checked/saturating narrowing via the generic casts
        let t: Option<i8> = CheckedCast::checked_cast(-1000i32);
        assert_eq!(t, None);
        let t: Option<i8> = CheckedCast::checked_cast(-100i32);
        assert_eq!(t, Some(-100));
        let t: i8 = SaturatingCast::saturating_cast(376i32);
        assert_eq!(t, 127);
        let t: i8 = SaturatingCast::saturating_cast(-1000i32);
        assert_eq!(t, -128);
    }

    #[test]
    fn generic_casts() {
        // same-width sign reinterpretations are rejected by checked_cast
        let r: Option<i8> = CheckedCast::checked_cast(200u8);
        assert_eq!(r, None);
        let r: Option<u8> = CheckedCast::checked_cast(-1i8);
        assert_eq!(r, None);
        // narrowing
        let r: Option<u8> = CheckedCast::checked_cast(300u16);
        assert_eq!(r, None);
        let r: Option<u8> = CheckedCast::checked_cast(255u16);
        assert_eq!(r, Some(255));
        // widening with sign change
        let r: Option<u128> = CheckedCast::checked_cast(-1i8);
        assert_eq!(r, None);
        let r: Option<i128> = CheckedCast::checked_cast(u64::MAX);
        assert_eq!(r, Some(u64::MAX as i128));
        // wrapping/saturating
        let r: u8 = WrappingCast::wrapping_cast(-1i32);
        assert_eq!(r, 255);
        let r: u8 = SaturatingCast::saturating_cast(-1i32);
        assert_eq!(r, 0);
        let r: i8 = SaturatingCast::saturating_cast(u128::MAX);
        assert_eq!(r, 127);
        let r: i8 = SaturatingCast::saturating_cast(i128::MIN);
        assert_eq!(r, -128);
        // identity
        let r: Option<usize> = CheckedCast::checked_cast(5usize);
        assert_eq!(r, Some(5));
    }

    #[test]
    fn generic_cast_exhaustive_u16_i8() {
        // brute-force agreement with TryInto for one representative pair
        for x in 0u16..=u16::MAX {
            let checked: Option<i8> = CheckedCast::checked_cast(x);
            let expected: Option<i8> = TryInto::try_into(x).ok();
            assert_eq!(checked, expected, "{x}");
        }
        for x in i16::MIN..=i16::MAX {
            let checked: Option<u8> = CheckedCast::checked_cast(x);
            let expected: Option<u8> = TryInto::try_into(x).ok();
            assert_eq!(checked, expected, "{x}");
        }
    }
}
