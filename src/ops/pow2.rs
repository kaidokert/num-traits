//! Power-of-two operations on unsigned integers, mirroring
//! `is_power_of_two` / `next_power_of_two` / `checked_next_power_of_two`
//! (stable since Rust 1.0) and the still-unstable
//! `wrapping_next_power_of_two`.
//!
//! Split: the predicate (`is_power_of_two`, a branchless
//! `count_ones == 1` check) is separate from the constructors (the three
//! `next_power_of_two` flavors, which share the `one_less` shift trick).
//!
//! **CT tiers**: [`IsPowerOfTwo`] is Tier B (popcount plus a boolean);
//! [`NextPowerOfTwo`] is Tier C (data-dependent early-out and overflow
//! panic).

c0nst::c0nst! {
/// Power-of-two predicate, for unsigned integers.
pub c0nst trait IsPowerOfTwo: Sized {
    /// Returns `true` if and only if `self == 2^k` for some integer `k`.
    /// Zero is not a power of two.
    ///
    /// ```
    /// use const_num_traits::IsPowerOfTwo;
    ///
    /// assert!(IsPowerOfTwo::is_power_of_two(16u8));
    /// assert!(!IsPowerOfTwo::is_power_of_two(10u8));
    /// assert!(!IsPowerOfTwo::is_power_of_two(0u8));
    /// ```
    fn is_power_of_two(self) -> bool;
}
}

c0nst::c0nst! {
/// Rounding up to a power of two, for unsigned integers.
pub c0nst trait NextPowerOfTwo: Sized {
    /// Returns the smallest power of two greater than or equal to `self`.
    ///
    /// # Panics
    ///
    /// With overflow checks enabled, panics if the next power of two doesn't
    /// fit in the type; without them, the result is wrapped (and wrong),
    /// matching the inherent method.
    type Output;
    fn next_power_of_two(self) -> Self::Output;

    /// Returns the smallest power of two greater than or equal to `self`,
    /// or `None` if it doesn't fit in the type.
    ///
    /// ```
    /// use const_num_traits::NextPowerOfTwo;
    ///
    /// assert_eq!(NextPowerOfTwo::checked_next_power_of_two(200u8), None);
    /// assert_eq!(NextPowerOfTwo::checked_next_power_of_two(3u8), Some(4));
    /// ```
    fn checked_next_power_of_two(self) -> Option<Self::Output>;

    /// Returns the smallest power of two greater than or equal to `self`,
    /// or 0 if it doesn't fit in the type (i.e. the result wraps to zero).
    ///
    /// This mirrors the still-unstable `wrapping_next_power_of_two` inherent
    /// method.
    fn wrapping_next_power_of_two(self) -> Self::Output;
}
}

macro_rules! power_of_two_impl {
    ($($t:ty)*) => {$(
        c0nst::c0nst! {
        c0nst impl IsPowerOfTwo for $t {
            #[inline]
            fn is_power_of_two(self) -> bool {
                <$t>::is_power_of_two(self)
            }
        }
        }

        c0nst::c0nst! {
        c0nst impl NextPowerOfTwo for $t {
            type Output = $t;
            #[inline]
            fn next_power_of_two(self) -> Self {
                <$t>::next_power_of_two(self)
            }

            #[inline]
            fn checked_next_power_of_two(self) -> Option<Self> {
                <$t>::checked_next_power_of_two(self)
            }

            // unstable in std; same one-less-than trick as core
            #[inline]
            fn wrapping_next_power_of_two(self) -> Self {
                if self <= 1 {
                    return 1;
                }
                let one_less = <$t>::MAX >> <$t>::leading_zeros(self - 1);
                one_less.wrapping_add(1)
            }
        }
        }
    )*};
}

power_of_two_impl!(usize u8 u16 u32 u64 u128);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_of_two() {
        assert!(IsPowerOfTwo::is_power_of_two(1u8));
        assert!(IsPowerOfTwo::is_power_of_two(128u8));
        assert!(!IsPowerOfTwo::is_power_of_two(0u8));
        assert_eq!(NextPowerOfTwo::next_power_of_two(0u8), 1);
        assert_eq!(NextPowerOfTwo::next_power_of_two(100u8), 128);
        assert_eq!(NextPowerOfTwo::checked_next_power_of_two(128u8), Some(128));
        assert_eq!(NextPowerOfTwo::checked_next_power_of_two(129u8), None);
        assert_eq!(NextPowerOfTwo::wrapping_next_power_of_two(129u8), 0);
        assert_eq!(NextPowerOfTwo::wrapping_next_power_of_two(0u8), 1);
        assert_eq!(NextPowerOfTwo::wrapping_next_power_of_two(65u8), 128);
    }
}
