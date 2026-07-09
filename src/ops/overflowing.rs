//! Overflowing arithmetic: the wrapped value plus an overflow flag.
//!
//! **CT tier A (CT-implementable)** for add/sub/mul/neg/shifts/abs: the flag is
//! designed to be consumed arithmetically (`carry as T`), as in multi-word
//! arithmetic; convert it to a mask rather than branching on it in constant-time
//! code. `OverflowingDiv`/`OverflowingRem` are CT-hostile (data-dependent
//! division) and `OverflowingPow` is exponent-dependent — Tier C for secrets.

macro_rules! overflowing_impl {
    ($trait_name:ident, $method:ident, $t:ty) => {
        c0nst::c0nst! {
        c0nst impl $trait_name for $t {
            type Output = $t;
            #[inline]
            fn $method(self, v: Self) -> (Self, bool) {
                <$t>::$method(self, v)
            }
        }
        }
    };
}

c0nst::c0nst! {
/// Performs addition with a flag for overflow.
pub c0nst trait OverflowingAdd: Sized {
    /// The result type (`Self` for the primitive impls).
    type Output;
    /// Returns a tuple of the sum along with a boolean indicating whether an arithmetic overflow would occur.
    /// If an overflow would have occurred then the wrapped value is returned.
    fn overflowing_add(self, v: Self) -> (Self::Output, bool);
}
}

overflowing_impl!(OverflowingAdd, overflowing_add, u8);
overflowing_impl!(OverflowingAdd, overflowing_add, u16);
overflowing_impl!(OverflowingAdd, overflowing_add, u32);
overflowing_impl!(OverflowingAdd, overflowing_add, u64);
overflowing_impl!(OverflowingAdd, overflowing_add, usize);
overflowing_impl!(OverflowingAdd, overflowing_add, u128);

overflowing_impl!(OverflowingAdd, overflowing_add, i8);
overflowing_impl!(OverflowingAdd, overflowing_add, i16);
overflowing_impl!(OverflowingAdd, overflowing_add, i32);
overflowing_impl!(OverflowingAdd, overflowing_add, i64);
overflowing_impl!(OverflowingAdd, overflowing_add, isize);
overflowing_impl!(OverflowingAdd, overflowing_add, i128);

c0nst::c0nst! {
/// Performs substraction with a flag for overflow.
pub c0nst trait OverflowingSub: Sized {
    /// The result type (`Self` for the primitive impls).
    type Output;
    /// Returns a tuple of the difference along with a boolean indicating whether an arithmetic overflow would occur.
    /// If an overflow would have occurred then the wrapped value is returned.
    fn overflowing_sub(self, v: Self) -> (Self::Output, bool);
}
}

overflowing_impl!(OverflowingSub, overflowing_sub, u8);
overflowing_impl!(OverflowingSub, overflowing_sub, u16);
overflowing_impl!(OverflowingSub, overflowing_sub, u32);
overflowing_impl!(OverflowingSub, overflowing_sub, u64);
overflowing_impl!(OverflowingSub, overflowing_sub, usize);
overflowing_impl!(OverflowingSub, overflowing_sub, u128);

overflowing_impl!(OverflowingSub, overflowing_sub, i8);
overflowing_impl!(OverflowingSub, overflowing_sub, i16);
overflowing_impl!(OverflowingSub, overflowing_sub, i32);
overflowing_impl!(OverflowingSub, overflowing_sub, i64);
overflowing_impl!(OverflowingSub, overflowing_sub, isize);
overflowing_impl!(OverflowingSub, overflowing_sub, i128);

c0nst::c0nst! {
/// Performs multiplication with a flag for overflow.
pub c0nst trait OverflowingMul: Sized {
    /// The result type (`Self` for the primitive impls).
    type Output;
    /// Returns a tuple of the product along with a boolean indicating whether an arithmetic overflow would occur.
    /// If an overflow would have occurred then the wrapped value is returned.
    fn overflowing_mul(self, v: Self) -> (Self::Output, bool);
}
}

overflowing_impl!(OverflowingMul, overflowing_mul, u8);
overflowing_impl!(OverflowingMul, overflowing_mul, u16);
overflowing_impl!(OverflowingMul, overflowing_mul, u32);
overflowing_impl!(OverflowingMul, overflowing_mul, u64);
overflowing_impl!(OverflowingMul, overflowing_mul, usize);
overflowing_impl!(OverflowingMul, overflowing_mul, u128);

overflowing_impl!(OverflowingMul, overflowing_mul, i8);
overflowing_impl!(OverflowingMul, overflowing_mul, i16);
overflowing_impl!(OverflowingMul, overflowing_mul, i32);
overflowing_impl!(OverflowingMul, overflowing_mul, i64);
overflowing_impl!(OverflowingMul, overflowing_mul, isize);
overflowing_impl!(OverflowingMul, overflowing_mul, i128);

c0nst::c0nst! {
/// Performs division with a flag for overflow.
pub c0nst trait OverflowingDiv: Sized {
    /// The result type (`Self` for the primitive impls).
    type Output;
    /// Returns a tuple of the quotient along with a boolean indicating whether
    /// an arithmetic overflow would occur. The only overflowing case is
    /// `MIN / -1` on a signed type, where the wrapped value is returned.
    ///
    /// # Panics
    ///
    /// Panics if `v` is zero.
    fn overflowing_div(self, v: Self) -> (Self::Output, bool);
}
}

overflowing_impl!(OverflowingDiv, overflowing_div, u8);
overflowing_impl!(OverflowingDiv, overflowing_div, u16);
overflowing_impl!(OverflowingDiv, overflowing_div, u32);
overflowing_impl!(OverflowingDiv, overflowing_div, u64);
overflowing_impl!(OverflowingDiv, overflowing_div, usize);
overflowing_impl!(OverflowingDiv, overflowing_div, u128);

overflowing_impl!(OverflowingDiv, overflowing_div, i8);
overflowing_impl!(OverflowingDiv, overflowing_div, i16);
overflowing_impl!(OverflowingDiv, overflowing_div, i32);
overflowing_impl!(OverflowingDiv, overflowing_div, i64);
overflowing_impl!(OverflowingDiv, overflowing_div, isize);
overflowing_impl!(OverflowingDiv, overflowing_div, i128);

c0nst::c0nst! {
/// Performs a remainder operation with a flag for overflow.
pub c0nst trait OverflowingRem: Sized {
    /// The result type (`Self` for the primitive impls).
    type Output;
    /// Returns a tuple of the remainder along with a boolean indicating
    /// whether an arithmetic overflow would occur. The only overflowing case
    /// is `MIN % -1` on a signed type, where the remainder is 0.
    ///
    /// # Panics
    ///
    /// Panics if `v` is zero.
    fn overflowing_rem(self, v: Self) -> (Self::Output, bool);
}
}

overflowing_impl!(OverflowingRem, overflowing_rem, u8);
overflowing_impl!(OverflowingRem, overflowing_rem, u16);
overflowing_impl!(OverflowingRem, overflowing_rem, u32);
overflowing_impl!(OverflowingRem, overflowing_rem, u64);
overflowing_impl!(OverflowingRem, overflowing_rem, usize);
overflowing_impl!(OverflowingRem, overflowing_rem, u128);

overflowing_impl!(OverflowingRem, overflowing_rem, i8);
overflowing_impl!(OverflowingRem, overflowing_rem, i16);
overflowing_impl!(OverflowingRem, overflowing_rem, i32);
overflowing_impl!(OverflowingRem, overflowing_rem, i64);
overflowing_impl!(OverflowingRem, overflowing_rem, isize);
overflowing_impl!(OverflowingRem, overflowing_rem, i128);

macro_rules! overflowing_neg_impl {
    ($($t:ty)*) => {$(
        c0nst::c0nst! {
        c0nst impl OverflowingNeg for $t {
            type Output = $t;
            #[inline]
            fn overflowing_neg(self) -> ($t, bool) { <$t>::overflowing_neg(self) }
        }
        }
    )*};
}

macro_rules! overflowing_unary_impl {
    ($trait_name:ident, $method:ident, $t:ty) => {
        c0nst::c0nst! {
        c0nst impl $trait_name for $t {
            type Output = $t;
            #[inline]
            fn $method(self) -> ($t, bool) {
                <$t>::$method(self)
            }
        }
        }
    };
}

c0nst::c0nst! {
/// Performs negation with a flag for overflow.
pub c0nst trait OverflowingNeg: Sized {
    /// Returns a tuple of the negated value along with a boolean indicating
    /// whether an arithmetic overflow would occur. For unsigned types, any
    /// nonzero value overflows; for signed types only `MIN` does.
    type Output;
    fn overflowing_neg(self) -> (Self::Output, bool);
}
}

overflowing_neg_impl!(u8 u16 u32 u64 usize u128 i8 i16 i32 i64 isize i128);

c0nst::c0nst! {
/// Computes the absolute value with a flag for overflow.
pub c0nst trait OverflowingAbs: Sized {
    /// The result type (`Self` for the primitive impls).
    type Output;
    /// Returns a tuple of the absolute value along with a boolean indicating
    /// whether an arithmetic overflow would occur. The only overflowing case
    /// is `MIN.overflowing_abs()` which returns `(MIN, true)`.
    fn overflowing_abs(self) -> (Self::Output, bool);
}
}

overflowing_unary_impl!(OverflowingAbs, overflowing_abs, i8);
overflowing_unary_impl!(OverflowingAbs, overflowing_abs, i16);
overflowing_unary_impl!(OverflowingAbs, overflowing_abs, i32);
overflowing_unary_impl!(OverflowingAbs, overflowing_abs, i64);
overflowing_unary_impl!(OverflowingAbs, overflowing_abs, isize);
overflowing_unary_impl!(OverflowingAbs, overflowing_abs, i128);

macro_rules! overflowing_u32_rhs_impl {
    ($trait_name:ident, $method:ident, $t:ty) => {
        c0nst::c0nst! {
        c0nst impl $trait_name for $t {
            type Output = $t;
            #[inline]
            fn $method(self, rhs: u32) -> ($t, bool) {
                <$t>::$method(self, rhs)
            }
        }
        }
    };
}

c0nst::c0nst! {
/// Performs a left shift with a flag for overflow.
pub c0nst trait OverflowingShl: Sized {
    /// The result type (`Self` for the primitive impls).
    type Output;
    /// Shifts `self` left by a `rhs` masked to the bit width of the type, and
    /// returns a boolean indicating whether `rhs` was larger than or equal to
    /// the number of bits.
    ///
    /// ```
    /// use const_num_traits::ops::overflowing::OverflowingShl;
    ///
    /// assert_eq!(OverflowingShl::overflowing_shl(0x1u16, 4), (0x10, false));
    /// assert_eq!(OverflowingShl::overflowing_shl(0x1u16, 20), (0x10, true));
    /// ```
    fn overflowing_shl(self, rhs: u32) -> (Self::Output, bool);
}
}

overflowing_u32_rhs_impl!(OverflowingShl, overflowing_shl, u8);
overflowing_u32_rhs_impl!(OverflowingShl, overflowing_shl, u16);
overflowing_u32_rhs_impl!(OverflowingShl, overflowing_shl, u32);
overflowing_u32_rhs_impl!(OverflowingShl, overflowing_shl, u64);
overflowing_u32_rhs_impl!(OverflowingShl, overflowing_shl, usize);
overflowing_u32_rhs_impl!(OverflowingShl, overflowing_shl, u128);

overflowing_u32_rhs_impl!(OverflowingShl, overflowing_shl, i8);
overflowing_u32_rhs_impl!(OverflowingShl, overflowing_shl, i16);
overflowing_u32_rhs_impl!(OverflowingShl, overflowing_shl, i32);
overflowing_u32_rhs_impl!(OverflowingShl, overflowing_shl, i64);
overflowing_u32_rhs_impl!(OverflowingShl, overflowing_shl, isize);
overflowing_u32_rhs_impl!(OverflowingShl, overflowing_shl, i128);

c0nst::c0nst! {
/// Performs a right shift with a flag for overflow.
pub c0nst trait OverflowingShr: Sized {
    /// The result type (`Self` for the primitive impls).
    type Output;
    /// Shifts `self` right by a `rhs` masked to the bit width of the type, and
    /// returns a boolean indicating whether `rhs` was larger than or equal to
    /// the number of bits.
    ///
    /// ```
    /// use const_num_traits::ops::overflowing::OverflowingShr;
    ///
    /// assert_eq!(OverflowingShr::overflowing_shr(0x10u16, 4), (0x1, false));
    /// assert_eq!(OverflowingShr::overflowing_shr(0x10u16, 20), (0x1, true));
    /// ```
    fn overflowing_shr(self, rhs: u32) -> (Self::Output, bool);
}
}

overflowing_u32_rhs_impl!(OverflowingShr, overflowing_shr, u8);
overflowing_u32_rhs_impl!(OverflowingShr, overflowing_shr, u16);
overflowing_u32_rhs_impl!(OverflowingShr, overflowing_shr, u32);
overflowing_u32_rhs_impl!(OverflowingShr, overflowing_shr, u64);
overflowing_u32_rhs_impl!(OverflowingShr, overflowing_shr, usize);
overflowing_u32_rhs_impl!(OverflowingShr, overflowing_shr, u128);

overflowing_u32_rhs_impl!(OverflowingShr, overflowing_shr, i8);
overflowing_u32_rhs_impl!(OverflowingShr, overflowing_shr, i16);
overflowing_u32_rhs_impl!(OverflowingShr, overflowing_shr, i32);
overflowing_u32_rhs_impl!(OverflowingShr, overflowing_shr, i64);
overflowing_u32_rhs_impl!(OverflowingShr, overflowing_shr, isize);
overflowing_u32_rhs_impl!(OverflowingShr, overflowing_shr, i128);

c0nst::c0nst! {
/// Performs exponentiation with a flag for overflow.
pub c0nst trait OverflowingPow: Sized {
    /// The result type (`Self` for the primitive impls).
    type Output;
    /// Raises `self` to the power of `exp`, returning a tuple of the
    /// (possibly wrapped) result and a boolean indicating whether an
    /// arithmetic overflow occurred.
    fn overflowing_pow(self, exp: u32) -> (Self::Output, bool);
}
}

overflowing_u32_rhs_impl!(OverflowingPow, overflowing_pow, u8);
overflowing_u32_rhs_impl!(OverflowingPow, overflowing_pow, u16);
overflowing_u32_rhs_impl!(OverflowingPow, overflowing_pow, u32);
overflowing_u32_rhs_impl!(OverflowingPow, overflowing_pow, u64);
overflowing_u32_rhs_impl!(OverflowingPow, overflowing_pow, usize);
overflowing_u32_rhs_impl!(OverflowingPow, overflowing_pow, u128);

overflowing_u32_rhs_impl!(OverflowingPow, overflowing_pow, i8);
overflowing_u32_rhs_impl!(OverflowingPow, overflowing_pow, i16);
overflowing_u32_rhs_impl!(OverflowingPow, overflowing_pow, i32);
overflowing_u32_rhs_impl!(OverflowingPow, overflowing_pow, i64);
overflowing_u32_rhs_impl!(OverflowingPow, overflowing_pow, isize);
overflowing_u32_rhs_impl!(OverflowingPow, overflowing_pow, i128);

#[test]
fn test_overflowing_extended_traits() {
    assert_eq!(
        OverflowingDiv::overflowing_div(i8::MIN, -1),
        (i8::MIN, true)
    );
    assert_eq!(OverflowingDiv::overflowing_div(100u8, 7), (14, false));
    assert_eq!(OverflowingRem::overflowing_rem(i8::MIN, -1), (0, true));
    assert_eq!(OverflowingNeg::overflowing_neg(1u8), (255, true));
    assert_eq!(OverflowingNeg::overflowing_neg(0u8), (0, false));
    assert_eq!(OverflowingNeg::overflowing_neg(i8::MIN), (i8::MIN, true));
    assert_eq!(OverflowingAbs::overflowing_abs(i8::MIN), (i8::MIN, true));
    assert_eq!(OverflowingAbs::overflowing_abs(-7i8), (7, false));
    assert_eq!(OverflowingShl::overflowing_shl(1u8, 8), (1, true));
    assert_eq!(OverflowingShr::overflowing_shr(0x80u8, 9), (0x40, true));
    assert_eq!(OverflowingPow::overflowing_pow(3u8, 6), (217, true));
    assert_eq!(OverflowingPow::overflowing_pow(3i32, 4), (81, false));
}

#[test]
fn test_overflowing_traits() {
    fn overflowing_add<T: OverflowingAdd<Output = T>>(a: T, b: T) -> (T, bool) {
        a.overflowing_add(b)
    }
    fn overflowing_sub<T: OverflowingSub<Output = T>>(a: T, b: T) -> (T, bool) {
        a.overflowing_sub(b)
    }
    fn overflowing_mul<T: OverflowingMul<Output = T>>(a: T, b: T) -> (T, bool) {
        a.overflowing_mul(b)
    }
    assert_eq!(overflowing_add(5i16, 2), (7, false));
    assert_eq!(overflowing_add(i16::MAX, 1), (i16::MIN, true));
    assert_eq!(overflowing_sub(5i16, 2), (3, false));
    assert_eq!(overflowing_sub(i16::MIN, 1), (i16::MAX, true));
    assert_eq!(overflowing_mul(5i16, 2), (10, false));
    assert_eq!(overflowing_mul(1_000_000_000i32, 10), (1410065408, true));
}
