//! Division-rounding and related operations: `div_ceil`, `div_floor`,
//! exact division, multiple-of computations and overflow-free `midpoint`,
//! mirroring the inherent methods on the primitive integer types.
//!
//! Stability in std (as of nightly 2026): `div_ceil` / `next_multiple_of`
//! (unsigned) are stable since 1.73, `is_multiple_of` since 1.87, `midpoint`
//! since 1.85/1.87; the signed `div_ceil` / `div_floor` /
//! `next_multiple_of` and `div_exact` / `checked_div_exact` are still
//! nightly-only. Anything newer than the crate's MSRV is hand-rolled here
//! with the same algorithm as core.
//!
//! **CT tiers**: [`Midpoint`] is Tier A (branchless); everything else here
//! is Tier C (division-based).

use core::ops::{Add, Div, Rem};

c0nst::c0nst! {
/// Performs division, rounding the quotient towards positive infinity.
pub c0nst trait DivCeil: Sized + [c0nst] Div<Self> {
    /// Calculates the quotient of `self` and `rhs`, rounding the result
    /// towards positive infinity.
    ///
    /// # Panics
    ///
    /// Panics if `rhs` is zero, or — with overflow checks enabled — on
    /// `MIN / -1` for signed types.
    ///
    /// ```
    /// use const_num_traits::DivCeil;
    ///
    /// assert_eq!(DivCeil::div_ceil(7u8, 2), 4);
    /// assert_eq!(DivCeil::div_ceil(7i8, -2), -3);
    /// assert_eq!(DivCeil::div_ceil(-7i8, 2), -3);
    /// ```
    fn div_ceil(self, rhs: Self) -> <Self as Div<Self>>::Output;
}
}

macro_rules! div_ceil_impl {
    (unsigned $($t:ty)*) => {$(
        c0nst::c0nst! {
        c0nst impl DivCeil for $t {
            #[inline]
            fn div_ceil(self, rhs: Self) -> Self {
                <$t>::div_ceil(self, rhs)
            }
        }
        }
    )*};
    // signed div_ceil is still unstable in std; same branchless algorithm
    (signed $($t:ty)*) => {$(
        c0nst::c0nst! {
        c0nst impl DivCeil for $t {
            #[inline]
            fn div_ceil(self, rhs: Self) -> Self {
                let d = self / rhs;
                let r = self % rhs;
                // `(self ^ rhs) >> (BITS - 1)` is -1 when the signs differ,
                // 0 when they match; +1 only when rounding up is needed.
                let correction = 1 + ((self ^ rhs) >> (<$t>::BITS - 1));
                if r != 0 {
                    d + correction
                } else {
                    d
                }
            }
        }
        }
    )*};
}

div_ceil_impl!(unsigned usize u8 u16 u32 u64 u128);
div_ceil_impl!(signed isize i8 i16 i32 i64 i128);

c0nst::c0nst! {
/// Performs division, rounding the quotient towards negative infinity.
pub c0nst trait DivFloor: Sized + [c0nst] Div<Self> {
    /// Calculates the quotient of `self` and `rhs`, rounding the result
    /// towards negative infinity. For unsigned types this is the normal
    /// integer division.
    ///
    /// # Panics
    ///
    /// Panics if `rhs` is zero, or — with overflow checks enabled — on
    /// `MIN / -1` for signed types.
    ///
    /// ```
    /// use const_num_traits::DivFloor;
    ///
    /// assert_eq!(DivFloor::div_floor(7u8, 2), 3);
    /// assert_eq!(DivFloor::div_floor(7i8, -2), -4);
    /// assert_eq!(DivFloor::div_floor(-7i8, 2), -4);
    /// ```
    fn div_floor(self, rhs: Self) -> <Self as Div<Self>>::Output;
}
}

macro_rules! div_floor_impl {
    // unsigned div_floor is plain division (and still unstable in std)
    (unsigned $($t:ty)*) => {$(
        c0nst::c0nst! {
        c0nst impl DivFloor for $t {
            #[inline]
            fn div_floor(self, rhs: Self) -> Self {
                self / rhs
            }
        }
        }
    )*};
    (signed $($t:ty)*) => {$(
        c0nst::c0nst! {
        c0nst impl DivFloor for $t {
            #[inline]
            fn div_floor(self, rhs: Self) -> Self {
                let d = self / rhs;
                let r = self % rhs;
                // all-ones (i.e. -1) iff the signs differ
                let correction = (self ^ rhs) >> (<$t>::BITS - 1);
                if r != 0 {
                    d + correction
                } else {
                    d
                }
            }
        }
        }
    )*};
}

div_floor_impl!(unsigned usize u8 u16 u32 u64 u128);
div_floor_impl!(signed isize i8 i16 i32 i64 i128);

c0nst::c0nst! {
/// Performs division without remainder.
pub c0nst trait DivExact: Sized + [c0nst] Div<Self> + [c0nst] Rem<Self> {
    /// Integer division without remainder. Computes `self / rhs`, returning
    /// `None` if `self % rhs != 0`.
    ///
    /// # Panics
    ///
    /// Panics if `rhs` is zero, or — with overflow checks enabled — on
    /// `MIN / -1` for signed types.
    ///
    /// ```
    /// use const_num_traits::DivExact;
    ///
    /// assert_eq!(DivExact::div_exact(64u8, 2), Some(32));
    /// assert_eq!(DivExact::div_exact(65u8, 2), None);
    /// ```
    fn div_exact(self, rhs: Self) -> Option<<Self as Div<Self>>::Output>;

    /// Checked integer division without remainder. Computes `self / rhs`,
    /// returning `None` if `rhs == 0`, `self % rhs != 0`, or the division
    /// overflowed (`MIN / -1` on a signed type).
    fn checked_div_exact(self, rhs: Self) -> Option<<Self as Div<Self>>::Output>;
}
}

macro_rules! div_exact_impl {
    ($($t:ty)*) => {$(
        c0nst::c0nst! {
        c0nst impl DivExact for $t {
            #[inline]
            fn div_exact(self, rhs: Self) -> Option<Self> {
                if self % rhs != 0 {
                    None
                } else {
                    Some(self / rhs)
                }
            }

            #[inline]
            fn checked_div_exact(self, rhs: Self) -> Option<Self> {
                // checked_rem rejects rhs == 0 and the MIN % -1 overflow,
                // checked_div rejects MIN / -1; together they cover all the
                // None cases without panicking.
                match <$t>::checked_rem(self, rhs) {
                    None => None,
                    Some(r) => {
                        if r != 0 {
                            None
                        } else {
                            <$t>::checked_div(self, rhs)
                        }
                    }
                }
            }
        }
        }
    )*};
}

div_exact_impl!(usize u8 u16 u32 u64 u128);
div_exact_impl!(isize i8 i16 i32 i64 i128);

c0nst::c0nst! {
/// Checks divisibility, mirroring `is_multiple_of` on the unsigned primitives.
pub c0nst trait MultipleOf: Sized + [c0nst] Rem<Self> {
    /// Returns `true` if `self` is an integer multiple of `rhs`, and `false`
    /// otherwise.
    ///
    /// This function is equivalent to `self % rhs == 0`, except that it
    /// will not panic for `rhs == 0`: instead, `0.is_multiple_of(0) == true`
    /// and `x.is_multiple_of(0) == false` for any nonzero `x`.
    ///
    /// Like std, this is only provided for unsigned types.
    ///
    /// ```
    /// use const_num_traits::MultipleOf;
    ///
    /// assert!(MultipleOf::is_multiple_of(6u8, 2));
    /// assert!(!MultipleOf::is_multiple_of(5u8, 2));
    /// assert!(MultipleOf::is_multiple_of(0u8, 0));
    /// assert!(!MultipleOf::is_multiple_of(5u8, 0));
    /// ```
    fn is_multiple_of(self, rhs: Self) -> bool;
}
}

macro_rules! multiple_of_impl {
    ($($t:ty)*) => {$(
        c0nst::c0nst! {
        c0nst impl MultipleOf for $t {
            #[inline]
            fn is_multiple_of(self, rhs: Self) -> bool {
                match rhs {
                    0 => self == 0,
                    _ => self % rhs == 0,
                }
            }
        }
        }
    )*};
}

multiple_of_impl!(usize u8 u16 u32 u64 u128);

c0nst::c0nst! {
/// Rounds up to the nearest multiple of a value.
pub c0nst trait NextMultipleOf: Sized + [c0nst] Add<Self> + [c0nst] Rem<Self> {
    /// Calculates the smallest value greater than or equal to `self` that is
    /// a multiple of `rhs` (for negative `rhs` on signed types: the closest
    /// to negative infinity).
    ///
    /// # Panics
    ///
    /// Panics if `rhs` is zero, or — with overflow checks enabled — if the
    /// result overflows.
    ///
    /// ```
    /// use const_num_traits::NextMultipleOf;
    ///
    /// assert_eq!(NextMultipleOf::next_multiple_of(16u8, 8), 16);
    /// assert_eq!(NextMultipleOf::next_multiple_of(23u8, 8), 24);
    /// assert_eq!(NextMultipleOf::next_multiple_of(-16i8, -5), -20);
    /// ```
    fn next_multiple_of(self, rhs: Self) -> <Self as Add<Self>>::Output;

    /// Calculates the smallest value greater than or equal to `self` that is
    /// a multiple of `rhs`, returning `None` if `rhs` is zero or the
    /// operation would result in overflow.
    fn checked_next_multiple_of(self, rhs: Self) -> Option<<Self as Add<Self>>::Output>;
}
}

macro_rules! next_multiple_of_impl {
    (unsigned $($t:ty)*) => {$(
        c0nst::c0nst! {
        c0nst impl NextMultipleOf for $t {
            #[inline]
            fn next_multiple_of(self, rhs: Self) -> Self {
                <$t>::next_multiple_of(self, rhs)
            }

            #[inline]
            fn checked_next_multiple_of(self, rhs: Self) -> Option<Self> {
                <$t>::checked_next_multiple_of(self, rhs)
            }
        }
        }
    )*};
    // signed next_multiple_of is still unstable in std; same algorithm
    (signed $($t:ty)*) => {$(
        c0nst::c0nst! {
        c0nst impl NextMultipleOf for $t {
            #[inline]
            fn next_multiple_of(self, rhs: Self) -> Self {
                // rhs == -1 would otherwise fail computing `MIN % -1`
                if rhs == -1 {
                    return self;
                }
                let r = self % rhs;
                let m = if (r > 0 && rhs < 0) || (r < 0 && rhs > 0) {
                    r + rhs
                } else {
                    r
                };
                if m == 0 {
                    self
                } else {
                    self + (rhs - m)
                }
            }

            #[inline]
            fn checked_next_multiple_of(self, rhs: Self) -> Option<Self> {
                if rhs == -1 {
                    return Some(self);
                }
                let r = match <$t>::checked_rem(self, rhs) {
                    Some(r) => r,
                    None => return None,
                };
                let m = if (r > 0 && rhs < 0) || (r < 0 && rhs > 0) {
                    // r + rhs cannot overflow: opposite signs
                    r + rhs
                } else {
                    r
                };
                if m == 0 {
                    Some(self)
                } else {
                    // rhs - m cannot overflow: m has the same sign as rhs
                    <$t>::checked_add(self, rhs - m)
                }
            }
        }
        }
    )*};
}

next_multiple_of_impl!(unsigned usize u8 u16 u32 u64 u128);
next_multiple_of_impl!(signed isize i8 i16 i32 i64 i128);

c0nst::c0nst! {
/// Computes the midpoint (average) of two values without overflowing.
pub c0nst trait Midpoint: Sized {
    /// Calculates the midpoint `(self + rhs) / 2` as if it were performed in
    /// a sufficiently-large type, so it never overflows. The result is
    /// rounded towards zero.
    ///
    /// ```
    /// use const_num_traits::Midpoint;
    ///
    /// assert_eq!(Midpoint::midpoint(u8::MAX, u8::MAX), u8::MAX);
    /// assert_eq!(Midpoint::midpoint(0u8, 7), 3);
    /// assert_eq!(Midpoint::midpoint(-7i8, 0), -3);
    /// ```
    type Output;
    fn midpoint(self, rhs: Self) -> Self::Output;
}
}

macro_rules! midpoint_impl {
    (unsigned $($t:ty)*) => {$(
        c0nst::c0nst! {
        c0nst impl Midpoint for $t {
            type Output = $t;
            #[inline]
            fn midpoint(self, rhs: Self) -> Self {
                <$t>::midpoint(self, rhs)
            }
        }
        }
    )*};
    // signed midpoint is stable since 1.87, newer than the MSRV; same
    // branchless Hacker's Delight algorithm as core
    (signed $($t:ty)*) => {$(
        c0nst::c0nst! {
        c0nst impl Midpoint for $t {
            type Output = $t;
            #[inline]
            fn midpoint(self, rhs: Self) -> Self {
                let t = ((self ^ rhs) >> 1) + (self & rhs);
                // The floor average of two integers whose sum is an odd
                // negative number is one below their truncated average;
                // bump it back towards zero.
                t + (if t < 0 { 1 } else { 0 } & (self ^ rhs))
            }
        }
        }
    )*};
}

midpoint_impl!(unsigned usize u8 u16 u32 u64 u128);
midpoint_impl!(signed isize i8 i16 i32 i64 i128);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn div_ceil_floor() {
        // cross-check the signed hand-rolled bodies against all small values
        for a in -50i32..=50 {
            for b in -50i32..=50 {
                if b == 0 {
                    continue;
                }
                let ceil = DivCeil::div_ceil(a, b);
                let floor = DivFloor::div_floor(a, b);
                let exact = (a as f64) / (b as f64);
                assert_eq!(ceil, exact.ceil() as i32, "{a} div_ceil {b}");
                assert_eq!(floor, exact.floor() as i32, "{a} div_floor {b}");
            }
        }
        assert_eq!(DivCeil::div_ceil(7u8, 2), 4);
        assert_eq!(DivFloor::div_floor(7u8, 2), 3);
    }

    #[test]
    fn div_exact() {
        assert_eq!(DivExact::div_exact(64u8, 4), Some(16));
        assert_eq!(DivExact::div_exact(66u8, 4), None);
        assert_eq!(DivExact::checked_div_exact(64u8, 0), None);
        assert_eq!(DivExact::checked_div_exact(i8::MIN, -1), None);
        assert_eq!(DivExact::checked_div_exact(-64i8, -4), Some(16));
        assert_eq!(DivExact::checked_div_exact(-65i8, -4), None);
    }

    #[test]
    fn multiples() {
        assert!(MultipleOf::is_multiple_of(12u32, 4));
        assert!(!MultipleOf::is_multiple_of(12u32, 5));
        assert!(MultipleOf::is_multiple_of(0u32, 0));
        assert!(!MultipleOf::is_multiple_of(12u32, 0));

        assert_eq!(NextMultipleOf::next_multiple_of(23u8, 8), 24);
        assert_eq!(NextMultipleOf::checked_next_multiple_of(250u8, 8), None);
        assert_eq!(NextMultipleOf::checked_next_multiple_of(23u8, 0), None);
        // match std's signed semantics
        assert_eq!(NextMultipleOf::next_multiple_of(16i8, 8), 16);
        assert_eq!(NextMultipleOf::next_multiple_of(23i8, 8), 24);
        assert_eq!(NextMultipleOf::next_multiple_of(16i8, -8), 16);
        assert_eq!(NextMultipleOf::next_multiple_of(23i8, -8), 16);
        assert_eq!(NextMultipleOf::next_multiple_of(-16i8, 8), -16);
        assert_eq!(NextMultipleOf::next_multiple_of(-23i8, 8), -16);
        assert_eq!(NextMultipleOf::next_multiple_of(-16i8, -8), -16);
        assert_eq!(NextMultipleOf::next_multiple_of(-23i8, -8), -24);
        assert_eq!(NextMultipleOf::next_multiple_of(i8::MIN, -1), i8::MIN);
        assert_eq!(NextMultipleOf::checked_next_multiple_of(i8::MAX, 2), None);
    }

    #[test]
    fn midpoint() {
        assert_eq!(Midpoint::midpoint(u8::MAX, u8::MAX), u8::MAX);
        assert_eq!(Midpoint::midpoint(0u8, 1), 0);
        // signed: rounded towards zero, matching std's documented examples
        assert_eq!(Midpoint::midpoint(-1i8, 2), 0);
        assert_eq!(Midpoint::midpoint(-7i8, 0), -3);
        assert_eq!(Midpoint::midpoint(0i8, 7), 3);
        assert_eq!(Midpoint::midpoint(i8::MIN, i8::MAX), 0);
        assert_eq!(Midpoint::midpoint(i8::MIN, i8::MIN), i8::MIN);
    }
}
