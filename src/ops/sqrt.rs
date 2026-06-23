//! Integer square root, mirroring the `isqrt` / `checked_isqrt` inherent
//! methods (stable in std since Rust 1.84).
//!
//! Split: `checked_isqrt` exists in std only on signed types
//! (it can't fail for unsigned ones), so it lives in its own signed-only
//! trait rather than forcing an artificial always-`Some` method onto the
//! unsigned impls.
//!
//! **CT tier C (CT-hostile)**: integer square root is an iterative,
//! data-dependent computation.

c0nst::c0nst! {
/// Integer square root.
pub c0nst trait Isqrt: Sized {
    /// Returns the integer square root: the largest integer `r` such that
    /// `r * r <= self`.
    ///
    /// # Panics
    ///
    /// Panics if `self` is negative (signed types only).
    ///
    /// ```
    /// use const_num_traits::Isqrt;
    ///
    /// assert_eq!(Isqrt::isqrt(10u32), 3);
    /// assert_eq!(Isqrt::isqrt(16u32), 4);
    /// ```
    type Output;
    fn isqrt(self) -> Self::Output;
}
}

c0nst::c0nst! {
/// Integer square root of signed values, `None` for negative inputs.
pub c0nst trait CheckedIsqrt: Sized {
    /// Returns the integer square root, or `None` if `self` is negative.
    /// Like std, this is only provided for signed types.
    ///
    /// ```
    /// use const_num_traits::CheckedIsqrt;
    ///
    /// assert_eq!(CheckedIsqrt::checked_isqrt(10i32), Some(3));
    /// assert_eq!(CheckedIsqrt::checked_isqrt(-1i32), None);
    /// ```
    type Output;
    fn checked_isqrt(self) -> Option<Self::Output>;
}
}

macro_rules! isqrt_impl {
    ($($t:ty)*) => {$(
        c0nst::c0nst! {
        c0nst impl Isqrt for $t {
            type Output = $t;
            #[inline]
            fn isqrt(self) -> Self {
                <$t>::isqrt(self)
            }
        }
        }
    )*};
}

macro_rules! checked_isqrt_impl {
    ($($t:ty)*) => {$(
        c0nst::c0nst! {
        c0nst impl CheckedIsqrt for $t {
            type Output = $t;
            #[inline]
            fn checked_isqrt(self) -> Option<Self> {
                <$t>::checked_isqrt(self)
            }
        }
        }
    )*};
}

isqrt_impl!(usize u8 u16 u32 u64 u128);
isqrt_impl!(isize i8 i16 i32 i64 i128);
checked_isqrt_impl!(isize i8 i16 i32 i64 i128);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn isqrt() {
        assert_eq!(Isqrt::isqrt(0u8), 0);
        assert_eq!(Isqrt::isqrt(u128::MAX), (1u128 << 64) - 1);
        assert_eq!(Isqrt::isqrt(99i32), 9);
        assert_eq!(Isqrt::isqrt(100i32), 10);
        assert_eq!(CheckedIsqrt::checked_isqrt(-1i32), None);
        assert_eq!(CheckedIsqrt::checked_isqrt(255i32), Some(15));
    }
}
