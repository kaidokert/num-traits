//! Euclidean division and remainder.
//!
//! **CT tier C (CT-hostile)**: division is data-dependent on every
//! mainstream CPU; constant-time types should not implement these.

use core::ops::{Div, Rem};

c0nst::c0nst! {
pub c0nst trait Euclid: Sized + [c0nst] Div<Self> + [c0nst] Rem<Self> {
    /// Calculates Euclidean division, the matching method for `rem_euclid`.
    ///
    /// This computes the integer `n` such that
    /// `self = n * v + self.rem_euclid(v)`.
    /// In other words, the result is `self / v` rounded to the integer `n`
    /// such that `self >= n * v`.
    ///
    /// # Examples
    ///
    /// ```
    /// use const_num_traits::Euclid;
    ///
    /// let a: i32 = 7;
    /// let b: i32 = 4;
    /// assert_eq!(Euclid::div_euclid(a, b), 1); // 7 > 4 * 1
    /// assert_eq!(Euclid::div_euclid(-a, b), -2); // -7 >= 4 * -2
    /// assert_eq!(Euclid::div_euclid(a, -b), -1); // 7 >= -4 * -1
    /// assert_eq!(Euclid::div_euclid(-a, -b), 2); // -7 >= -4 * 2
    /// ```
    fn div_euclid(self, v: Self) -> <Self as Div<Self>>::Output;

    /// Calculates the least nonnegative remainder of `self (mod v)`.
    ///
    /// In particular, the return value `r` satisfies `0.0 <= r < v.abs()` in
    /// most cases. However, due to a floating point round-off error it can
    /// result in `r == v.abs()`, violating the mathematical definition, if
    /// `self` is much smaller than `v.abs()` in magnitude and `self < 0.0`.
    /// This result is not an element of the function's codomain, but it is the
    /// closest floating point number in the real numbers and thus fulfills the
    /// property `self == self.div_euclid(v) * v + self.rem_euclid(v)`
    /// approximatively.
    ///
    /// # Examples
    ///
    /// ```
    /// use const_num_traits::Euclid;
    ///
    /// let a: i32 = 7;
    /// let b: i32 = 4;
    /// assert_eq!(Euclid::rem_euclid(a, b), 3);
    /// assert_eq!(Euclid::rem_euclid(-a, b), 1);
    /// assert_eq!(Euclid::rem_euclid(a, -b), 3);
    /// assert_eq!(Euclid::rem_euclid(-a, -b), 1);
    /// ```
    fn rem_euclid(self, v: Self) -> <Self as Rem<Self>>::Output;

    /// Returns both the quotient and remainder from Euclidean division.
    ///
    /// By default, it internally calls both `Euclid::div_euclid` and `Euclid::rem_euclid`,
    /// but it can be overridden in order to implement some optimization.
    ///
    /// # Examples
    ///
    /// ```
    /// # use const_num_traits::Euclid;
    /// let x = 5u8;
    /// let y = 3u8;
    ///
    /// let div = Euclid::div_euclid(x, y);
    /// let rem = Euclid::rem_euclid(x, y);
    ///
    /// assert_eq!((div, rem), Euclid::div_rem_euclid(x, y));
    /// ```
    fn div_rem_euclid(self, v: Self) -> (<Self as Div<Self>>::Output, <Self as Rem<Self>>::Output);
}
}

macro_rules! euclid_forward_impl {
    ($($t:ty)*) => {$(
        c0nst::c0nst! {
        impl c0nst Euclid for $t {
            #[inline]
            fn div_euclid(self, v: $t) -> Self {
                <$t>::div_euclid(self, v)
            }

            #[inline]
            fn rem_euclid(self, v: $t) -> Self {
                <$t>::rem_euclid(self, v)
            }

            #[inline]
            fn div_rem_euclid(self, v: $t) -> ($t, $t) {
                (<$t>::div_euclid(self, v), <$t>::rem_euclid(self, v))
            }
        }
        }
    )*}
}

euclid_forward_impl!(isize i8 i16 i32 i64 i128);
euclid_forward_impl!(usize u8 u16 u32 u64 u128);

// f32/f64 keep plain (non-const) impls: their inherent `div_euclid`/`rem_euclid`
// aren't const fns yet, so a separate non-const macro path is used.
#[cfg(feature = "std")]
macro_rules! euclid_forward_impl_float {
    ($($t:ty)*) => {$(
        impl Euclid for $t {
            #[inline]
            fn div_euclid(self, v: $t) -> Self {
                <$t>::div_euclid(self, v)
            }

            #[inline]
            fn rem_euclid(self, v: $t) -> Self {
                <$t>::rem_euclid(self, v)
            }

            #[inline]
            fn div_rem_euclid(self, v: $t) -> ($t, $t) {
                (<$t>::div_euclid(self, v), <$t>::rem_euclid(self, v))
            }
        }
    )*}
}

#[cfg(feature = "std")]
euclid_forward_impl_float!(f32 f64);

#[cfg(not(feature = "std"))]
impl Euclid for f32 {
    #[inline]
    fn div_euclid(self, v: f32) -> f32 {
        let q = <f32 as crate::float::FloatCore>::trunc(self / v);
        if self % v < 0.0 {
            return if v > 0.0 { q - 1.0 } else { q + 1.0 };
        }
        q
    }

    #[inline]
    fn rem_euclid(self, v: f32) -> f32 {
        let r = self % v;
        if r < 0.0 {
            r + <f32 as crate::float::FloatCore>::abs(v)
        } else {
            r
        }
    }

    #[inline]
    fn div_rem_euclid(self, v: f32) -> (f32, f32) {
        (Euclid::div_euclid(self, v), Euclid::rem_euclid(self, v))
    }
}

#[cfg(not(feature = "std"))]
impl Euclid for f64 {
    #[inline]
    fn div_euclid(self, v: f64) -> f64 {
        let q = <f64 as crate::float::FloatCore>::trunc(self / v);
        if self % v < 0.0 {
            return if v > 0.0 { q - 1.0 } else { q + 1.0 };
        }
        q
    }

    #[inline]
    fn rem_euclid(self, v: f64) -> f64 {
        let r = self % v;
        if r < 0.0 {
            r + <f64 as crate::float::FloatCore>::abs(v)
        } else {
            r
        }
    }

    #[inline]
    fn div_rem_euclid(self, v: f64) -> (f64, f64) {
        (Euclid::div_euclid(self, v), Euclid::rem_euclid(self, v))
    }
}

c0nst::c0nst! {
pub c0nst trait CheckedEuclid: [c0nst] Euclid {
    /// Performs euclid division, returning `None` on division by zero or if
    /// overflow occurred.
    fn checked_div_euclid(self, v: Self) -> Option<<Self as Div<Self>>::Output>;

    /// Finds the euclid remainder of dividing two numbers, returning `None` on
    /// division by zero or if overflow occurred.
    fn checked_rem_euclid(self, v: Self) -> Option<<Self as Rem<Self>>::Output>;

    /// Returns both the quotient and remainder from checked Euclidean division,
    /// returning `None` on division by zero or if overflow occurred.
    ///
    /// # Examples
    ///
    /// ```
    /// # use const_num_traits::CheckedEuclid;
    /// let x = 5u8;
    /// let y = 3u8;
    ///
    /// let div = CheckedEuclid::checked_div_euclid(x, y);
    /// let rem = CheckedEuclid::checked_rem_euclid(x, y);
    ///
    /// assert_eq!(Some((div.unwrap(), rem.unwrap())), CheckedEuclid::checked_div_rem_euclid(x, y));
    /// ```
    fn checked_div_rem_euclid(self, v: Self) -> Option<(<Self as Div<Self>>::Output, <Self as Rem<Self>>::Output)>;
}
}

macro_rules! checked_euclid_forward_impl {
    ($($t:ty)*) => {$(
        c0nst::c0nst! {
        impl c0nst CheckedEuclid for $t {
            #[inline]
            fn checked_div_euclid(self, v: $t) -> Option<Self> {
                <$t>::checked_div_euclid(self, v)
            }

            #[inline]
            fn checked_rem_euclid(self, v: $t) -> Option<Self> {
                <$t>::checked_rem_euclid(self, v)
            }

            // `?` desugars through `Try`/`FromResidual` which aren't const;
            // hand-roll the bail-out via `match`.
            #[inline]
            fn checked_div_rem_euclid(self, v: $t) -> Option<(Self, Self)> {
                let d = match <$t>::checked_div_euclid(self, v) {
                    Some(x) => x,
                    None => return None,
                };
                let r = match <$t>::checked_rem_euclid(self, v) {
                    Some(x) => x,
                    None => return None,
                };
                Some((d, r))
            }
        }
        }
    )*}
}

checked_euclid_forward_impl!(isize i8 i16 i32 i64 i128);
checked_euclid_forward_impl!(usize u8 u16 u32 u64 u128);

c0nst::c0nst! {
/// Performs Euclidean division and remainder that wrap around on overflow.
pub c0nst trait WrappingEuclid: [c0nst] Euclid {
    /// Wrapping Euclidean division. Computes `Euclid::div_euclid(self, v)`,
    /// wrapping around at the boundary of the type. The only wrapping case is
    /// `MIN / -1` on a signed type.
    ///
    /// # Panics
    ///
    /// Panics if `v` is zero.
    fn wrapping_div_euclid(self, v: Self) -> <Self as Div<Self>>::Output;

    /// Wrapping Euclidean remainder. Computes `Euclid::rem_euclid(self, v)`,
    /// wrapping around at the boundary of the type. The only wrapping case is
    /// `MIN % -1` on a signed type, where the remainder is 0.
    ///
    /// # Panics
    ///
    /// Panics if `v` is zero.
    fn wrapping_rem_euclid(self, v: Self) -> <Self as Rem<Self>>::Output;
}
}

macro_rules! wrapping_euclid_forward_impl {
    ($($t:ty)*) => {$(
        c0nst::c0nst! {
        impl c0nst WrappingEuclid for $t {
            #[inline]
            fn wrapping_div_euclid(self, v: $t) -> Self {
                <$t>::wrapping_div_euclid(self, v)
            }

            #[inline]
            fn wrapping_rem_euclid(self, v: $t) -> Self {
                <$t>::wrapping_rem_euclid(self, v)
            }
        }
        }
    )*}
}

wrapping_euclid_forward_impl!(isize i8 i16 i32 i64 i128);
wrapping_euclid_forward_impl!(usize u8 u16 u32 u64 u128);

c0nst::c0nst! {
/// Performs Euclidean division and remainder with a flag for overflow.
pub c0nst trait OverflowingEuclid: [c0nst] Euclid {
    /// Returns a tuple of the Euclidean quotient along with a boolean
    /// indicating whether an arithmetic overflow would occur. The only
    /// overflowing case is `MIN / -1` on a signed type.
    ///
    /// # Panics
    ///
    /// Panics if `v` is zero.
    fn overflowing_div_euclid(self, v: Self) -> (<Self as Div<Self>>::Output, bool);

    /// Returns a tuple of the Euclidean remainder along with a boolean
    /// indicating whether an arithmetic overflow would occur. The only
    /// overflowing case is `MIN % -1` on a signed type, where the remainder
    /// is 0.
    ///
    /// # Panics
    ///
    /// Panics if `v` is zero.
    fn overflowing_rem_euclid(self, v: Self) -> (<Self as Rem<Self>>::Output, bool);
}
}

macro_rules! overflowing_euclid_forward_impl {
    ($($t:ty)*) => {$(
        c0nst::c0nst! {
        impl c0nst OverflowingEuclid for $t {
            #[inline]
            fn overflowing_div_euclid(self, v: $t) -> (Self, bool) {
                <$t>::overflowing_div_euclid(self, v)
            }

            #[inline]
            fn overflowing_rem_euclid(self, v: $t) -> (Self, bool) {
                <$t>::overflowing_rem_euclid(self, v)
            }
        }
        }
    )*}
}

overflowing_euclid_forward_impl!(isize i8 i16 i32 i64 i128);
overflowing_euclid_forward_impl!(usize u8 u16 u32 u64 u128);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn euclid_unsigned() {
        macro_rules! test_euclid {
            ($($t:ident)+) => {
                $(
                    {
                        let x: $t = 10;
                        let y: $t = 3;
                        let div = Euclid::div_euclid(x, y);
                        let rem = Euclid::rem_euclid(x, y);
                        assert_eq!(div, 3);
                        assert_eq!(rem, 1);
                        assert_eq!((div, rem), Euclid::div_rem_euclid(x, y));
                    }
                )+
            };
        }

        test_euclid!(usize u8 u16 u32 u64);
    }

    #[test]
    fn euclid_signed() {
        macro_rules! test_euclid {
            ($($t:ident)+) => {
                $(
                    {
                        let x: $t = 10;
                        let y: $t = -3;
                        assert_eq!(Euclid::div_euclid(x, y), -3);
                        assert_eq!(Euclid::div_euclid(-x, y), 4);
                        assert_eq!(Euclid::rem_euclid(x, y), 1);
                        assert_eq!(Euclid::rem_euclid(-x, y), 2);
                        assert_eq!((Euclid::div_euclid(x, y), Euclid::rem_euclid(x, y)), Euclid::div_rem_euclid(x, y));
                        let x: $t = $t::min_value() + 1;
                        let y: $t = -1;
                        assert_eq!(Euclid::div_euclid(x, y), $t::max_value());
                    }
                )+
            };
        }

        test_euclid!(isize i8 i16 i32 i64 i128);
    }

    #[test]
    #[cfg(feature = "std")]
    fn euclid_float() {
        macro_rules! test_euclid {
            ($($t:ident)+) => {
                $(
                    {
                        let x: $t = 12.1;
                        let y: $t = 3.2;
                        // Uses inherent `EPSILON` const now that `FloatCore` is gone.
                        assert!(Euclid::div_euclid(x, y) * y + Euclid::rem_euclid(x, y) - x
                        <= 46.4 * <$t>::EPSILON);
                        assert!(Euclid::div_euclid(x, -y) * -y + Euclid::rem_euclid(x, -y) - x
                        <= 46.4 * <$t>::EPSILON);
                        assert!(Euclid::div_euclid(-x, y) * y + Euclid::rem_euclid(-x, y) + x
                        <= 46.4 * <$t>::EPSILON);
                        assert!(Euclid::div_euclid(-x, -y) * -y + Euclid::rem_euclid(-x, -y) + x
                        <= 46.4 * <$t>::EPSILON);
                        assert_eq!((Euclid::div_euclid(x, y), Euclid::rem_euclid(x, y)), Euclid::div_rem_euclid(x, y));
                    }
                )+
            };
        }

        test_euclid!(f32 f64);
    }

    #[test]
    fn euclid_wrapping_overflowing() {
        assert_eq!(
            WrappingEuclid::wrapping_div_euclid(i8::MIN, -1),
            i8::MIN
        );
        assert_eq!(WrappingEuclid::wrapping_rem_euclid(i8::MIN, -1), 0);
        assert_eq!(WrappingEuclid::wrapping_div_euclid(-7i32, 4), -2);
        assert_eq!(WrappingEuclid::wrapping_rem_euclid(-7i32, 4), 1);
        assert_eq!(
            OverflowingEuclid::overflowing_div_euclid(i8::MIN, -1),
            (i8::MIN, true)
        );
        assert_eq!(
            OverflowingEuclid::overflowing_rem_euclid(i8::MIN, -1),
            (0, true)
        );
        assert_eq!(
            OverflowingEuclid::overflowing_div_euclid(7u8, 4),
            (1, false)
        );
    }

    #[test]
    fn euclid_checked() {
        macro_rules! test_euclid_checked {
            ($($t:ident)+) => {
                $(
                    {
                        assert_eq!(CheckedEuclid::checked_div_euclid($t::min_value(), -1), None);
                        assert_eq!(CheckedEuclid::checked_rem_euclid($t::min_value(), -1), None);
                        assert_eq!(CheckedEuclid::checked_div_euclid(1, 0), None);
                        assert_eq!(CheckedEuclid::checked_rem_euclid(1, 0), None);
                    }
                )+
            };
        }

        test_euclid_checked!(isize i8 i16 i32 i64 i128);
    }
}
