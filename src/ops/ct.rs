//! Constant-time (masked-return) counterparts of the Tier-B atoms, built on
//! [`subtle`]'s [`Choice`] and [`CtOption`]. Only available with the `ct`
//! cargo feature.
//!
//! The plain Tier-B traits (`CheckedAdd`, `Parity`, `IsPowerOfTwo`, …)
//! return `bool`/`Option`, which invite branching on secret-derived data at
//! the call site. The traits here return `Choice`/`CtOption` instead, so
//! the secret-derived condition stays masked until the caller explicitly
//! unwraps it.
//!
//! Selection and equality are deliberately **not** duplicated here — use
//! `subtle::ConditionallySelectable` and `subtle::ConstantTimeEq` directly.
//!
//! These are plain (never-`const`) traits: `subtle`'s constructors are not
//! `const fn`, so a const port has to wait for subtle. This does not affect
//! the `c0nst` story of the rest of the crate.
//!
//! Note on the primitive impls: "constant time" here means the usual
//! best-effort guarantee for Rust code — branchless source operating on
//! values the compiler has no reason to branch on (the same contract subtle
//! itself provides). Types with stronger needs (e.g. a `Ct`-personality
//! bigint) implement these traits with their own hardened bodies.

use subtle::{Choice, ConstantTimeEq, CtOption};

/// Constant-time zero check.
pub trait CtIsZero {
    /// Returns a masked `true` if `self` is zero.
    ///
    /// ```
    /// use const_num_traits::ops::ct::CtIsZero;
    ///
    /// assert_eq!(0u32.ct_is_zero().unwrap_u8(), 1);
    /// assert_eq!(5u32.ct_is_zero().unwrap_u8(), 0);
    /// ```
    fn ct_is_zero(&self) -> Choice;
}

/// Constant-time parity check (the masked counterpart of [`Parity`]).
///
/// [`Parity`]: crate::Parity
pub trait CtParity {
    /// Returns a masked `true` if `self` is odd.
    fn ct_is_odd(&self) -> Choice;

    /// Returns a masked `true` if `self` is even.
    fn ct_is_even(&self) -> Choice;
}

/// Constant-time power-of-two check (the masked counterpart of
/// [`IsPowerOfTwo`]).
///
/// [`IsPowerOfTwo`]: crate::IsPowerOfTwo
pub trait CtIsPowerOfTwo {
    /// Returns a masked `true` if `self` is a power of two.
    fn ct_is_power_of_two(&self) -> Choice;
}

/// Constant-time checked addition (the masked counterpart of
/// [`CheckedAdd`]).
///
/// [`CheckedAdd`]: crate::CheckedAdd
pub trait CtCheckedAdd: Sized {
    /// Computes `self + v`, returning a [`CtOption`] that is `None`-masked
    /// on overflow.
    ///
    /// ```
    /// use const_num_traits::ops::ct::CtCheckedAdd;
    ///
    /// assert_eq!(250u8.ct_checked_add(&5).unwrap(), 255);
    /// assert!(bool::from(255u8.ct_checked_add(&1).is_none()));
    /// ```
    fn ct_checked_add(&self, v: &Self) -> CtOption<Self>;
}

/// Constant-time checked subtraction (the masked counterpart of
/// [`CheckedSub`]).
///
/// [`CheckedSub`]: crate::CheckedSub
pub trait CtCheckedSub: Sized {
    /// Computes `self - v`, returning a [`CtOption`] that is `None`-masked
    /// on overflow.
    fn ct_checked_sub(&self, v: &Self) -> CtOption<Self>;
}

/// Constant-time checked multiplication (the masked counterpart of
/// [`CheckedMul`]).
///
/// [`CheckedMul`]: crate::CheckedMul
pub trait CtCheckedMul: Sized {
    /// Computes `self * v`, returning a [`CtOption`] that is `None`-masked
    /// on overflow.
    fn ct_checked_mul(&self, v: &Self) -> CtOption<Self>;
}

/// Constant-time checked negation (the masked counterpart of
/// [`CheckedNeg`]).
///
/// [`CheckedNeg`]: crate::CheckedNeg
pub trait CtCheckedNeg: Sized {
    /// Computes `-self`, returning a [`CtOption`] that is `None`-masked when
    /// the result is unrepresentable.
    fn ct_checked_neg(&self) -> CtOption<Self>;
}

/// Constant-time signed difference of unsigned values (the masked
/// counterpart of [`CheckedSignedDiff`]).
///
/// [`CheckedSignedDiff`]: crate::CheckedSignedDiff
pub trait CtCheckedSignedDiff: Sized {
    /// The signed counterpart type.
    type Signed;

    /// Computes `self - rhs` as a signed value, `None`-masked if it doesn't
    /// fit.
    fn ct_checked_signed_diff(&self, rhs: &Self) -> CtOption<Self::Signed>;
}

macro_rules! ct_common_impl {
    ($($t:ty)*) => {$(
        impl CtIsZero for $t {
            #[inline]
            fn ct_is_zero(&self) -> Choice {
                self.ct_eq(&0)
            }
        }

        impl CtParity for $t {
            #[inline]
            fn ct_is_odd(&self) -> Choice {
                Choice::from((*self & 1) as u8)
            }

            #[inline]
            fn ct_is_even(&self) -> Choice {
                Choice::from(((*self & 1) ^ 1) as u8)
            }
        }

        impl CtCheckedAdd for $t {
            #[inline]
            fn ct_checked_add(&self, v: &Self) -> CtOption<Self> {
                let (val, overflow) = <$t>::overflowing_add(*self, *v);
                CtOption::new(val, Choice::from((overflow as u8) ^ 1))
            }
        }

        impl CtCheckedSub for $t {
            #[inline]
            fn ct_checked_sub(&self, v: &Self) -> CtOption<Self> {
                let (val, overflow) = <$t>::overflowing_sub(*self, *v);
                CtOption::new(val, Choice::from((overflow as u8) ^ 1))
            }
        }

        impl CtCheckedMul for $t {
            #[inline]
            fn ct_checked_mul(&self, v: &Self) -> CtOption<Self> {
                let (val, overflow) = <$t>::overflowing_mul(*self, *v);
                CtOption::new(val, Choice::from((overflow as u8) ^ 1))
            }
        }

        impl CtCheckedNeg for $t {
            #[inline]
            fn ct_checked_neg(&self) -> CtOption<Self> {
                let (val, overflow) = <$t>::overflowing_neg(*self);
                CtOption::new(val, Choice::from((overflow as u8) ^ 1))
            }
        }
    )*};
}

ct_common_impl!(usize u8 u16 u32 u64 u128);
ct_common_impl!(isize i8 i16 i32 i64 i128);

macro_rules! ct_unsigned_impl {
    ($($t:ty => $s:ty;)*) => {$(
        impl CtIsPowerOfTwo for $t {
            // popcount is data-independent; compare the count in CT
            #[inline]
            fn ct_is_power_of_two(&self) -> Choice {
                <$t>::count_ones(*self).ct_eq(&1)
            }
        }

        impl CtCheckedSignedDiff for $t {
            type Signed = $s;

            // same algorithm as `checked_signed_diff`, with the overflow
            // flag masked instead of branched on
            #[inline]
            fn ct_checked_signed_diff(&self, rhs: &Self) -> CtOption<$s> {
                let res = <$t>::wrapping_sub(*self, *rhs) as $s;
                let overflow = (*self >= *rhs) == (res < 0);
                CtOption::new(res, Choice::from((overflow as u8) ^ 1))
            }
        }
    )*};
}

ct_unsigned_impl! {
    u8 => i8;
    u16 => i16;
    u32 => i32;
    u64 => i64;
    usize => isize;
    u128 => i128;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ct_predicates() {
        assert_eq!(0u64.ct_is_zero().unwrap_u8(), 1);
        assert_eq!(1u64.ct_is_zero().unwrap_u8(), 0);
        assert_eq!(0i32.ct_is_zero().unwrap_u8(), 1);
        assert_eq!((-1i32).ct_is_zero().unwrap_u8(), 0);
        assert_eq!(3u8.ct_is_odd().unwrap_u8(), 1);
        assert_eq!(3u8.ct_is_even().unwrap_u8(), 0);
        assert_eq!((-3i8).ct_is_odd().unwrap_u8(), 1);
        assert_eq!(16u32.ct_is_power_of_two().unwrap_u8(), 1);
        assert_eq!(0u32.ct_is_power_of_two().unwrap_u8(), 0);
        assert_eq!(10u32.ct_is_power_of_two().unwrap_u8(), 0);
    }

    #[test]
    fn ct_checked_ops() {
        assert_eq!(250u8.ct_checked_add(&5).unwrap(), 255);
        assert!(bool::from(255u8.ct_checked_add(&1).is_none()));
        assert_eq!(5u8.ct_checked_sub(&5).unwrap(), 0);
        assert!(bool::from(0u8.ct_checked_sub(&1).is_none()));
        assert_eq!(16u8.ct_checked_mul(&15).unwrap(), 240);
        assert!(bool::from(16u8.ct_checked_mul(&16).is_none()));
        assert_eq!((-5i8).ct_checked_neg().unwrap(), 5);
        assert!(bool::from(i8::MIN.ct_checked_neg().is_none()));
        assert!(bool::from(1u8.ct_checked_neg().is_none()));
        assert_eq!(0u8.ct_checked_neg().unwrap(), 0);
        // agree with the plain checked ops across a sweep
        for a in 0u8..=255 {
            for b in [0u8, 1, 7, 127, 128, 255] {
                assert_eq!(a.checked_add(b), a.ct_checked_add(&b).into());
                assert_eq!(a.checked_sub(b), a.ct_checked_sub(&b).into());
                assert_eq!(a.checked_mul(b), a.ct_checked_mul(&b).into());
            }
        }
    }

    #[test]
    fn ct_signed_diff() {
        assert_eq!(10u8.ct_checked_signed_diff(&14).unwrap(), -4);
        assert!(bool::from(u8::MAX.ct_checked_signed_diff(&0).is_none()));
        // agree with the plain trait
        use crate::ops::mixed::CheckedSignedDiff;
        for a in 0u8..=255 {
            for b in [0u8, 1, 100, 127, 128, 255] {
                let plain = CheckedSignedDiff::checked_signed_diff(a, b);
                let ct: Option<i8> = a.ct_checked_signed_diff(&b).into();
                assert_eq!(plain, ct, "{a} {b}");
            }
        }
    }
}
