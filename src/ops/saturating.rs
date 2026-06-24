//! Saturating arithmetic.
//!
//! **CT tier A (CT-implementable)** for add/sub/mul/neg/abs/pow-style
//! saturation: it is expressible as a branchless select on an overflow mask.
//! `SaturatingDiv` is CT-hostile — it performs data-dependent division and
//! panics on a zero divisor — so it is Tier C for secret inputs.

use core::ops::{Add, Div, Mul, Neg, Sub};

c0nst::c0nst! {
/// Saturating math operations. Deprecated, use `SaturatingAdd`, `SaturatingSub` and
/// `SaturatingMul` instead.
pub c0nst trait Saturating {
    /// Saturating addition operator.
    /// Returns a+b, saturating at the numeric bounds instead of overflowing.
    fn saturating_add(self, v: Self) -> Self;

    /// Saturating subtraction operator.
    /// Returns a-b, saturating at the numeric bounds instead of overflowing.
    fn saturating_sub(self, v: Self) -> Self;
}
}

macro_rules! deprecated_saturating_impl {
    ($trait_name:ident for $($t:ty)*) => {$(
        c0nst::c0nst! {
        c0nst impl $trait_name for $t {
            #[inline]
            fn saturating_add(self, v: Self) -> Self {
                Self::saturating_add(self, v)
            }

            #[inline]
            fn saturating_sub(self, v: Self) -> Self {
                Self::saturating_sub(self, v)
            }
        }
        }
    )*}
}

deprecated_saturating_impl!(Saturating for isize i8 i16 i32 i64 i128);
deprecated_saturating_impl!(Saturating for usize u8 u16 u32 u64 u128);

macro_rules! saturating_impl {
    ($trait_name:ident, $method:ident, $t:ty) => {
        c0nst::c0nst! {
        c0nst impl $trait_name for $t {
            #[inline]
            fn $method(self, v: Self) -> Self {
                <$t>::$method(self, v)
            }
        }
        }
    };
}

c0nst::c0nst! {
/// Performs addition that saturates at the numeric bounds instead of overflowing.
pub c0nst trait SaturatingAdd: Sized + [c0nst] Add<Self> {
    /// Saturating addition. Computes `self + other`, saturating at the relevant high or low boundary of
    /// the type.
    fn saturating_add(self, v: Self) -> <Self as Add<Self>>::Output;
}
}

saturating_impl!(SaturatingAdd, saturating_add, u8);
saturating_impl!(SaturatingAdd, saturating_add, u16);
saturating_impl!(SaturatingAdd, saturating_add, u32);
saturating_impl!(SaturatingAdd, saturating_add, u64);
saturating_impl!(SaturatingAdd, saturating_add, usize);
saturating_impl!(SaturatingAdd, saturating_add, u128);

saturating_impl!(SaturatingAdd, saturating_add, i8);
saturating_impl!(SaturatingAdd, saturating_add, i16);
saturating_impl!(SaturatingAdd, saturating_add, i32);
saturating_impl!(SaturatingAdd, saturating_add, i64);
saturating_impl!(SaturatingAdd, saturating_add, isize);
saturating_impl!(SaturatingAdd, saturating_add, i128);

c0nst::c0nst! {
/// Performs subtraction that saturates at the numeric bounds instead of overflowing.
pub c0nst trait SaturatingSub: Sized + [c0nst] Sub<Self> {
    /// Saturating subtraction. Computes `self - other`, saturating at the relevant high or low boundary of
    /// the type.
    fn saturating_sub(self, v: Self) -> <Self as Sub<Self>>::Output;
}
}

saturating_impl!(SaturatingSub, saturating_sub, u8);
saturating_impl!(SaturatingSub, saturating_sub, u16);
saturating_impl!(SaturatingSub, saturating_sub, u32);
saturating_impl!(SaturatingSub, saturating_sub, u64);
saturating_impl!(SaturatingSub, saturating_sub, usize);
saturating_impl!(SaturatingSub, saturating_sub, u128);

saturating_impl!(SaturatingSub, saturating_sub, i8);
saturating_impl!(SaturatingSub, saturating_sub, i16);
saturating_impl!(SaturatingSub, saturating_sub, i32);
saturating_impl!(SaturatingSub, saturating_sub, i64);
saturating_impl!(SaturatingSub, saturating_sub, isize);
saturating_impl!(SaturatingSub, saturating_sub, i128);

c0nst::c0nst! {
/// Performs multiplication that saturates at the numeric bounds instead of overflowing.
pub c0nst trait SaturatingMul: Sized + [c0nst] Mul<Self> {
    /// Saturating multiplication. Computes `self * other`, saturating at the relevant high or low boundary of
    /// the type.
    fn saturating_mul(self, v: Self) -> <Self as Mul<Self>>::Output;
}
}

saturating_impl!(SaturatingMul, saturating_mul, u8);
saturating_impl!(SaturatingMul, saturating_mul, u16);
saturating_impl!(SaturatingMul, saturating_mul, u32);
saturating_impl!(SaturatingMul, saturating_mul, u64);
saturating_impl!(SaturatingMul, saturating_mul, usize);
saturating_impl!(SaturatingMul, saturating_mul, u128);

saturating_impl!(SaturatingMul, saturating_mul, i8);
saturating_impl!(SaturatingMul, saturating_mul, i16);
saturating_impl!(SaturatingMul, saturating_mul, i32);
saturating_impl!(SaturatingMul, saturating_mul, i64);
saturating_impl!(SaturatingMul, saturating_mul, isize);
saturating_impl!(SaturatingMul, saturating_mul, i128);

c0nst::c0nst! {
/// Performs division that saturates at the numeric bounds instead of overflowing.
pub c0nst trait SaturatingDiv: Sized + [c0nst] Div<Self> {
    /// Saturating division. Computes `self / other`, saturating at the
    /// relevant high or low boundary of the type. The only saturating case is
    /// `MIN / -1` on a signed type, which saturates to `MAX`.
    ///
    /// # Panics
    ///
    /// Panics if `other` is zero.
    ///
    /// ```
    /// use const_num_traits::SaturatingDiv;
    ///
    /// assert_eq!(SaturatingDiv::saturating_div(5i32, 2), 2);
    /// assert_eq!(SaturatingDiv::saturating_div(i32::MIN, -1), i32::MAX);
    /// ```
    fn saturating_div(self, v: Self) -> <Self as Div<Self>>::Output;
}
}

saturating_impl!(SaturatingDiv, saturating_div, u8);
saturating_impl!(SaturatingDiv, saturating_div, u16);
saturating_impl!(SaturatingDiv, saturating_div, u32);
saturating_impl!(SaturatingDiv, saturating_div, u64);
saturating_impl!(SaturatingDiv, saturating_div, usize);
saturating_impl!(SaturatingDiv, saturating_div, u128);

saturating_impl!(SaturatingDiv, saturating_div, i8);
saturating_impl!(SaturatingDiv, saturating_div, i16);
saturating_impl!(SaturatingDiv, saturating_div, i32);
saturating_impl!(SaturatingDiv, saturating_div, i64);
saturating_impl!(SaturatingDiv, saturating_div, isize);
saturating_impl!(SaturatingDiv, saturating_div, i128);

macro_rules! saturating_unary_impl {
    ($trait_name:ident, $method:ident, $t:ty) => {
        c0nst::c0nst! {
        c0nst impl $trait_name for $t {
            #[inline]
            fn $method(self) -> $t {
                <$t>::$method(self)
            }
        }
        }
    };
}

c0nst::c0nst! {
/// Performs negation that saturates at the numeric bounds instead of overflowing.
pub c0nst trait SaturatingNeg: Sized + [c0nst] Neg {
    /// Saturating negation. Computes `-self`, saturating at the numeric
    /// bounds: `MIN.saturating_neg()` is `MAX`.
    ///
    /// ```
    /// use const_num_traits::SaturatingNeg;
    ///
    /// assert_eq!(SaturatingNeg::saturating_neg(-5i32), 5);
    /// assert_eq!(SaturatingNeg::saturating_neg(i32::MIN), i32::MAX);
    /// ```
    fn saturating_neg(self) -> <Self as Neg>::Output;
}
}

saturating_unary_impl!(SaturatingNeg, saturating_neg, i8);
saturating_unary_impl!(SaturatingNeg, saturating_neg, i16);
saturating_unary_impl!(SaturatingNeg, saturating_neg, i32);
saturating_unary_impl!(SaturatingNeg, saturating_neg, i64);
saturating_unary_impl!(SaturatingNeg, saturating_neg, isize);
saturating_unary_impl!(SaturatingNeg, saturating_neg, i128);

c0nst::c0nst! {
/// Computes the absolute value, saturating at the numeric bounds instead of overflowing.
pub c0nst trait SaturatingAbs: Sized + [c0nst] Neg {
    /// Saturating absolute value. Computes `self.abs()`, saturating at the
    /// numeric bounds: `MIN.saturating_abs()` is `MAX`.
    ///
    /// ```
    /// use const_num_traits::SaturatingAbs;
    ///
    /// assert_eq!(SaturatingAbs::saturating_abs(-5i32), 5);
    /// assert_eq!(SaturatingAbs::saturating_abs(i32::MIN), i32::MAX);
    /// ```
    fn saturating_abs(self) -> <Self as Neg>::Output;
}
}

saturating_unary_impl!(SaturatingAbs, saturating_abs, i8);
saturating_unary_impl!(SaturatingAbs, saturating_abs, i16);
saturating_unary_impl!(SaturatingAbs, saturating_abs, i32);
saturating_unary_impl!(SaturatingAbs, saturating_abs, i64);
saturating_unary_impl!(SaturatingAbs, saturating_abs, isize);
saturating_unary_impl!(SaturatingAbs, saturating_abs, i128);

macro_rules! saturating_pow_impl {
    ($t:ty) => {
        c0nst::c0nst! {
        c0nst impl SaturatingPow for $t {
            #[inline]
            fn saturating_pow(self, exp: u32) -> $t {
                <$t>::saturating_pow(self, exp)
            }
        }
        }
    };
}

c0nst::c0nst! {
/// Performs exponentiation that saturates at the numeric bounds instead of overflowing.
pub c0nst trait SaturatingPow: Sized + [c0nst] Mul<Self> {
    /// Saturating exponentiation. Computes `self.pow(exp)`, saturating at the
    /// relevant high or low boundary of the type.
    ///
    /// ```
    /// use const_num_traits::SaturatingPow;
    ///
    /// assert_eq!(SaturatingPow::saturating_pow(4u8, 3), 64);
    /// assert_eq!(SaturatingPow::saturating_pow(4u8, 4), u8::MAX);
    /// assert_eq!(SaturatingPow::saturating_pow(-2i8, 7), i8::MIN);
    /// ```
    fn saturating_pow(self, exp: u32) -> <Self as Mul<Self>>::Output;
}
}

saturating_pow_impl!(u8);
saturating_pow_impl!(u16);
saturating_pow_impl!(u32);
saturating_pow_impl!(u64);
saturating_pow_impl!(usize);
saturating_pow_impl!(u128);

saturating_pow_impl!(i8);
saturating_pow_impl!(i16);
saturating_pow_impl!(i32);
saturating_pow_impl!(i64);
saturating_pow_impl!(isize);
saturating_pow_impl!(i128);

#[test]
fn test_saturating_div_neg_abs_pow() {
    assert_eq!(SaturatingDiv::saturating_div(i8::MIN, -1), i8::MAX);
    assert_eq!(SaturatingDiv::saturating_div(100u8, 7), 14);
    assert_eq!(SaturatingNeg::saturating_neg(i16::MIN), i16::MAX);
    assert_eq!(SaturatingNeg::saturating_neg(7i16), -7);
    assert_eq!(SaturatingAbs::saturating_abs(i16::MIN), i16::MAX);
    assert_eq!(SaturatingAbs::saturating_abs(-7i16), 7);
    assert_eq!(SaturatingPow::saturating_pow(2u16, 17), u16::MAX);
    assert_eq!(SaturatingPow::saturating_pow(-2i16, 15), i16::MIN);
}

#[test]
fn test_saturating_traits() {
    fn saturating_add<T: SaturatingAdd<Output = T>>(a: T, b: T) -> T {
        a.saturating_add(b)
    }
    fn saturating_sub<T: SaturatingSub<Output = T>>(a: T, b: T) -> T {
        a.saturating_sub(b)
    }
    fn saturating_mul<T: SaturatingMul<Output = T>>(a: T, b: T) -> T {
        a.saturating_mul(b)
    }
    assert_eq!(saturating_add(255, 1), 255u8);
    assert_eq!(saturating_add(127, 1), 127i8);
    assert_eq!(saturating_add(-128, -1), -128i8);
    assert_eq!(saturating_sub(0, 1), 0u8);
    assert_eq!(saturating_sub(-128, 1), -128i8);
    assert_eq!(saturating_sub(127, -1), 127i8);
    assert_eq!(saturating_mul(255, 2), 255u8);
    assert_eq!(saturating_mul(127, 2), 127i8);
    assert_eq!(saturating_mul(-128, 2), -128i8);
}
