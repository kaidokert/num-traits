//! Wrapping (modular) arithmetic.
//!
//! **CT tier A (CT-implementable)** for modular add/sub/mul/neg/shifts/abs on
//! the builtin integers: branchless on the data. `WrappingDiv`/`WrappingRem`
//! are CT-hostile (data-dependent division) and `WrappingPow` is
//! exponent-dependent — Tier C for secret inputs.

use core::num::Wrapping;

macro_rules! wrapping_impl {
    ($trait_name:ident, $method:ident, $t:ty) => {
        c0nst::c0nst! {
        c0nst impl $trait_name for $t {
            type Output = $t;
            #[inline]
            fn $method(self, v: Self) -> Self {
                <$t>::$method(self, v)
            }
        }
        }
    };
    ($trait_name:ident, $method:ident, $t:ty, $rhs:ty) => {
        c0nst::c0nst! {
        c0nst impl $trait_name<$rhs> for $t {
            type Output = $t;
            #[inline]
            fn $method(self, v: $rhs) -> Self {
                <$t>::$method(self, v)
            }
        }
        }
    };
}

c0nst::c0nst! {
/// Performs addition that wraps around on overflow.
pub c0nst trait WrappingAdd: Sized {
    /// The result type (`Self` for the primitive impls).
    type Output;
    /// Wrapping (modular) addition. Computes `self + other`, wrapping around at the boundary of
    /// the type.
    fn wrapping_add(self, v: Self) -> Self::Output;
}
}

wrapping_impl!(WrappingAdd, wrapping_add, u8);
wrapping_impl!(WrappingAdd, wrapping_add, u16);
wrapping_impl!(WrappingAdd, wrapping_add, u32);
wrapping_impl!(WrappingAdd, wrapping_add, u64);
wrapping_impl!(WrappingAdd, wrapping_add, usize);
wrapping_impl!(WrappingAdd, wrapping_add, u128);

wrapping_impl!(WrappingAdd, wrapping_add, i8);
wrapping_impl!(WrappingAdd, wrapping_add, i16);
wrapping_impl!(WrappingAdd, wrapping_add, i32);
wrapping_impl!(WrappingAdd, wrapping_add, i64);
wrapping_impl!(WrappingAdd, wrapping_add, isize);
wrapping_impl!(WrappingAdd, wrapping_add, i128);

c0nst::c0nst! {
/// Performs subtraction that wraps around on overflow.
pub c0nst trait WrappingSub: Sized {
    /// The result type (`Self` for the primitive impls).
    type Output;
    /// Wrapping (modular) subtraction. Computes `self - other`, wrapping around at the boundary
    /// of the type.
    fn wrapping_sub(self, v: Self) -> Self::Output;
}
}

wrapping_impl!(WrappingSub, wrapping_sub, u8);
wrapping_impl!(WrappingSub, wrapping_sub, u16);
wrapping_impl!(WrappingSub, wrapping_sub, u32);
wrapping_impl!(WrappingSub, wrapping_sub, u64);
wrapping_impl!(WrappingSub, wrapping_sub, usize);
wrapping_impl!(WrappingSub, wrapping_sub, u128);

wrapping_impl!(WrappingSub, wrapping_sub, i8);
wrapping_impl!(WrappingSub, wrapping_sub, i16);
wrapping_impl!(WrappingSub, wrapping_sub, i32);
wrapping_impl!(WrappingSub, wrapping_sub, i64);
wrapping_impl!(WrappingSub, wrapping_sub, isize);
wrapping_impl!(WrappingSub, wrapping_sub, i128);

c0nst::c0nst! {
/// Performs multiplication that wraps around on overflow.
pub c0nst trait WrappingMul: Sized {
    /// The result type (`Self` for the primitive impls).
    type Output;
    /// Wrapping (modular) multiplication. Computes `self * other`, wrapping around at the boundary
    /// of the type.
    fn wrapping_mul(self, v: Self) -> Self::Output;
}
}

wrapping_impl!(WrappingMul, wrapping_mul, u8);
wrapping_impl!(WrappingMul, wrapping_mul, u16);
wrapping_impl!(WrappingMul, wrapping_mul, u32);
wrapping_impl!(WrappingMul, wrapping_mul, u64);
wrapping_impl!(WrappingMul, wrapping_mul, usize);
wrapping_impl!(WrappingMul, wrapping_mul, u128);

wrapping_impl!(WrappingMul, wrapping_mul, i8);
wrapping_impl!(WrappingMul, wrapping_mul, i16);
wrapping_impl!(WrappingMul, wrapping_mul, i32);
wrapping_impl!(WrappingMul, wrapping_mul, i64);
wrapping_impl!(WrappingMul, wrapping_mul, isize);
wrapping_impl!(WrappingMul, wrapping_mul, i128);

macro_rules! wrapping_neg_impl {
    ($($t:ty)*) => {$(
        c0nst::c0nst! {
        c0nst impl WrappingNeg for $t {
            type Output = $t;
            #[inline]
            fn wrapping_neg(self) -> $t {
                <$t>::wrapping_neg(self)
            }
        }
        }
    )*};
}

macro_rules! wrapping_unary_impl {
    ($trait_name:ident, $method:ident, $t:ty) => {
        c0nst::c0nst! {
        c0nst impl $trait_name for $t {
            type Output = $t;
            #[inline]
            fn $method(self) -> $t {
                <$t>::$method(self)
            }
        }
        }
    };
}

c0nst::c0nst! {
/// Performs a negation that does not panic.
pub c0nst trait WrappingNeg: Sized {
    /// Wrapping (modular) negation. Computes `-self`,
    /// wrapping around at the boundary of the type.
    ///
    /// Since unsigned types do not have negative equivalents
    /// all applications of this function will wrap (except for `-0`).
    /// For values smaller than the corresponding signed type's maximum
    /// the result is the same as casting the corresponding signed value.
    /// Any larger values are equivalent to `MAX + 1 - (val - MAX - 1)` where
    /// `MAX` is the corresponding signed type's maximum.
    ///
    /// ```
    /// use const_num_traits::WrappingNeg;
    ///
    /// assert_eq!(100i8.wrapping_neg(), -100);
    /// assert_eq!((-100i8).wrapping_neg(), 100);
    /// assert_eq!((-128i8).wrapping_neg(), -128); // wrapped!
    /// ```
    type Output;
    fn wrapping_neg(self) -> Self::Output;
}
}

wrapping_neg_impl!(u8 u16 u32 u64 usize u128 i8 i16 i32 i64 isize i128);

macro_rules! wrapping_shift_impl {
    ($trait_name:ident, $method:ident, $t:ty) => {
        c0nst::c0nst! {
        c0nst impl $trait_name for $t {
            type Output = $t;
            #[inline]
            fn $method(self, rhs: u32) -> $t {
                <$t>::$method(self, rhs)
            }
        }
        }
    };
}

c0nst::c0nst! {
/// Performs a left shift that does not panic.
pub c0nst trait WrappingShl: Sized {
    /// The result type (`Self` for the primitive impls).
    type Output;
    /// Panic-free bitwise shift-left; yields `self << mask(rhs)`,
    /// where `mask` removes any high order bits of `rhs` that would
    /// cause the shift to exceed the bitwidth of the type.
    ///
    /// ```
    /// use const_num_traits::WrappingShl;
    ///
    /// let x: u16 = 0x0001;
    ///
    /// assert_eq!(WrappingShl::wrapping_shl(x, 0),  0x0001);
    /// assert_eq!(WrappingShl::wrapping_shl(x, 1),  0x0002);
    /// assert_eq!(WrappingShl::wrapping_shl(x, 15), 0x8000);
    /// assert_eq!(WrappingShl::wrapping_shl(x, 16), 0x0001);
    /// ```
    fn wrapping_shl(self, rhs: u32) -> Self::Output;
}
}

wrapping_shift_impl!(WrappingShl, wrapping_shl, u8);
wrapping_shift_impl!(WrappingShl, wrapping_shl, u16);
wrapping_shift_impl!(WrappingShl, wrapping_shl, u32);
wrapping_shift_impl!(WrappingShl, wrapping_shl, u64);
wrapping_shift_impl!(WrappingShl, wrapping_shl, usize);
wrapping_shift_impl!(WrappingShl, wrapping_shl, u128);

wrapping_shift_impl!(WrappingShl, wrapping_shl, i8);
wrapping_shift_impl!(WrappingShl, wrapping_shl, i16);
wrapping_shift_impl!(WrappingShl, wrapping_shl, i32);
wrapping_shift_impl!(WrappingShl, wrapping_shl, i64);
wrapping_shift_impl!(WrappingShl, wrapping_shl, isize);
wrapping_shift_impl!(WrappingShl, wrapping_shl, i128);

c0nst::c0nst! {
/// Performs a right shift that does not panic.
pub c0nst trait WrappingShr: Sized {
    /// The result type (`Self` for the primitive impls).
    type Output;
    /// Panic-free bitwise shift-right; yields `self >> mask(rhs)`,
    /// where `mask` removes any high order bits of `rhs` that would
    /// cause the shift to exceed the bitwidth of the type.
    ///
    /// ```
    /// use const_num_traits::WrappingShr;
    ///
    /// let x: u16 = 0x8000;
    ///
    /// assert_eq!(WrappingShr::wrapping_shr(x, 0),  0x8000);
    /// assert_eq!(WrappingShr::wrapping_shr(x, 1),  0x4000);
    /// assert_eq!(WrappingShr::wrapping_shr(x, 15), 0x0001);
    /// assert_eq!(WrappingShr::wrapping_shr(x, 16), 0x8000);
    /// ```
    fn wrapping_shr(self, rhs: u32) -> Self::Output;
}
}

wrapping_shift_impl!(WrappingShr, wrapping_shr, u8);
wrapping_shift_impl!(WrappingShr, wrapping_shr, u16);
wrapping_shift_impl!(WrappingShr, wrapping_shr, u32);
wrapping_shift_impl!(WrappingShr, wrapping_shr, u64);
wrapping_shift_impl!(WrappingShr, wrapping_shr, usize);
wrapping_shift_impl!(WrappingShr, wrapping_shr, u128);

wrapping_shift_impl!(WrappingShr, wrapping_shr, i8);
wrapping_shift_impl!(WrappingShr, wrapping_shr, i16);
wrapping_shift_impl!(WrappingShr, wrapping_shr, i32);
wrapping_shift_impl!(WrappingShr, wrapping_shr, i64);
wrapping_shift_impl!(WrappingShr, wrapping_shr, isize);
wrapping_shift_impl!(WrappingShr, wrapping_shr, i128);

c0nst::c0nst! {
/// Performs division that wraps around on overflow.
pub c0nst trait WrappingDiv: Sized {
    /// The result type (`Self` for the primitive impls).
    type Output;
    /// Wrapping (modular) division. Computes `self / other`, wrapping around
    /// at the boundary of the type.
    ///
    /// The only case where such wrapping can occur is when one divides
    /// `MIN / -1` on a signed type; this is equivalent to `-MIN`, a positive
    /// value that is too large to represent in the type. For unsigned types
    /// this is a normal division and can never wrap.
    ///
    /// # Panics
    ///
    /// Panics if `other` is zero.
    ///
    /// ```
    /// use const_num_traits::WrappingDiv;
    ///
    /// assert_eq!(WrappingDiv::wrapping_div(100i8, 10), 10);
    /// assert_eq!(WrappingDiv::wrapping_div(i8::MIN, -1), i8::MIN); // wrapped!
    /// ```
    fn wrapping_div(self, v: Self) -> Self::Output;
}
}

wrapping_impl!(WrappingDiv, wrapping_div, u8);
wrapping_impl!(WrappingDiv, wrapping_div, u16);
wrapping_impl!(WrappingDiv, wrapping_div, u32);
wrapping_impl!(WrappingDiv, wrapping_div, u64);
wrapping_impl!(WrappingDiv, wrapping_div, usize);
wrapping_impl!(WrappingDiv, wrapping_div, u128);

wrapping_impl!(WrappingDiv, wrapping_div, i8);
wrapping_impl!(WrappingDiv, wrapping_div, i16);
wrapping_impl!(WrappingDiv, wrapping_div, i32);
wrapping_impl!(WrappingDiv, wrapping_div, i64);
wrapping_impl!(WrappingDiv, wrapping_div, isize);
wrapping_impl!(WrappingDiv, wrapping_div, i128);

c0nst::c0nst! {
/// Performs a remainder operation that wraps around on overflow.
pub c0nst trait WrappingRem: Sized {
    /// The result type (`Self` for the primitive impls).
    type Output;
    /// Wrapping (modular) remainder. Computes `self % other`, wrapping around
    /// at the boundary of the type.
    ///
    /// The only case where such wrapping can occur is `MIN % -1` on a signed
    /// type, where the remainder is defined to be 0. For unsigned types this
    /// is a normal remainder and can never wrap.
    ///
    /// # Panics
    ///
    /// Panics if `other` is zero.
    ///
    /// ```
    /// use const_num_traits::WrappingRem;
    ///
    /// assert_eq!(WrappingRem::wrapping_rem(100i8, 10), 0);
    /// assert_eq!(WrappingRem::wrapping_rem(i8::MIN, -1), 0); // wrapped!
    /// ```
    fn wrapping_rem(self, v: Self) -> Self::Output;
}
}

wrapping_impl!(WrappingRem, wrapping_rem, u8);
wrapping_impl!(WrappingRem, wrapping_rem, u16);
wrapping_impl!(WrappingRem, wrapping_rem, u32);
wrapping_impl!(WrappingRem, wrapping_rem, u64);
wrapping_impl!(WrappingRem, wrapping_rem, usize);
wrapping_impl!(WrappingRem, wrapping_rem, u128);

wrapping_impl!(WrappingRem, wrapping_rem, i8);
wrapping_impl!(WrappingRem, wrapping_rem, i16);
wrapping_impl!(WrappingRem, wrapping_rem, i32);
wrapping_impl!(WrappingRem, wrapping_rem, i64);
wrapping_impl!(WrappingRem, wrapping_rem, isize);
wrapping_impl!(WrappingRem, wrapping_rem, i128);

c0nst::c0nst! {
/// Computes the absolute value, wrapping around on overflow.
pub c0nst trait WrappingAbs: Sized {
    /// The result type (`Self` for the primitive impls).
    type Output;
    /// Wrapping (modular) absolute value. Computes `self.abs()`, wrapping
    /// around at the boundary of the type. The only wrapping case is
    /// `MIN.wrapping_abs() == MIN`.
    ///
    /// ```
    /// use const_num_traits::WrappingAbs;
    ///
    /// assert_eq!(WrappingAbs::wrapping_abs(-100i8), 100);
    /// assert_eq!(WrappingAbs::wrapping_abs(i8::MIN), i8::MIN); // wrapped!
    /// ```
    fn wrapping_abs(self) -> Self::Output;
}
}

wrapping_unary_impl!(WrappingAbs, wrapping_abs, i8);
wrapping_unary_impl!(WrappingAbs, wrapping_abs, i16);
wrapping_unary_impl!(WrappingAbs, wrapping_abs, i32);
wrapping_unary_impl!(WrappingAbs, wrapping_abs, i64);
wrapping_unary_impl!(WrappingAbs, wrapping_abs, isize);
wrapping_unary_impl!(WrappingAbs, wrapping_abs, i128);

c0nst::c0nst! {
/// Performs exponentiation that wraps around on overflow.
pub c0nst trait WrappingPow: Sized {
    /// The result type (`Self` for the primitive impls).
    type Output;
    /// Wrapping (modular) exponentiation. Computes `self.pow(exp)`, wrapping
    /// around at the boundary of the type.
    ///
    /// ```
    /// use const_num_traits::WrappingPow;
    ///
    /// assert_eq!(WrappingPow::wrapping_pow(3u8, 5), 243);
    /// assert_eq!(WrappingPow::wrapping_pow(3u8, 6), 217); // wrapped!
    /// ```
    fn wrapping_pow(self, exp: u32) -> Self::Output;
}
}

wrapping_shift_impl!(WrappingPow, wrapping_pow, u8);
wrapping_shift_impl!(WrappingPow, wrapping_pow, u16);
wrapping_shift_impl!(WrappingPow, wrapping_pow, u32);
wrapping_shift_impl!(WrappingPow, wrapping_pow, u64);
wrapping_shift_impl!(WrappingPow, wrapping_pow, usize);
wrapping_shift_impl!(WrappingPow, wrapping_pow, u128);

wrapping_shift_impl!(WrappingPow, wrapping_pow, i8);
wrapping_shift_impl!(WrappingPow, wrapping_pow, i16);
wrapping_shift_impl!(WrappingPow, wrapping_pow, i32);
wrapping_shift_impl!(WrappingPow, wrapping_pow, i64);
wrapping_shift_impl!(WrappingPow, wrapping_pow, isize);
wrapping_shift_impl!(WrappingPow, wrapping_pow, i128);

#[test]
fn test_wrapping_div_rem_abs_pow() {
    assert_eq!(WrappingDiv::wrapping_div(i8::MIN, -1), i8::MIN);
    assert_eq!(WrappingDiv::wrapping_div(100u8, 7), 14);
    assert_eq!(WrappingRem::wrapping_rem(i8::MIN, -1), 0);
    assert_eq!(WrappingRem::wrapping_rem(100u8, 7), 2);
    assert_eq!(WrappingAbs::wrapping_abs(i16::MIN), i16::MIN);
    assert_eq!(WrappingAbs::wrapping_abs(-7i16), 7);
    assert_eq!(WrappingPow::wrapping_pow(3i8, 5), -13);
    assert_eq!(WrappingPow::wrapping_pow(2u16, 16), 0);
}

// Wrapping<T> blanket impls stay non-const: std's `Add`/`Sub`/`Mul`/`Neg`/`Shl`/`Shr`
// impls for `Wrapping<T>` are not const-trait impls (same situation as Num).
impl<T: WrappingAdd<Output = T>> WrappingAdd for Wrapping<T> {
    type Output = Wrapping<T>;
    fn wrapping_add(self, v: Self) -> Self {
        Wrapping(self.0.wrapping_add(v.0))
    }
}
impl<T: WrappingSub<Output = T>> WrappingSub for Wrapping<T> {
    type Output = Wrapping<T>;
    fn wrapping_sub(self, v: Self) -> Self {
        Wrapping(self.0.wrapping_sub(v.0))
    }
}
impl<T: WrappingMul<Output = T>> WrappingMul for Wrapping<T> {
    type Output = Wrapping<T>;
    fn wrapping_mul(self, v: Self) -> Self {
        Wrapping(self.0.wrapping_mul(v.0))
    }
}
impl<T: WrappingNeg<Output = T>> WrappingNeg for Wrapping<T> {
    type Output = Wrapping<T>;
    fn wrapping_neg(self) -> Self {
        Wrapping(self.0.wrapping_neg())
    }
}
impl<T: WrappingShl<Output = T>> WrappingShl for Wrapping<T> {
    type Output = Wrapping<T>;
    fn wrapping_shl(self, rhs: u32) -> Self {
        Wrapping(self.0.wrapping_shl(rhs))
    }
}
impl<T: WrappingShr<Output = T>> WrappingShr for Wrapping<T> {
    type Output = Wrapping<T>;
    fn wrapping_shr(self, rhs: u32) -> Self {
        Wrapping(self.0.wrapping_shr(rhs))
    }
}

#[test]
fn test_wrapping_traits() {
    fn wrapping_add<T: WrappingAdd<Output = T>>(a: T, b: T) -> T {
        a.wrapping_add(b)
    }
    fn wrapping_sub<T: WrappingSub<Output = T>>(a: T, b: T) -> T {
        a.wrapping_sub(b)
    }
    fn wrapping_mul<T: WrappingMul<Output = T>>(a: T, b: T) -> T {
        a.wrapping_mul(b)
    }
    fn wrapping_neg<T: WrappingNeg<Output = T>>(a: T) -> T {
        a.wrapping_neg()
    }
    fn wrapping_shl<T: WrappingShl<Output = T>>(a: T, b: u32) -> T {
        a.wrapping_shl(b)
    }
    fn wrapping_shr<T: WrappingShr<Output = T>>(a: T, b: u32) -> T {
        a.wrapping_shr(b)
    }
    assert_eq!(wrapping_add(255, 1), 0u8);
    assert_eq!(wrapping_sub(0, 1), 255u8);
    assert_eq!(wrapping_mul(255, 2), 254u8);
    assert_eq!(wrapping_neg(255), 1u8);
    assert_eq!(wrapping_shl(255, 8), 255u8);
    assert_eq!(wrapping_shr(255, 8), 255u8);
    assert_eq!(wrapping_add(255, 1), (Wrapping(255u8) + Wrapping(1u8)).0);
    assert_eq!(wrapping_sub(0, 1), (Wrapping(0u8) - Wrapping(1u8)).0);
    assert_eq!(wrapping_mul(255, 2), (Wrapping(255u8) * Wrapping(2u8)).0);
    assert_eq!(wrapping_neg(255), (-Wrapping(255u8)).0);
    assert_eq!(wrapping_shl(255, 8), (Wrapping(255u8) << 8).0);
    assert_eq!(wrapping_shr(255, 8), (Wrapping(255u8) >> 8).0);
}

#[test]
fn wrapping_is_wrappingadd() {
    fn require_wrappingadd<T: WrappingAdd<Output = T>>(_: &T) {}
    require_wrappingadd(&Wrapping(42));
}

#[test]
fn wrapping_is_wrappingsub() {
    fn require_wrappingsub<T: WrappingSub<Output = T>>(_: &T) {}
    require_wrappingsub(&Wrapping(42));
}

#[test]
fn wrapping_is_wrappingmul() {
    fn require_wrappingmul<T: WrappingMul<Output = T>>(_: &T) {}
    require_wrappingmul(&Wrapping(42));
}

#[test]
fn wrapping_is_wrappingneg() {
    fn require_wrappingneg<T: WrappingNeg<Output = T>>(_: &T) {}
    require_wrappingneg(&Wrapping(42));
}

#[test]
fn wrapping_is_wrappingshl() {
    fn require_wrappingshl<T: WrappingShl<Output = T>>(_: &T) {}
    require_wrappingshl(&Wrapping(42));
}

#[test]
fn wrapping_is_wrappingshr() {
    fn require_wrappingshr<T: WrappingShr<Output = T>>(_: &T) {}
    require_wrappingshr(&Wrapping(42));
}
