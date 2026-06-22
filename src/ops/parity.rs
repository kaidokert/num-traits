//! Parity (odd/even) checks.
//!
//! std has no inherent `is_odd`/`is_even` (they live in `num-integer`'s
//! `Integer` bundle); this is the atom version. Unlike a blanket impl over
//! `BitAnd + One + PartialEq + Clone` (modmath's current fallback shape),
//! a per-type trait needs no `PartialEq`/`Clone` bounds — important both
//! for constant-time types that deliberately don't expose `PartialEq` and
//! for multi-limb bigints, which can answer from the lowest limb alone
//! instead of comparing whole values.
//!
//! Both methods live in one trait: they're each other's negation, one
//! capability. Note for constant-time callers: the returned `bool` *is* the
//! low bit of the operand — treat it as secret-derived (Tier B).
//!
//! **CT tier B (caller-leaky)**: the check is one AND, but the returned
//! `bool` is the operand's low bit. A masked counterpart, `CtParity`, is
//! available with the `ct` feature.

c0nst::c0nst! {
/// Checks whether a value is odd or even.
pub c0nst trait Parity {
    /// Returns `true` if `self` is odd.
    ///
    /// For signed types this is a property of the value's low bit, so it is
    /// correct for negative values too (`-3` is odd).
    ///
    /// ```
    /// use const_num_traits::Parity;
    ///
    /// assert!(Parity::is_odd(3u8));
    /// assert!(!Parity::is_odd(4u8));
    /// assert!(Parity::is_odd(-3i8));
    /// ```
    fn is_odd(self) -> bool;

    /// Returns `true` if `self` is even.
    ///
    /// ```
    /// use const_num_traits::Parity;
    ///
    /// assert!(Parity::is_even(4u8));
    /// assert!(!Parity::is_even(3u8));
    /// assert!(Parity::is_even(0u8));
    /// ```
    fn is_even(self) -> bool;
}
}

macro_rules! parity_impl {
    ($($t:ty)*) => {$(
        c0nst::c0nst! {
        c0nst impl Parity for $t {
            #[inline]
            fn is_odd(self) -> bool {
                self & 1 == 1
            }

            #[inline]
            fn is_even(self) -> bool {
                self & 1 == 0
            }
        }
        }
    )*};
}

parity_impl!(usize u8 u16 u32 u64 u128);
parity_impl!(isize i8 i16 i32 i64 i128);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parity() {
        assert!(Parity::is_odd(1u8));
        assert!(Parity::is_even(0u8));
        assert!(Parity::is_even(u128::MAX - 1));
        assert!(Parity::is_odd(u128::MAX));
        assert!(Parity::is_odd(-1i8));
        assert!(Parity::is_odd(i8::MIN.wrapping_sub(1))); // i8::MAX, odd
        assert!(Parity::is_even(i8::MIN));
        assert!(Parity::is_odd(-3i64));
        assert!(Parity::is_even(-4i64));
    }
}
