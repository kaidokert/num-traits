//! Checked arithmetic: `None` on overflow.
//!
//! **CT tier B (caller-leaky)** for the branchless checked atoms
//! (add/sub/mul/neg/shifts/abs): the `Option` return invites branching on
//! secret-derived data at the call site. Masked counterparts live in `ops::ct`
//! (cargo feature `ct`). `CheckedDiv`/`CheckedRem` are CT-hostile (data-dependent
//! division) and `CheckedPow` is exponent-dependent — Tier C for secret inputs.

c0nst::c0nst! {
/// Performs addition, returning `None` if overflow occurred.
pub c0nst trait CheckedAdd: Sized {
    /// The result type. `Self` for the primitive impls; a distinct owned type
    /// lets non-`Copy` types implement this via `impl for &Self`.
    type Output;
    /// Adds two numbers, checking for overflow. If overflow happens, `None` is
    /// returned.
    fn checked_add(self, v: Self) -> Option<Self::Output>;
}
}

macro_rules! checked_impl {
    ($trait_name:ident, $method:ident, $t:ty) => {
        c0nst::c0nst! {
        c0nst impl $trait_name for $t {
            type Output = $t;
            #[inline]
            fn $method(self, v: $t) -> Option<$t> {
                <$t>::$method(self, v)
            }
        }
        }
    };
}

checked_impl!(CheckedAdd, checked_add, u8);
checked_impl!(CheckedAdd, checked_add, u16);
checked_impl!(CheckedAdd, checked_add, u32);
checked_impl!(CheckedAdd, checked_add, u64);
checked_impl!(CheckedAdd, checked_add, usize);
checked_impl!(CheckedAdd, checked_add, u128);

checked_impl!(CheckedAdd, checked_add, i8);
checked_impl!(CheckedAdd, checked_add, i16);
checked_impl!(CheckedAdd, checked_add, i32);
checked_impl!(CheckedAdd, checked_add, i64);
checked_impl!(CheckedAdd, checked_add, isize);
checked_impl!(CheckedAdd, checked_add, i128);

c0nst::c0nst! {
/// Performs subtraction, returning `None` if overflow occurred.
pub c0nst trait CheckedSub: Sized {
    /// The result type (`Self` for the primitive impls).
    type Output;
    /// Subtracts two numbers, checking for overflow. If overflow happens,
    /// `None` is returned.
    fn checked_sub(self, v: Self) -> Option<Self::Output>;
}
}

checked_impl!(CheckedSub, checked_sub, u8);
checked_impl!(CheckedSub, checked_sub, u16);
checked_impl!(CheckedSub, checked_sub, u32);
checked_impl!(CheckedSub, checked_sub, u64);
checked_impl!(CheckedSub, checked_sub, usize);
checked_impl!(CheckedSub, checked_sub, u128);

checked_impl!(CheckedSub, checked_sub, i8);
checked_impl!(CheckedSub, checked_sub, i16);
checked_impl!(CheckedSub, checked_sub, i32);
checked_impl!(CheckedSub, checked_sub, i64);
checked_impl!(CheckedSub, checked_sub, isize);
checked_impl!(CheckedSub, checked_sub, i128);

c0nst::c0nst! {
/// Performs multiplication, returning `None` if overflow occurred.
pub c0nst trait CheckedMul: Sized {
    /// The result type (`Self` for the primitive impls).
    type Output;
    /// Multiplies two numbers, checking for overflow. If overflow happens,
    /// `None` is returned.
    fn checked_mul(self, v: Self) -> Option<Self::Output>;
}
}

checked_impl!(CheckedMul, checked_mul, u8);
checked_impl!(CheckedMul, checked_mul, u16);
checked_impl!(CheckedMul, checked_mul, u32);
checked_impl!(CheckedMul, checked_mul, u64);
checked_impl!(CheckedMul, checked_mul, usize);
checked_impl!(CheckedMul, checked_mul, u128);

checked_impl!(CheckedMul, checked_mul, i8);
checked_impl!(CheckedMul, checked_mul, i16);
checked_impl!(CheckedMul, checked_mul, i32);
checked_impl!(CheckedMul, checked_mul, i64);
checked_impl!(CheckedMul, checked_mul, isize);
checked_impl!(CheckedMul, checked_mul, i128);

c0nst::c0nst! {
/// Performs division, returning `None` on division by zero or if overflow
/// occurred.
pub c0nst trait CheckedDiv: Sized {
    /// The result type (`Self` for the primitive impls).
    type Output;
    /// Divides two numbers, checking for overflow and division by
    /// zero. If any of that happens, `None` is returned.
    fn checked_div(self, v: Self) -> Option<Self::Output>;
}
}

checked_impl!(CheckedDiv, checked_div, u8);
checked_impl!(CheckedDiv, checked_div, u16);
checked_impl!(CheckedDiv, checked_div, u32);
checked_impl!(CheckedDiv, checked_div, u64);
checked_impl!(CheckedDiv, checked_div, usize);
checked_impl!(CheckedDiv, checked_div, u128);

checked_impl!(CheckedDiv, checked_div, i8);
checked_impl!(CheckedDiv, checked_div, i16);
checked_impl!(CheckedDiv, checked_div, i32);
checked_impl!(CheckedDiv, checked_div, i64);
checked_impl!(CheckedDiv, checked_div, isize);
checked_impl!(CheckedDiv, checked_div, i128);

c0nst::c0nst! {
/// Performs integral remainder, returning `None` on division by zero or if
/// overflow occurred.
pub c0nst trait CheckedRem: Sized {
    /// The result type (`Self` for the primitive impls).
    type Output;
    /// Finds the remainder of dividing two numbers, checking for overflow and
    /// division by zero. If any of that happens, `None` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use const_num_traits::CheckedRem;
    /// use std::i32::MIN;
    ///
    /// assert_eq!(CheckedRem::checked_rem(10, 7), Some(3));
    /// assert_eq!(CheckedRem::checked_rem(10, -7), Some(3));
    /// assert_eq!(CheckedRem::checked_rem(-10, 7), Some(-3));
    /// assert_eq!(CheckedRem::checked_rem(-10, -7), Some(-3));
    ///
    /// assert_eq!(CheckedRem::checked_rem(10, 0), None);
    ///
    /// assert_eq!(CheckedRem::checked_rem(MIN, 1), Some(0));
    /// assert_eq!(CheckedRem::checked_rem(MIN, -1), None);
    /// ```
    fn checked_rem(self, v: Self) -> Option<Self::Output>;
}
}

checked_impl!(CheckedRem, checked_rem, u8);
checked_impl!(CheckedRem, checked_rem, u16);
checked_impl!(CheckedRem, checked_rem, u32);
checked_impl!(CheckedRem, checked_rem, u64);
checked_impl!(CheckedRem, checked_rem, usize);
checked_impl!(CheckedRem, checked_rem, u128);

checked_impl!(CheckedRem, checked_rem, i8);
checked_impl!(CheckedRem, checked_rem, i16);
checked_impl!(CheckedRem, checked_rem, i32);
checked_impl!(CheckedRem, checked_rem, i64);
checked_impl!(CheckedRem, checked_rem, isize);
checked_impl!(CheckedRem, checked_rem, i128);

macro_rules! checked_impl_unary {
    ($trait_name:ident, $method:ident, $t:ty) => {
        c0nst::c0nst! {
        c0nst impl $trait_name for $t {
            type Output = $t;
            #[inline]
            fn $method(self) -> Option<$t> {
                <$t>::$method(self)
            }
        }
        }
    };
}

c0nst::c0nst! {
/// Performs negation, returning `None` if the result can't be represented.
pub c0nst trait CheckedNeg: Sized {
    /// The negation result type (`Self` for the primitive impls; a distinct
    /// owned type lets non-`Copy` types implement this for references).
    type Output;

    /// Negates a number, returning `None` for results that can't be represented, like signed `MIN`
    /// values that can't be positive, or non-zero unsigned values that can't be negative.
    ///
    /// # Examples
    ///
    /// ```
    /// use const_num_traits::CheckedNeg;
    /// use std::i32::MIN;
    ///
    /// assert_eq!(CheckedNeg::checked_neg(1_i32), Some(-1));
    /// assert_eq!(CheckedNeg::checked_neg(-1_i32), Some(1));
    /// assert_eq!(CheckedNeg::checked_neg(MIN), None);
    ///
    /// assert_eq!(CheckedNeg::checked_neg(0_u32), Some(0));
    /// assert_eq!(CheckedNeg::checked_neg(1_u32), None);
    /// ```
    fn checked_neg(self) -> Option<Self::Output>;
}
}

macro_rules! checked_neg_impl {
    ($($t:ty)*) => {$(
        c0nst::c0nst! {
        c0nst impl CheckedNeg for $t {
            type Output = $t;
            #[inline]
            fn checked_neg(self) -> Option<$t> {
                <$t>::checked_neg(self)
            }
        }
        }
    )*};
}

checked_neg_impl!(u8 u16 u32 u64 usize u128 i8 i16 i32 i64 isize i128);

c0nst::c0nst! {
/// Performs shift left, returning `None` on shifts larger than or equal to
/// the type width.
pub c0nst trait CheckedShl: Sized {
    /// The result type (`Self` for the primitive impls).
    type Output;
    /// Checked shift left. Computes `self << rhs`, returning `None`
    /// if `rhs` is larger than or equal to the number of bits in `self`.
    ///
    /// ```
    /// use const_num_traits::CheckedShl;
    ///
    /// let x: u16 = 0x0001;
    ///
    /// assert_eq!(CheckedShl::checked_shl(x, 0),  Some(0x0001));
    /// assert_eq!(CheckedShl::checked_shl(x, 1),  Some(0x0002));
    /// assert_eq!(CheckedShl::checked_shl(x, 15), Some(0x8000));
    /// assert_eq!(CheckedShl::checked_shl(x, 16), None);
    /// ```
    fn checked_shl(self, rhs: u32) -> Option<Self::Output>;
}
}

macro_rules! checked_shift_impl {
    ($trait_name:ident, $method:ident, $t:ty) => {
        c0nst::c0nst! {
        c0nst impl $trait_name for $t {
            type Output = $t;
            #[inline]
            fn $method(self, rhs: u32) -> Option<$t> {
                <$t>::$method(self, rhs)
            }
        }
        }
    };
}

checked_shift_impl!(CheckedShl, checked_shl, u8);
checked_shift_impl!(CheckedShl, checked_shl, u16);
checked_shift_impl!(CheckedShl, checked_shl, u32);
checked_shift_impl!(CheckedShl, checked_shl, u64);
checked_shift_impl!(CheckedShl, checked_shl, usize);
checked_shift_impl!(CheckedShl, checked_shl, u128);

checked_shift_impl!(CheckedShl, checked_shl, i8);
checked_shift_impl!(CheckedShl, checked_shl, i16);
checked_shift_impl!(CheckedShl, checked_shl, i32);
checked_shift_impl!(CheckedShl, checked_shl, i64);
checked_shift_impl!(CheckedShl, checked_shl, isize);
checked_shift_impl!(CheckedShl, checked_shl, i128);

c0nst::c0nst! {
/// Performs shift right, returning `None` on shifts larger than or equal to
/// the type width.
pub c0nst trait CheckedShr: Sized {
    /// The result type (`Self` for the primitive impls).
    type Output;
    /// Checked shift right. Computes `self >> rhs`, returning `None`
    /// if `rhs` is larger than or equal to the number of bits in `self`.
    ///
    /// ```
    /// use const_num_traits::CheckedShr;
    ///
    /// let x: u16 = 0x8000;
    ///
    /// assert_eq!(CheckedShr::checked_shr(x, 0),  Some(0x8000));
    /// assert_eq!(CheckedShr::checked_shr(x, 1),  Some(0x4000));
    /// assert_eq!(CheckedShr::checked_shr(x, 15), Some(0x0001));
    /// assert_eq!(CheckedShr::checked_shr(x, 16), None);
    /// ```
    fn checked_shr(self, rhs: u32) -> Option<Self::Output>;
}
}

checked_shift_impl!(CheckedShr, checked_shr, u8);
checked_shift_impl!(CheckedShr, checked_shr, u16);
checked_shift_impl!(CheckedShr, checked_shr, u32);
checked_shift_impl!(CheckedShr, checked_shr, u64);
checked_shift_impl!(CheckedShr, checked_shr, usize);
checked_shift_impl!(CheckedShr, checked_shr, u128);

checked_shift_impl!(CheckedShr, checked_shr, i8);
checked_shift_impl!(CheckedShr, checked_shr, i16);
checked_shift_impl!(CheckedShr, checked_shr, i32);
checked_shift_impl!(CheckedShr, checked_shr, i64);
checked_shift_impl!(CheckedShr, checked_shr, isize);
checked_shift_impl!(CheckedShr, checked_shr, i128);

c0nst::c0nst! {
/// Computes the absolute value, returning `None` if it can't be represented.
pub c0nst trait CheckedAbs: Sized {
    /// The result type (`Self` for the primitive impls).
    type Output;
    /// Checked absolute value. Computes `self.abs()`, returning `None` if
    /// `self == MIN` (the result can't be represented).
    ///
    /// # Examples
    ///
    /// ```
    /// use const_num_traits::CheckedAbs;
    ///
    /// assert_eq!(CheckedAbs::checked_abs(-5i32), Some(5));
    /// assert_eq!(CheckedAbs::checked_abs(i32::MIN), None);
    /// ```
    fn checked_abs(self) -> Option<Self::Output>;
}
}

checked_impl_unary!(CheckedAbs, checked_abs, i8);
checked_impl_unary!(CheckedAbs, checked_abs, i16);
checked_impl_unary!(CheckedAbs, checked_abs, i32);
checked_impl_unary!(CheckedAbs, checked_abs, i64);
checked_impl_unary!(CheckedAbs, checked_abs, isize);
checked_impl_unary!(CheckedAbs, checked_abs, i128);

c0nst::c0nst! {
/// Performs exponentiation, returning `None` if overflow occurred.
pub c0nst trait CheckedPow: Sized {
    /// The result type (`Self` for the primitive impls).
    type Output;
    /// Checked exponentiation. Computes `self.pow(exp)`, returning `None` if
    /// overflow occurred.
    ///
    /// Note that `0⁰` returns `Some(1)`, matching the inherent
    /// `checked_pow` on the primitive integer types.
    ///
    /// # Examples
    ///
    /// ```
    /// use const_num_traits::CheckedPow;
    ///
    /// assert_eq!(CheckedPow::checked_pow(2i8, 4), Some(16));
    /// assert_eq!(CheckedPow::checked_pow(7i8, 8), None);
    /// assert_eq!(CheckedPow::checked_pow(0u32, 0), Some(1));
    /// ```
    fn checked_pow(self, exp: u32) -> Option<Self::Output>;
}
}

checked_shift_impl!(CheckedPow, checked_pow, u8);
checked_shift_impl!(CheckedPow, checked_pow, u16);
checked_shift_impl!(CheckedPow, checked_pow, u32);
checked_shift_impl!(CheckedPow, checked_pow, u64);
checked_shift_impl!(CheckedPow, checked_pow, usize);
checked_shift_impl!(CheckedPow, checked_pow, u128);

checked_shift_impl!(CheckedPow, checked_pow, i8);
checked_shift_impl!(CheckedPow, checked_pow, i16);
checked_shift_impl!(CheckedPow, checked_pow, i32);
checked_shift_impl!(CheckedPow, checked_pow, i64);
checked_shift_impl!(CheckedPow, checked_pow, isize);
checked_shift_impl!(CheckedPow, checked_pow, i128);

#[test]
fn test_checked_abs_pow() {
    fn checked_abs<T: CheckedAbs<Output = T>>(a: T) -> Option<T> {
        a.checked_abs()
    }
    fn checked_pow<T: CheckedPow<Output = T>>(a: T, exp: u32) -> Option<T> {
        a.checked_pow(exp)
    }
    assert_eq!(checked_abs(-5i16), Some(5));
    assert_eq!(checked_abs(i16::MIN), None);
    assert_eq!(checked_pow(3u8, 5), Some(243));
    assert_eq!(checked_pow(2u8, 8), None);
    assert_eq!(checked_pow(-3i32, 3), Some(-27));
}
