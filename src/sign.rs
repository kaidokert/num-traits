use core::num::Wrapping;
use core::ops::Neg;

use crate::Zero;
use crate::float::FloatCore;

c0nst::c0nst! {
/// Returns the sign of a number.
///
/// This is the standalone atom for the `signum` capability; [`Signed`]
/// inherits it as a supertrait (the same extraction pattern as
/// `PrimBits`/`PrimInt`).
pub c0nst trait Signum: Sized {
    /// The (owned) sign result type.
    type Output;

    /// Returns the sign of the number.
    ///
    /// For `f32` and `f64`:
    ///
    /// * `1.0` if the number is positive, `+0.0` or `INFINITY`
    /// * `-1.0` if the number is negative, `-0.0` or `NEG_INFINITY`
    /// * `NaN` if the number is `NaN`
    ///
    /// For signed integers:
    ///
    /// * `0` if the number is zero
    /// * `1` if the number is positive
    /// * `-1` if the number is negative
    fn signum(self) -> Self::Output;
}
}

c0nst::c0nst! {
/// Useful functions for signed numbers (i.e. numbers that can be negative).
pub c0nst trait Signed: Sized + [c0nst] Zero + [c0nst] PartialOrd + [c0nst] Neg + [c0nst] Signum {
    /// Computes the absolute value.
    ///
    /// For `f32` and `f64`, `NaN` will be returned if the number is `NaN`.
    ///
    /// For signed integers the result is always non-negative and **total**:
    /// `-::MIN` is unrepresentable, so `abs(::MIN)` saturates to `::MAX` (the
    /// nearest representable magnitude) rather than overflowing. This is
    /// deliberately consistent across const evaluation, debug and release —
    /// unlike the inherent `i32::abs`, which panics/overflows on `::MIN`. For a
    /// different overflow policy use [`WrappingAbs`](crate::WrappingAbs),
    /// [`CheckedAbs`](crate::CheckedAbs) or [`StrictAbs`](crate::StrictAbs); for
    /// a statically-proven total `abs`, use [`NonMin`](crate::NonMin).
    fn abs(self) -> <Self as Neg>::Output;

    /// The positive difference of two numbers.
    ///
    /// Returns `zero` if the number is less than or equal to `other`, otherwise the difference
    /// between `self` and `other` is returned.
    fn abs_sub(self, other: Self) -> <Self as Neg>::Output;

    /// Returns true if the number is positive and false if the number is zero or negative.
    fn is_positive(self) -> bool;

    /// Returns true if the number is negative and false if the number is zero or positive.
    fn is_negative(self) -> bool;
}
}

macro_rules! signed_impl {
    ($($t:ty)*) => ($(
        c0nst::c0nst! {
        c0nst impl Signum for $t {
            type Output = $t;
            #[inline]
            fn signum(self) -> $t {
                match self {
                    n if n > 0 => 1,
                    0 => 0,
                    _ => -1,
                }
            }
        }
        }

        c0nst::c0nst! {
        c0nst impl Signed for $t {
            #[inline]
            fn abs(self) -> $t {
                // Total and identical in const / debug / release: an absolute
                // value is non-negative, so `MIN` saturates to `MAX` (the
                // nearest representable magnitude) rather than overflowing.
                // Use `WrappingAbs`/`CheckedAbs`/`StrictAbs` for other policies.
                self.saturating_abs()
            }

            #[inline]
            fn abs_sub(self, other: $t) -> $t {
                // Saturating subtraction keeps the positive difference total
                // and const/runtime-consistent when `self - other` would
                // overflow (e.g. `MAX - MIN`).
                if self <= other { 0 } else { self.saturating_sub(other) }
            }

            #[inline]
            fn is_positive(self) -> bool { self > 0 }

            #[inline]
            fn is_negative(self) -> bool { self < 0 }
        }
        }
    )*)
}

signed_impl!(isize i8 i16 i32 i64 i128);

// `Wrapping<T>: Num` is itself a non-const impl (std's `PartialEq` on
// `Wrapping<T>` isn't const), so these stay non-const.
impl<T: Signum<Output = T>> Signum for Wrapping<T> {
    type Output = Wrapping<T>;
    #[inline]
    fn signum(self) -> Self {
        Wrapping(self.0.signum())
    }
}

impl<T: Signed + Neg<Output = T> + Signum<Output = T>> Signed for Wrapping<T>
where
    Wrapping<T>: Zero + PartialOrd + Neg<Output = Wrapping<T>>,
{
    #[inline]
    fn abs(self) -> Self {
        Wrapping(self.0.abs())
    }

    #[inline]
    fn abs_sub(self, other: Self) -> Self {
        Wrapping(self.0.abs_sub(other.0))
    }

    #[inline]
    fn is_positive(self) -> bool {
        self.0.is_positive()
    }

    #[inline]
    fn is_negative(self) -> bool {
        self.0.is_negative()
    }
}

// Float Signed/Signum stay non-const: `FloatCore::signum` isn't const-callable.
macro_rules! signed_float_impl {
    ($t:ty) => {
        impl Signum for $t {
            type Output = $t;
            /// # Returns
            ///
            /// - `1.0` if the number is positive, `+0.0` or `INFINITY`
            /// - `-1.0` if the number is negative, `-0.0` or `NEG_INFINITY`
            /// - `NAN` if the number is NaN
            #[inline]
            fn signum(self) -> $t {
                FloatCore::signum(self)
            }
        }

        impl Signed for $t {
            /// Computes the absolute value. Returns `NAN` if the number is `NAN`.
            #[inline]
            fn abs(self) -> $t {
                FloatCore::abs(self)
            }

            /// The positive difference of two numbers. Returns `0.0` if the number is
            /// less than or equal to `other`, otherwise the difference between`self`
            /// and `other` is returned.
            #[inline]
            fn abs_sub(self, other: $t) -> $t {
                if self <= other { 0. } else { self - other }
            }

            /// Returns `true` if the number is positive, including `+0.0` and `INFINITY`
            #[inline]
            fn is_positive(self) -> bool {
                FloatCore::is_sign_positive(self)
            }

            /// Returns `true` if the number is negative, including `-0.0` and `NEG_INFINITY`
            #[inline]
            fn is_negative(self) -> bool {
                FloatCore::is_sign_negative(self)
            }
        }
    };
}

signed_float_impl!(f32);
signed_float_impl!(f64);

c0nst::c0nst! {
/// Computes the absolute value.
///
/// For `f32` and `f64`, `NaN` will be returned if the number is `NaN`
///
/// For signed integers, `::MIN` will be returned if the number is `::MIN`.
#[inline(always)]
pub c0nst fn abs<T: [c0nst] Signed + [c0nst] Neg<Output = T> + [c0nst] Destruct>(value: T) -> T {
    value.abs()
}
}

c0nst::c0nst! {
/// The positive difference of two numbers.
///
/// Returns zero if `x` is less than or equal to `y`, otherwise the difference
/// between `x` and `y` is returned.
#[inline(always)]
pub c0nst fn abs_sub<T: [c0nst] Signed + [c0nst] Neg<Output = T> + [c0nst] Destruct>(x: T, y: T) -> T {
    x.abs_sub(y)
}
}

c0nst::c0nst! {
/// Returns the sign of the number.
///
/// For `f32` and `f64`:
///
/// * `1.0` if the number is positive, `+0.0` or `INFINITY`
/// * `-1.0` if the number is negative, `-0.0` or `NEG_INFINITY`
/// * `NaN` if the number is `NaN`
///
/// For signed integers:
///
/// * `0` if the number is zero
/// * `1` if the number is positive
/// * `-1` if the number is negative
#[inline(always)]
pub c0nst fn signum<T: [c0nst] Signed + [c0nst] Signum<Output = T> + [c0nst] Destruct>(value: T) -> T {
    value.signum()
}
}

c0nst::c0nst! {
/// A trait for values which cannot be negative
pub c0nst trait Unsigned: Sized {}
}

macro_rules! empty_trait_impl {
    ($name:ident for $($t:ty)*) => ($(
        c0nst::c0nst! {
        c0nst impl $name for $t {}
        }
    )*)
}

empty_trait_impl!(Unsigned for usize u8 u16 u32 u64 u128);

// Same `Wrapping<T>: Num` story as `Signed`: non-const blanket impl.
impl<T: Unsigned> Unsigned for Wrapping<T> {}

#[test]
fn unsigned_wrapping_is_unsigned() {
    fn require_unsigned<T: Unsigned>(_: &T) {}
    require_unsigned(&Wrapping(42_u32));
}

#[test]
fn signed_wrapping_is_signed() {
    fn require_signed<T: Signed>(_: &T) {}
    require_signed(&Wrapping(-42));
}

#[test]
fn abs_is_total_and_non_negative() {
    // ordinary cases
    assert_eq!(Signed::abs(-7i32), 7);
    assert_eq!(Signed::abs(7i32), 7);
    assert_eq!(Signed::abs(0i32), 0);
    // the MIN case: saturates to MAX (non-negative, total — never panics)
    assert_eq!(Signed::abs(i8::MIN), i8::MAX);
    assert_eq!(Signed::abs(i32::MIN), i32::MAX);
    assert_eq!(Signed::abs(i64::MIN), i64::MAX);
    // abs_sub stays total when the difference would overflow
    assert_eq!(Signed::abs_sub(i32::MAX, i32::MIN), i32::MAX);
    assert_eq!(Signed::abs_sub(5i32, 8), 0);
    assert_eq!(Signed::abs_sub(8i32, 5), 3);
}
