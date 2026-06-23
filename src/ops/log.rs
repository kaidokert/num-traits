//! Integer logarithms, mirroring the `ilog2` / `ilog10` / `ilog` inherent
//! methods (stable in std since Rust 1.67) and their checked variants.
//!
//! Split per base: `ilog2` only needs `leading_zeros` — cheap
//! and constant-time-implementable — while `ilog10` and `ilog` require
//! data-dependent division loops. Bundling them would force the expensive
//! capability onto types that can only offer the cheap one. The panicking
//! and checked variants stay paired within each trait: they share one
//! capability, and the panicking form derives from the checked one.
//!
//! **CT tiers**: [`Ilog2::checked_ilog2`] is Tier B (a `leading_zeros`
//! instruction plus an `Option`); the panicking `ilog2` and all of
//! [`Ilog10`]/[`Ilog`] are Tier C (data-dependent panic / division loops).

c0nst::c0nst! {
/// Base 2 integer logarithm, rounded down.
pub c0nst trait Ilog2: Sized {
    /// Returns the base 2 logarithm of the number, rounded down.
    ///
    /// # Panics
    ///
    /// Panics if the number is zero (or negative, for signed types).
    ///
    /// ```
    /// use const_num_traits::Ilog2;
    ///
    /// assert_eq!(Ilog2::ilog2(32u32), 5);
    /// assert_eq!(Ilog2::ilog2(33u32), 5);
    /// ```
    fn ilog2(self) -> u32;

    /// Returns the base 2 logarithm of the number, rounded down, or `None`
    /// if the number is zero (or negative, for signed types).
    fn checked_ilog2(self) -> Option<u32>;
}
}

c0nst::c0nst! {
/// Base 10 integer logarithm, rounded down.
pub c0nst trait Ilog10: Sized {
    /// Returns the base 10 logarithm of the number, rounded down.
    ///
    /// # Panics
    ///
    /// Panics if the number is zero (or negative, for signed types).
    fn ilog10(self) -> u32;

    /// Returns the base 10 logarithm of the number, rounded down, or `None`
    /// if the number is zero (or negative, for signed types).
    fn checked_ilog10(self) -> Option<u32>;
}
}

c0nst::c0nst! {
/// Integer logarithm with respect to an arbitrary base, rounded down.
pub c0nst trait Ilog: Sized {
    /// Returns the logarithm of the number with respect to `base`, rounded
    /// down.
    ///
    /// # Panics
    ///
    /// Panics if the number is zero (or negative, for signed types), or if
    /// `base` is less than 2.
    ///
    /// ```
    /// use const_num_traits::Ilog;
    ///
    /// assert_eq!(Ilog::ilog(125u32, 5), 3);
    /// ```
    fn ilog(self, base: Self) -> u32;

    /// Returns the logarithm of the number with respect to `base`, rounded
    /// down, or `None` if the number is zero (or negative, for signed
    /// types) or if `base` is less than 2.
    fn checked_ilog(self, base: Self) -> Option<u32>;
}
}

macro_rules! ilog_impl {
    ($($t:ty)*) => {$(
        c0nst::c0nst! {
        c0nst impl Ilog2 for $t {
            #[inline]
            fn ilog2(self) -> u32 {
                <$t>::ilog2(self)
            }

            #[inline]
            fn checked_ilog2(self) -> Option<u32> {
                <$t>::checked_ilog2(self)
            }
        }
        }

        c0nst::c0nst! {
        c0nst impl Ilog10 for $t {
            #[inline]
            fn ilog10(self) -> u32 {
                <$t>::ilog10(self)
            }

            #[inline]
            fn checked_ilog10(self) -> Option<u32> {
                <$t>::checked_ilog10(self)
            }
        }
        }

        c0nst::c0nst! {
        c0nst impl Ilog for $t {
            #[inline]
            fn ilog(self, base: Self) -> u32 {
                <$t>::ilog(self, base)
            }

            #[inline]
            fn checked_ilog(self, base: Self) -> Option<u32> {
                <$t>::checked_ilog(self, base)
            }
        }
        }
    )*};
}

ilog_impl!(usize u8 u16 u32 u64 u128);
ilog_impl!(isize i8 i16 i32 i64 i128);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ilog() {
        assert_eq!(Ilog2::ilog2(1u8), 0);
        assert_eq!(Ilog2::ilog2(u128::MAX), 127);
        assert_eq!(Ilog10::ilog10(999u16), 2);
        assert_eq!(Ilog10::ilog10(1000u16), 3);
        assert_eq!(Ilog::ilog(80i32, 3), 3);
        assert_eq!(Ilog2::checked_ilog2(0u8), None);
        assert_eq!(Ilog2::checked_ilog2(-1i8), None);
        assert_eq!(Ilog10::checked_ilog10(100i64), Some(2));
        assert_eq!(Ilog::checked_ilog(5u8, 1), None);
        assert_eq!(Ilog::checked_ilog(5u8, 5), Some(1));
    }
}
