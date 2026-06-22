//! Mixed-signedness arithmetic: adding/subtracting a signed value to an
//! unsigned one and vice versa, plus the signed difference of unsigned
//! values. Mirrors the `*_add_signed` / `*_sub_signed` / `*_add_unsigned` /
//! `*_sub_unsigned` / `checked_signed_diff` inherent methods.
//!
//! Per DESIGN.md these follow the crate-wide flavor-per-trait convention
//! (`CheckedAddSigned`, `WrappingAddSigned`, …): one method per trait, so a
//! type can implement exactly the flavors it supports — e.g. a
//! constant-time type implements only the `Wrapping*` (Tier A) and
//! `Overflowing*`/`Checked*` (Tier B) members and never has to expose the
//! panicking `Strict*` ones.
//!
//! Stability in std: the `add_signed` and `add/sub_unsigned` families are
//! stable since 1.66 (delegated); the `sub_signed` family (1.90), the
//! `strict_*` variants and `checked_signed_diff` (both 1.91) are newer than
//! the crate's MSRV and hand-rolled with the same algorithm as core.
//!
//! **CT tiers**: the `Wrapping*`/`Overflowing*`/`Saturating*` flavors are
//! Tier A, the `Checked*` flavors Tier B, the `Strict*` flavors Tier C.

/// Declares a one-method mixed-signedness trait with a counterpart
/// associated type.
macro_rules! declare_mixed_trait {
    (
        $(#[$tdoc:meta])*
        trait $name:ident, type $assoc:ident;
        $(#[$mdoc:meta])*
        fn $method:ident -> $ret:ty;
    ) => {
        c0nst::c0nst! {
        $(#[$tdoc])*
        pub c0nst trait $name: Sized {
            /// The counterpart type of opposite signedness.
            type $assoc;

            /// The (owned) result type.
            type Output;

            $(#[$mdoc])*
            fn $method(self, rhs: Self::$assoc) -> $ret;
        }
        }
    };
}

// ---- unsigned receivers, signed right-hand side ----

declare_mixed_trait! {
    /// Checked addition of a signed value to an unsigned value.
    trait CheckedAddSigned, type Signed;
    /// Computes `self + rhs`, returning `None` if overflow occurred.
    ///
    /// ```
    /// use const_num_traits::CheckedAddSigned;
    ///
    /// assert_eq!(CheckedAddSigned::checked_add_signed(1u8, 2), Some(3));
    /// assert_eq!(CheckedAddSigned::checked_add_signed(1u8, -2), None);
    /// ```
    fn checked_add_signed -> Option<Self::Output>;
}

declare_mixed_trait! {
    /// Wrapping (modular) addition of a signed value to an unsigned value.
    trait WrappingAddSigned, type Signed;
    /// Computes `self + rhs`, wrapping around at the boundary of the type.
    fn wrapping_add_signed -> Self::Output;
}

declare_mixed_trait! {
    /// Overflowing addition of a signed value to an unsigned value.
    trait OverflowingAddSigned, type Signed;
    /// Computes `self + rhs`, returning the wrapped sum and whether overflow
    /// occurred.
    fn overflowing_add_signed -> (Self::Output, bool);
}

declare_mixed_trait! {
    /// Saturating addition of a signed value to an unsigned value.
    trait SaturatingAddSigned, type Signed;
    /// Computes `self + rhs`, saturating at the numeric bounds.
    fn saturating_add_signed -> Self::Output;
}

declare_mixed_trait! {
    /// Strict addition of a signed value to an unsigned value.
    trait StrictAddSigned, type Signed;
    /// Computes `self + rhs`, panicking on overflow even in release builds.
    fn strict_add_signed -> Self::Output;
}

declare_mixed_trait! {
    /// Checked subtraction of a signed value from an unsigned value.
    trait CheckedSubSigned, type Signed;
    /// Computes `self - rhs`, returning `None` if overflow occurred.
    ///
    /// ```
    /// use const_num_traits::CheckedSubSigned;
    ///
    /// assert_eq!(CheckedSubSigned::checked_sub_signed(1u8, 2), None);
    /// assert_eq!(CheckedSubSigned::checked_sub_signed(1u8, -2), Some(3));
    /// ```
    fn checked_sub_signed -> Option<Self::Output>;
}

declare_mixed_trait! {
    /// Wrapping (modular) subtraction of a signed value from an unsigned value.
    trait WrappingSubSigned, type Signed;
    /// Computes `self - rhs`, wrapping around at the boundary of the type.
    fn wrapping_sub_signed -> Self::Output;
}

declare_mixed_trait! {
    /// Overflowing subtraction of a signed value from an unsigned value.
    trait OverflowingSubSigned, type Signed;
    /// Computes `self - rhs`, returning the wrapped difference and whether
    /// overflow occurred.
    fn overflowing_sub_signed -> (Self::Output, bool);
}

declare_mixed_trait! {
    /// Saturating subtraction of a signed value from an unsigned value.
    trait SaturatingSubSigned, type Signed;
    /// Computes `self - rhs`, saturating at the numeric bounds.
    fn saturating_sub_signed -> Self::Output;
}

declare_mixed_trait! {
    /// Strict subtraction of a signed value from an unsigned value.
    trait StrictSubSigned, type Signed;
    /// Computes `self - rhs`, panicking on overflow even in release builds.
    fn strict_sub_signed -> Self::Output;
}

// ---- signed receivers, unsigned right-hand side ----

declare_mixed_trait! {
    /// Checked addition of an unsigned value to a signed value.
    trait CheckedAddUnsigned, type Unsigned;
    /// Computes `self + rhs`, returning `None` if overflow occurred.
    ///
    /// ```
    /// use const_num_traits::CheckedAddUnsigned;
    ///
    /// assert_eq!(CheckedAddUnsigned::checked_add_unsigned(1i8, 2), Some(3));
    /// assert_eq!(CheckedAddUnsigned::checked_add_unsigned(i8::MAX, 1), None);
    /// ```
    fn checked_add_unsigned -> Option<Self::Output>;
}

declare_mixed_trait! {
    /// Wrapping (modular) addition of an unsigned value to a signed value.
    trait WrappingAddUnsigned, type Unsigned;
    /// Computes `self + rhs`, wrapping around at the boundary of the type.
    fn wrapping_add_unsigned -> Self::Output;
}

declare_mixed_trait! {
    /// Overflowing addition of an unsigned value to a signed value.
    trait OverflowingAddUnsigned, type Unsigned;
    /// Computes `self + rhs`, returning the wrapped sum and whether overflow
    /// occurred.
    fn overflowing_add_unsigned -> (Self::Output, bool);
}

declare_mixed_trait! {
    /// Saturating addition of an unsigned value to a signed value.
    trait SaturatingAddUnsigned, type Unsigned;
    /// Computes `self + rhs`, saturating at the numeric bounds.
    fn saturating_add_unsigned -> Self::Output;
}

declare_mixed_trait! {
    /// Strict addition of an unsigned value to a signed value.
    trait StrictAddUnsigned, type Unsigned;
    /// Computes `self + rhs`, panicking on overflow even in release builds.
    fn strict_add_unsigned -> Self::Output;
}

declare_mixed_trait! {
    /// Checked subtraction of an unsigned value from a signed value.
    trait CheckedSubUnsigned, type Unsigned;
    /// Computes `self - rhs`, returning `None` if overflow occurred.
    ///
    /// ```
    /// use const_num_traits::CheckedSubUnsigned;
    ///
    /// assert_eq!(CheckedSubUnsigned::checked_sub_unsigned(1i8, 2), Some(-1));
    /// assert_eq!(CheckedSubUnsigned::checked_sub_unsigned(i8::MIN, 1), None);
    /// ```
    fn checked_sub_unsigned -> Option<Self::Output>;
}

declare_mixed_trait! {
    /// Wrapping (modular) subtraction of an unsigned value from a signed value.
    trait WrappingSubUnsigned, type Unsigned;
    /// Computes `self - rhs`, wrapping around at the boundary of the type.
    fn wrapping_sub_unsigned -> Self::Output;
}

declare_mixed_trait! {
    /// Overflowing subtraction of an unsigned value from a signed value.
    trait OverflowingSubUnsigned, type Unsigned;
    /// Computes `self - rhs`, returning the wrapped difference and whether
    /// overflow occurred.
    fn overflowing_sub_unsigned -> (Self::Output, bool);
}

declare_mixed_trait! {
    /// Saturating subtraction of an unsigned value from a signed value.
    trait SaturatingSubUnsigned, type Unsigned;
    /// Computes `self - rhs`, saturating at the numeric bounds.
    fn saturating_sub_unsigned -> Self::Output;
}

declare_mixed_trait! {
    /// Strict subtraction of an unsigned value from a signed value.
    trait StrictSubUnsigned, type Unsigned;
    /// Computes `self - rhs`, panicking on overflow even in release builds.
    fn strict_sub_unsigned -> Self::Output;
}

// ---- impls ----

macro_rules! mixed_signed_impl {
    ($($t:ty => $s:ty;)*) => {$(
        c0nst::c0nst! {
        c0nst impl CheckedAddSigned for $t {
            type Signed = $s;
            type Output = $t;

            #[inline]
            fn checked_add_signed(self, rhs: $s) -> Option<$t> {
                <$t>::checked_add_signed(self, rhs)
            }
        }
        }

        c0nst::c0nst! {
        c0nst impl WrappingAddSigned for $t {
            type Signed = $s;
            type Output = $t;

            #[inline]
            fn wrapping_add_signed(self, rhs: $s) -> $t {
                <$t>::wrapping_add_signed(self, rhs)
            }
        }
        }

        c0nst::c0nst! {
        c0nst impl OverflowingAddSigned for $t {
            type Signed = $s;
            type Output = $t;

            #[inline]
            fn overflowing_add_signed(self, rhs: $s) -> ($t, bool) {
                <$t>::overflowing_add_signed(self, rhs)
            }
        }
        }

        c0nst::c0nst! {
        c0nst impl SaturatingAddSigned for $t {
            type Signed = $s;
            type Output = $t;

            #[inline]
            fn saturating_add_signed(self, rhs: $s) -> $t {
                <$t>::saturating_add_signed(self, rhs)
            }
        }
        }

        c0nst::c0nst! {
        c0nst impl StrictAddSigned for $t {
            type Signed = $s;
            type Output = $t;

            // stable since 1.91, newer than the MSRV
            #[inline]
            #[track_caller]
            fn strict_add_signed(self, rhs: $s) -> $t {
                let (val, overflowed) = <$t>::overflowing_add_signed(self, rhs);
                if overflowed {
                    panic!("attempt to add with overflow")
                }
                val
            }
        }
        }

        // the whole sub_signed family is newer than the MSRV (1.90/1.91);
        // same algorithms as core
        c0nst::c0nst! {
        c0nst impl OverflowingSubSigned for $t {
            type Signed = $s;
            type Output = $t;

            #[inline]
            fn overflowing_sub_signed(self, rhs: $s) -> ($t, bool) {
                // subtracting the bit pattern is correct modulo 2^BITS; the
                // borrow flag just has to flip when rhs was negative
                let (res, overflow) = <$t>::overflowing_sub(self, rhs as $t);
                (res, overflow != (rhs < 0))
            }
        }
        }

        c0nst::c0nst! {
        c0nst impl CheckedSubSigned for $t {
            type Signed = $s;
            type Output = $t;

            #[inline]
            fn checked_sub_signed(self, rhs: $s) -> Option<$t> {
                let (val, overflowed) =
                    OverflowingSubSigned::overflowing_sub_signed(self, rhs);
                if overflowed {
                    None
                } else {
                    Some(val)
                }
            }
        }
        }

        c0nst::c0nst! {
        c0nst impl WrappingSubSigned for $t {
            type Signed = $s;
            type Output = $t;

            #[inline]
            fn wrapping_sub_signed(self, rhs: $s) -> $t {
                <$t>::wrapping_sub(self, rhs as $t)
            }
        }
        }

        c0nst::c0nst! {
        c0nst impl SaturatingSubSigned for $t {
            type Signed = $s;
            type Output = $t;

            #[inline]
            fn saturating_sub_signed(self, rhs: $s) -> $t {
                let (val, overflowed) =
                    OverflowingSubSigned::overflowing_sub_signed(self, rhs);
                if !overflowed {
                    val
                } else if rhs < 0 {
                    <$t>::MAX
                } else {
                    0
                }
            }
        }
        }

        c0nst::c0nst! {
        c0nst impl StrictSubSigned for $t {
            type Signed = $s;
            type Output = $t;

            #[inline]
            #[track_caller]
            fn strict_sub_signed(self, rhs: $s) -> $t {
                let (val, overflowed) =
                    OverflowingSubSigned::overflowing_sub_signed(self, rhs);
                if overflowed {
                    panic!("attempt to subtract with overflow")
                }
                val
            }
        }
        }
    )*};
}

mixed_signed_impl! {
    u8 => i8;
    u16 => i16;
    u32 => i32;
    u64 => i64;
    usize => isize;
    u128 => i128;
}

macro_rules! mixed_unsigned_impl {
    ($($t:ty => $u:ty;)*) => {$(
        c0nst::c0nst! {
        c0nst impl CheckedAddUnsigned for $t {
            type Unsigned = $u;
            type Output = $t;

            #[inline]
            fn checked_add_unsigned(self, rhs: $u) -> Option<$t> {
                <$t>::checked_add_unsigned(self, rhs)
            }
        }
        }

        c0nst::c0nst! {
        c0nst impl WrappingAddUnsigned for $t {
            type Unsigned = $u;
            type Output = $t;

            #[inline]
            fn wrapping_add_unsigned(self, rhs: $u) -> $t {
                <$t>::wrapping_add_unsigned(self, rhs)
            }
        }
        }

        c0nst::c0nst! {
        c0nst impl OverflowingAddUnsigned for $t {
            type Unsigned = $u;
            type Output = $t;

            #[inline]
            fn overflowing_add_unsigned(self, rhs: $u) -> ($t, bool) {
                <$t>::overflowing_add_unsigned(self, rhs)
            }
        }
        }

        c0nst::c0nst! {
        c0nst impl SaturatingAddUnsigned for $t {
            type Unsigned = $u;
            type Output = $t;

            #[inline]
            fn saturating_add_unsigned(self, rhs: $u) -> $t {
                <$t>::saturating_add_unsigned(self, rhs)
            }
        }
        }

        c0nst::c0nst! {
        c0nst impl StrictAddUnsigned for $t {
            type Unsigned = $u;
            type Output = $t;

            #[inline]
            #[track_caller]
            fn strict_add_unsigned(self, rhs: $u) -> $t {
                let (val, overflowed) = <$t>::overflowing_add_unsigned(self, rhs);
                if overflowed {
                    panic!("attempt to add with overflow")
                }
                val
            }
        }
        }

        c0nst::c0nst! {
        c0nst impl CheckedSubUnsigned for $t {
            type Unsigned = $u;
            type Output = $t;

            #[inline]
            fn checked_sub_unsigned(self, rhs: $u) -> Option<$t> {
                <$t>::checked_sub_unsigned(self, rhs)
            }
        }
        }

        c0nst::c0nst! {
        c0nst impl WrappingSubUnsigned for $t {
            type Unsigned = $u;
            type Output = $t;

            #[inline]
            fn wrapping_sub_unsigned(self, rhs: $u) -> $t {
                <$t>::wrapping_sub_unsigned(self, rhs)
            }
        }
        }

        c0nst::c0nst! {
        c0nst impl OverflowingSubUnsigned for $t {
            type Unsigned = $u;
            type Output = $t;

            #[inline]
            fn overflowing_sub_unsigned(self, rhs: $u) -> ($t, bool) {
                <$t>::overflowing_sub_unsigned(self, rhs)
            }
        }
        }

        c0nst::c0nst! {
        c0nst impl SaturatingSubUnsigned for $t {
            type Unsigned = $u;
            type Output = $t;

            #[inline]
            fn saturating_sub_unsigned(self, rhs: $u) -> $t {
                <$t>::saturating_sub_unsigned(self, rhs)
            }
        }
        }

        c0nst::c0nst! {
        c0nst impl StrictSubUnsigned for $t {
            type Unsigned = $u;
            type Output = $t;

            #[inline]
            #[track_caller]
            fn strict_sub_unsigned(self, rhs: $u) -> $t {
                let (val, overflowed) = <$t>::overflowing_sub_unsigned(self, rhs);
                if overflowed {
                    panic!("attempt to subtract with overflow")
                }
                val
            }
        }
        }
    )*};
}

mixed_unsigned_impl! {
    i8 => u8;
    i16 => u16;
    i32 => u32;
    i64 => u64;
    isize => usize;
    i128 => u128;
}

c0nst::c0nst! {
/// Computes the signed difference of two unsigned values.
pub c0nst trait CheckedSignedDiff: Sized {
    /// The signed counterpart type (e.g. `i32` for `u32`).
    type Signed;

    /// Checked signed difference. Computes `self - rhs` as a signed value,
    /// returning `None` if the result doesn't fit in the signed type.
    ///
    /// ```
    /// use const_num_traits::CheckedSignedDiff;
    ///
    /// assert_eq!(CheckedSignedDiff::checked_signed_diff(10u8, 14), Some(-4));
    /// assert_eq!(CheckedSignedDiff::checked_signed_diff(u8::MAX, 0), None);
    /// ```
    fn checked_signed_diff(self, rhs: Self) -> Option<Self::Signed>;
}
}

macro_rules! checked_signed_diff_impl {
    ($($t:ty => $s:ty;)*) => {$(
        c0nst::c0nst! {
        c0nst impl CheckedSignedDiff for $t {
            type Signed = $s;

            // stable since 1.91, newer than the MSRV; same algorithm as core
            #[inline]
            fn checked_signed_diff(self, rhs: Self) -> Option<$s> {
                let res = <$t>::wrapping_sub(self, rhs) as $s;
                let overflow = (self >= rhs) == (res < 0);
                if !overflow {
                    Some(res)
                } else {
                    None
                }
            }
        }
        }
    )*};
}

checked_signed_diff_impl! {
    u8 => i8;
    u16 => i16;
    u32 => i32;
    u64 => i64;
    usize => isize;
    u128 => i128;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_signed() {
        assert_eq!(CheckedAddSigned::checked_add_signed(1u8, -2), None);
        assert_eq!(CheckedAddSigned::checked_add_signed(254u8, 1), Some(255));
        assert_eq!(WrappingAddSigned::wrapping_add_signed(1u8, -2), 255);
        assert_eq!(
            OverflowingAddSigned::overflowing_add_signed(255u8, 1),
            (0, true)
        );
        assert_eq!(SaturatingAddSigned::saturating_add_signed(1u8, -2), 0);
        assert_eq!(
            SaturatingAddSigned::saturating_add_signed(250u8, 100),
            255
        );
        assert_eq!(StrictAddSigned::strict_add_signed(250u8, 5), 255);
    }

    #[test]
    fn sub_signed() {
        assert_eq!(CheckedSubSigned::checked_sub_signed(1u8, 2), None);
        assert_eq!(CheckedSubSigned::checked_sub_signed(1u8, -2), Some(3));
        assert_eq!(CheckedSubSigned::checked_sub_signed(u8::MAX, -1), None);
        assert_eq!(WrappingSubSigned::wrapping_sub_signed(0u8, 1), 255);
        assert_eq!(
            OverflowingSubSigned::overflowing_sub_signed(0u8, 1),
            (255, true)
        );
        assert_eq!(
            OverflowingSubSigned::overflowing_sub_signed(255u8, -1),
            (0, true)
        );
        assert_eq!(
            OverflowingSubSigned::overflowing_sub_signed(10u8, -5),
            (15, false)
        );
        assert_eq!(SaturatingSubSigned::saturating_sub_signed(0u8, 1), 0);
        assert_eq!(
            SaturatingSubSigned::saturating_sub_signed(255u8, -1),
            255
        );
        assert_eq!(StrictSubSigned::strict_sub_signed(10u8, -5), 15);
    }

    #[test]
    #[should_panic(expected = "attempt to subtract with overflow")]
    fn strict_sub_signed_panics() {
        let _ = StrictSubSigned::strict_sub_signed(0u8, 1);
    }

    #[test]
    fn add_sub_unsigned() {
        assert_eq!(
            CheckedAddUnsigned::checked_add_unsigned(i8::MAX, 1),
            None
        );
        assert_eq!(
            WrappingAddUnsigned::wrapping_add_unsigned(i8::MAX, 1),
            i8::MIN
        );
        assert_eq!(
            OverflowingAddUnsigned::overflowing_add_unsigned(-1i8, 2),
            (1, false)
        );
        assert_eq!(
            SaturatingAddUnsigned::saturating_add_unsigned(100i8, 100),
            i8::MAX
        );
        assert_eq!(StrictAddUnsigned::strict_add_unsigned(-1i8, 128), 127);
        assert_eq!(
            CheckedSubUnsigned::checked_sub_unsigned(i8::MIN, 1),
            None
        );
        assert_eq!(
            WrappingSubUnsigned::wrapping_sub_unsigned(i8::MIN, 1),
            i8::MAX
        );
        assert_eq!(
            OverflowingSubUnsigned::overflowing_sub_unsigned(1i8, 2),
            (-1, false)
        );
        assert_eq!(
            SaturatingSubUnsigned::saturating_sub_unsigned(-100i8, 100),
            i8::MIN
        );
        assert_eq!(
            StrictSubUnsigned::strict_sub_unsigned(0i8, 128),
            i8::MIN
        );
    }

    #[test]
    fn signed_diff() {
        assert_eq!(
            CheckedSignedDiff::checked_signed_diff(10u8, 14),
            Some(-4)
        );
        assert_eq!(CheckedSignedDiff::checked_signed_diff(14u8, 10), Some(4));
        assert_eq!(CheckedSignedDiff::checked_signed_diff(u8::MAX, 0), None);
        assert_eq!(CheckedSignedDiff::checked_signed_diff(0u8, u8::MAX), None);
        assert_eq!(
            CheckedSignedDiff::checked_signed_diff(0u8, 128),
            Some(i8::MIN)
        );
        assert_eq!(
            CheckedSignedDiff::checked_signed_diff(127u8, 0),
            Some(i8::MAX)
        );
    }
}
