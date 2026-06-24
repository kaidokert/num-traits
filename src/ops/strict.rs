//! Strict arithmetic: like the unchecked operators but guaranteed to panic on
//! overflow in every build profile, mirroring the `strict_*` inherent methods
//! on the primitive integer types (stable in std since Rust 1.91).
//!
//! Bodies are hand-rolled on top of the `checked_*`/`overflowing_*` inherent
//! methods (using the same panic messages as std) so the crate's MSRV doesn't
//! have to rise to 1.91.
//!
//! **CT tier C (CT-hostile)**: panicking on overflow is data-dependent
//! control flow by definition.

use core::ops::{Add, Div, Mul, Neg, Rem, Shl, Shr, Sub};

use super::euclid::Euclid;

c0nst::c0nst! {
/// Performs addition that panics on overflow, even in release builds.
pub c0nst trait StrictAdd: Sized + [c0nst] Add<Self> {
    /// Strict addition. Computes `self + v`, panicking on overflow regardless
    /// of whether overflow checks are enabled.
    ///
    /// ```
    /// use const_num_traits::StrictAdd;
    ///
    /// assert_eq!(StrictAdd::strict_add(5u8, 250), 255);
    /// ```
    fn strict_add(self, v: Self) -> <Self as Add<Self>>::Output;
}
}

c0nst::c0nst! {
/// Performs subtraction that panics on overflow, even in release builds.
pub c0nst trait StrictSub: Sized + [c0nst] Sub<Self> {
    /// Strict subtraction. Computes `self - v`, panicking on overflow
    /// regardless of whether overflow checks are enabled.
    fn strict_sub(self, v: Self) -> <Self as Sub<Self>>::Output;
}
}

c0nst::c0nst! {
/// Performs multiplication that panics on overflow, even in release builds.
pub c0nst trait StrictMul: Sized + [c0nst] Mul<Self> {
    /// Strict multiplication. Computes `self * v`, panicking on overflow
    /// regardless of whether overflow checks are enabled.
    fn strict_mul(self, v: Self) -> <Self as Mul<Self>>::Output;
}
}

macro_rules! strict_binary_impl {
    ($trait_name:ident, $method:ident, $overflowing:ident, $msg:expr, $t:ty) => {
        c0nst::c0nst! {
        c0nst impl $trait_name for $t {
            #[inline]
            #[track_caller]
            fn $method(self, v: Self) -> Self {
                let (val, overflowed) = <$t>::$overflowing(self, v);
                if overflowed {
                    panic!($msg)
                }
                val
            }
        }
        }
    };
}

macro_rules! strict_binary_impl_all {
    ($trait_name:ident, $method:ident, $overflowing:ident, $msg:expr) => {
        strict_binary_impl!($trait_name, $method, $overflowing, $msg, u8);
        strict_binary_impl!($trait_name, $method, $overflowing, $msg, u16);
        strict_binary_impl!($trait_name, $method, $overflowing, $msg, u32);
        strict_binary_impl!($trait_name, $method, $overflowing, $msg, u64);
        strict_binary_impl!($trait_name, $method, $overflowing, $msg, usize);
        strict_binary_impl!($trait_name, $method, $overflowing, $msg, u128);
        strict_binary_impl!($trait_name, $method, $overflowing, $msg, i8);
        strict_binary_impl!($trait_name, $method, $overflowing, $msg, i16);
        strict_binary_impl!($trait_name, $method, $overflowing, $msg, i32);
        strict_binary_impl!($trait_name, $method, $overflowing, $msg, i64);
        strict_binary_impl!($trait_name, $method, $overflowing, $msg, isize);
        strict_binary_impl!($trait_name, $method, $overflowing, $msg, i128);
    };
}

strict_binary_impl_all!(
    StrictAdd,
    strict_add,
    overflowing_add,
    "attempt to add with overflow"
);
strict_binary_impl_all!(
    StrictSub,
    strict_sub,
    overflowing_sub,
    "attempt to subtract with overflow"
);
strict_binary_impl_all!(
    StrictMul,
    strict_mul,
    overflowing_mul,
    "attempt to multiply with overflow"
);

c0nst::c0nst! {
/// Performs division that panics on overflow, even in release builds.
pub c0nst trait StrictDiv: Sized + [c0nst] Div<Self> {
    /// Strict division. Computes `self / v`, panicking on overflow (only
    /// possible for `MIN / -1` on a signed type) or if `v` is zero.
    fn strict_div(self, v: Self) -> <Self as Div<Self>>::Output;
}
}

c0nst::c0nst! {
/// Performs a remainder operation that panics on overflow, even in release builds.
pub c0nst trait StrictRem: Sized + [c0nst] Rem<Self> {
    /// Strict remainder. Computes `self % v`, panicking on overflow (only
    /// possible for `MIN % -1` on a signed type) or if `v` is zero.
    fn strict_rem(self, v: Self) -> <Self as Rem<Self>>::Output;
}
}

strict_binary_impl_all!(
    StrictDiv,
    strict_div,
    overflowing_div,
    "attempt to divide with overflow"
);
strict_binary_impl_all!(
    StrictRem,
    strict_rem,
    overflowing_rem,
    "attempt to calculate the remainder with overflow"
);

c0nst::c0nst! {
/// Performs negation that panics on overflow, even in release builds.
pub c0nst trait StrictNeg: Sized {
    /// Strict negation. Computes `-self`, panicking on overflow: for unsigned
    /// types any nonzero value overflows, for signed types only `MIN` does.
    type Output;
    fn strict_neg(self) -> Self::Output;
}
}

macro_rules! strict_neg_impl {
    ($($t:ty)*) => {$(
        c0nst::c0nst! {
        c0nst impl StrictNeg for $t {
            type Output = $t;
            #[inline]
            #[track_caller]
            fn strict_neg(self) -> Self {
                let (val, overflowed) = <$t>::overflowing_neg(self);
                if overflowed {
                    panic!("attempt to negate with overflow")
                }
                val
            }
        }
        }
    )*}
}

strict_neg_impl!(usize u8 u16 u32 u64 u128);
strict_neg_impl!(isize i8 i16 i32 i64 i128);

c0nst::c0nst! {
/// Computes the absolute value, panicking on overflow even in release builds.
pub c0nst trait StrictAbs: Sized + [c0nst] Neg {
    /// Strict absolute value. Computes `self.abs()`, panicking for
    /// `MIN.strict_abs()` (the result can't be represented).
    fn strict_abs(self) -> <Self as Neg>::Output;
}
}

macro_rules! strict_abs_impl {
    ($($t:ty)*) => {$(
        c0nst::c0nst! {
        c0nst impl StrictAbs for $t {
            #[inline]
            #[track_caller]
            fn strict_abs(self) -> Self {
                // std routes strict_abs through the negation overflow panic.
                match <$t>::checked_abs(self) {
                    Some(val) => val,
                    None => panic!("attempt to negate with overflow"),
                }
            }
        }
        }
    )*}
}

strict_abs_impl!(isize i8 i16 i32 i64 i128);

c0nst::c0nst! {
/// Performs a left shift that panics on overflowing shift amounts, even in release builds.
pub c0nst trait StrictShl: Sized + [c0nst] Shl<u32> {
    /// Strict shift left. Computes `self << rhs`, panicking if `rhs` is
    /// larger than or equal to the number of bits in `self`.
    fn strict_shl(self, rhs: u32) -> <Self as Shl<u32>>::Output;
}
}

c0nst::c0nst! {
/// Performs a right shift that panics on overflowing shift amounts, even in release builds.
pub c0nst trait StrictShr: Sized + [c0nst] Shr<u32> {
    /// Strict shift right. Computes `self >> rhs`, panicking if `rhs` is
    /// larger than or equal to the number of bits in `self`.
    fn strict_shr(self, rhs: u32) -> <Self as Shr<u32>>::Output;
}
}

macro_rules! strict_shift_impl {
    ($trait_name:ident, $method:ident, $checked:ident, $msg:expr, $($t:ty)*) => {$(
        c0nst::c0nst! {
        c0nst impl $trait_name for $t {
            #[inline]
            #[track_caller]
            fn $method(self, rhs: u32) -> Self {
                match <$t>::$checked(self, rhs) {
                    Some(val) => val,
                    None => panic!($msg),
                }
            }
        }
        }
    )*}
}

strict_shift_impl!(
    StrictShl,
    strict_shl,
    checked_shl,
    "attempt to shift left with overflow",
    usize u8 u16 u32 u64 u128 isize i8 i16 i32 i64 i128
);
strict_shift_impl!(
    StrictShr,
    strict_shr,
    checked_shr,
    "attempt to shift right with overflow",
    usize u8 u16 u32 u64 u128 isize i8 i16 i32 i64 i128
);

c0nst::c0nst! {
/// Performs exponentiation that panics on overflow, even in release builds.
pub c0nst trait StrictPow: Sized + [c0nst] Mul<Self> {
    /// Strict exponentiation. Computes `self.pow(exp)`, panicking on
    /// overflow regardless of whether overflow checks are enabled.
    fn strict_pow(self, exp: u32) -> <Self as Mul<Self>>::Output;
}
}

strict_shift_impl!(
    StrictPow,
    strict_pow,
    checked_pow,
    "attempt to exponentiate with overflow",
    usize u8 u16 u32 u64 u128 isize i8 i16 i32 i64 i128
);

c0nst::c0nst! {
/// Performs Euclidean division and remainder that panic on overflow, even in release builds.
pub c0nst trait StrictEuclid: [c0nst] Euclid {
    /// Strict Euclidean division. Computes `Euclid::div_euclid(self, v)`,
    /// panicking on overflow (only possible for `MIN / -1` on a signed type)
    /// or if `v` is zero.
    fn strict_div_euclid(self, v: Self) -> <Self as Div<Self>>::Output;

    /// Strict Euclidean remainder. Computes `Euclid::rem_euclid(self, v)`,
    /// panicking on overflow (only possible for `MIN % -1` on a signed type)
    /// or if `v` is zero.
    fn strict_rem_euclid(self, v: Self) -> <Self as Rem<Self>>::Output;
}
}

macro_rules! strict_euclid_impl {
    ($($t:ty)*) => {$(
        c0nst::c0nst! {
        c0nst impl StrictEuclid for $t {
            #[inline]
            #[track_caller]
            fn strict_div_euclid(self, v: $t) -> Self {
                let (val, overflowed) = <$t>::overflowing_div_euclid(self, v);
                if overflowed {
                    panic!("attempt to divide with overflow")
                }
                val
            }

            #[inline]
            #[track_caller]
            fn strict_rem_euclid(self, v: $t) -> Self {
                let (val, overflowed) = <$t>::overflowing_rem_euclid(self, v);
                if overflowed {
                    panic!("attempt to calculate the remainder with overflow")
                }
                val
            }
        }
        }
    )*}
}

strict_euclid_impl!(usize u8 u16 u32 u64 u128);
strict_euclid_impl!(isize i8 i16 i32 i64 i128);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strict_ok_paths() {
        assert_eq!(StrictAdd::strict_add(250u8, 5), 255);
        assert_eq!(StrictSub::strict_sub(5i8, 10), -5);
        assert_eq!(StrictMul::strict_mul(16u8, 15), 240);
        assert_eq!(StrictDiv::strict_div(100i32, -3), -33);
        assert_eq!(StrictRem::strict_rem(100i32, -3), 1);
        assert_eq!(StrictNeg::strict_neg(-100i8), 100);
        assert_eq!(StrictNeg::strict_neg(0u8), 0);
        assert_eq!(StrictAbs::strict_abs(-100i8), 100);
        assert_eq!(StrictShl::strict_shl(1u8, 7), 0x80);
        assert_eq!(StrictShr::strict_shr(0x80u8, 7), 1);
        assert_eq!(StrictPow::strict_pow(3u8, 5), 243);
        assert_eq!(StrictEuclid::strict_div_euclid(-7i32, 4), -2);
        assert_eq!(StrictEuclid::strict_rem_euclid(-7i32, 4), 1);
    }

    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
    fn strict_add_panics() {
        let _ = StrictAdd::strict_add(u8::MAX, 1);
    }

    #[test]
    #[should_panic(expected = "attempt to negate with overflow")]
    fn strict_neg_unsigned_panics() {
        let _ = StrictNeg::strict_neg(1u8);
    }

    #[test]
    #[should_panic(expected = "attempt to negate with overflow")]
    fn strict_abs_panics() {
        let _ = StrictAbs::strict_abs(i8::MIN);
    }

    #[test]
    #[should_panic(expected = "attempt to shift left with overflow")]
    fn strict_shl_panics() {
        let _ = StrictShl::strict_shl(1u8, 8);
    }

    #[test]
    #[should_panic(expected = "attempt to exponentiate with overflow")]
    fn strict_pow_panics() {
        let _ = StrictPow::strict_pow(2u8, 8);
    }

    #[test]
    #[should_panic(expected = "attempt to divide with overflow")]
    fn strict_div_euclid_panics() {
        let _ = StrictEuclid::strict_div_euclid(i8::MIN, -1);
    }
}
