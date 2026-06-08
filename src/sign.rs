use core::num::Wrapping;
use core::ops::Neg;

use crate::float::FloatCore;
use crate::Num;

c0nst::c0nst! {
/// Useful functions for signed numbers (i.e. numbers that can be negative).
pub c0nst trait Signed: Sized + [c0nst] Num + [c0nst] Neg<Output = Self> {
    /// Computes the absolute value.
    ///
    /// For `f32` and `f64`, `NaN` will be returned if the number is `NaN`.
    ///
    /// For signed integers, `::MIN` will be returned if the number is `::MIN`.
    fn abs(&self) -> Self;

    /// The positive difference of two numbers.
    ///
    /// Returns `zero` if the number is less than or equal to `other`, otherwise the difference
    /// between `self` and `other` is returned.
    fn abs_sub(&self, other: &Self) -> Self;

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
    fn signum(&self) -> Self;

    /// Returns true if the number is positive and false if the number is zero or negative.
    fn is_positive(&self) -> bool;

    /// Returns true if the number is negative and false if the number is zero or positive.
    fn is_negative(&self) -> bool;
}
}

macro_rules! signed_impl {
    ($($t:ty)*) => ($(
        c0nst::c0nst! {
        impl c0nst Signed for $t {
            #[inline]
            fn abs(&self) -> $t {
                if self.is_negative() { -*self } else { *self }
            }

            #[inline]
            fn abs_sub(&self, other: &$t) -> $t {
                if *self <= *other { 0 } else { *self - *other }
            }

            #[inline]
            fn signum(&self) -> $t {
                match *self {
                    n if n > 0 => 1,
                    0 => 0,
                    _ => -1,
                }
            }

            #[inline]
            fn is_positive(&self) -> bool { *self > 0 }

            #[inline]
            fn is_negative(&self) -> bool { *self < 0 }
        }
        }
    )*)
}

signed_impl!(isize i8 i16 i32 i64 i128);

// `Wrapping<T>: Num` is itself a non-const impl (std's `PartialEq` on
// `Wrapping<T>` isn't const), so this stays non-const.
impl<T: Signed> Signed for Wrapping<T>
where
    Wrapping<T>: Num + Neg<Output = Wrapping<T>>,
{
    #[inline]
    fn abs(&self) -> Self {
        Wrapping(self.0.abs())
    }

    #[inline]
    fn abs_sub(&self, other: &Self) -> Self {
        Wrapping(self.0.abs_sub(&other.0))
    }

    #[inline]
    fn signum(&self) -> Self {
        Wrapping(self.0.signum())
    }

    #[inline]
    fn is_positive(&self) -> bool {
        self.0.is_positive()
    }

    #[inline]
    fn is_negative(&self) -> bool {
        self.0.is_negative()
    }
}

// Float Signed stays non-const: `FloatCore::signum` isn't const-callable.
macro_rules! signed_float_impl {
    ($t:ty) => {
        impl Signed for $t {
            /// Computes the absolute value. Returns `NAN` if the number is `NAN`.
            #[inline]
            fn abs(&self) -> $t {
                FloatCore::abs(*self)
            }

            /// The positive difference of two numbers. Returns `0.0` if the number is
            /// less than or equal to `other`, otherwise the difference between`self`
            /// and `other` is returned.
            #[inline]
            fn abs_sub(&self, other: &$t) -> $t {
                if *self <= *other {
                    0.
                } else {
                    *self - *other
                }
            }

            /// # Returns
            ///
            /// - `1.0` if the number is positive, `+0.0` or `INFINITY`
            /// - `-1.0` if the number is negative, `-0.0` or `NEG_INFINITY`
            /// - `NAN` if the number is NaN
            #[inline]
            fn signum(&self) -> $t {
                FloatCore::signum(*self)
            }

            /// Returns `true` if the number is positive, including `+0.0` and `INFINITY`
            #[inline]
            fn is_positive(&self) -> bool {
                FloatCore::is_sign_positive(*self)
            }

            /// Returns `true` if the number is negative, including `-0.0` and `NEG_INFINITY`
            #[inline]
            fn is_negative(&self) -> bool {
                FloatCore::is_sign_negative(*self)
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
pub c0nst fn abs<T: [c0nst] Signed + [c0nst] Destruct>(value: T) -> T {
    value.abs()
}
}

c0nst::c0nst! {
/// The positive difference of two numbers.
///
/// Returns zero if `x` is less than or equal to `y`, otherwise the difference
/// between `x` and `y` is returned.
#[inline(always)]
pub c0nst fn abs_sub<T: [c0nst] Signed + [c0nst] Destruct>(x: T, y: T) -> T {
    x.abs_sub(&y)
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
pub c0nst fn signum<T: [c0nst] Signed + [c0nst] Destruct>(value: T) -> T {
    value.signum()
}
}

c0nst::c0nst! {
/// A trait for values which cannot be negative
pub c0nst trait Unsigned: [c0nst] Num {}
}

macro_rules! empty_trait_impl {
    ($name:ident for $($t:ty)*) => ($(
        c0nst::c0nst! {
        impl c0nst $name for $t {}
        }
    )*)
}

empty_trait_impl!(Unsigned for usize u8 u16 u32 u64 u128);

// Same `Wrapping<T>: Num` story as `Signed`: non-const blanket impl.
impl<T: Unsigned> Unsigned for Wrapping<T> where Wrapping<T>: Num {}

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
