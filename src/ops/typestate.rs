//! Opt-in typestate proofs (the `typestate` feature).
//!
//! A *typestate* is a zero-runtime-cost proof that a value satisfies a
//! structural property, constructed once with a checked constructor and then
//! consumed by operations that exploit the invariant to delete a branch, an
//! `Option`, or a division. A proof with no consuming op would be dead weight,
//! so every type here ships with at least one op that *spends* it.
//!
//! This module is **purely additive** and gated behind the `typestate` feature:
//! nothing here is visible unless you opt in.
//!
//! Currently: [`PowerOfTwo`] + [`PowerOfTwoOps`].

use core::marker::PhantomData;

/// Proof that a value is a power of two (`2^k`, `k ≥ 0`), for an unsigned
/// integer type `T`.
///
/// **Representation is the exponent `k`, not the value** — so the consuming
/// operations in [`PowerOfTwoOps`] are pure shifts and masks with nothing
/// recomputed per call, and a (future) big-integer backend carries a tiny
/// `u32` proof regardless of its width. The field is private, so the
/// representation is an implementation detail. Consequently `PowerOfTwo` does
/// **not** deref to `T`; use [`get`](Self::get) (which reconstructs `1 << k`)
/// or [`exp`](Self::exp).
///
/// `PowerOfTwo<T>` is always `Copy` (it stores only a `u32`), independent of
/// whether `T` is.
///
/// # Examples
///
/// ```
/// use const_num_traits::{PowerOfTwo, PowerOfTwoOps};
///
/// let p = PowerOfTwo::<u32>::new(16).unwrap();
/// assert_eq!(p.exp(), 4);
/// assert_eq!(p.get(), 16);
/// assert_eq!(100u32.div_pow2(p), 6); // 100 / 16
/// assert_eq!(100u32.rem_pow2(p), 4); // 100 % 16
/// assert!(PowerOfTwo::<u32>::new(6).is_none());
/// ```
pub struct PowerOfTwo<T> {
    // Invariant: `1 << exp` is a valid power of two of `T` (`exp < T::BITS`).
    exp: u32,
    _t: PhantomData<T>,
}

impl<T> Clone for PowerOfTwo<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for PowerOfTwo<T> {}

impl<T> PowerOfTwo<T> {
    /// Constructs the proof directly from an exponent, without checking.
    ///
    /// # Safety
    ///
    /// `exp` must be `< T::BITS` (so that `1 << exp` is a valid power of two of
    /// `T`). Passing a too-large exponent makes the consuming ops produce
    /// nonsense or overflow.
    #[inline]
    pub const unsafe fn from_exp_unchecked(exp: u32) -> Self {
        PowerOfTwo {
            exp,
            _t: PhantomData,
        }
    }

    /// The exponent `k` such that the proven value is `2^k`. Free (no recompute).
    #[inline]
    pub const fn exp(self) -> u32 {
        self.exp
    }
}

c0nst::c0nst! {
/// Operations that *consume* a [`PowerOfTwo`] proof to replace division and
/// remainder with a shift and a mask. These are **new** methods, not a
/// speed-up of the blanket `MultipleOf`/`NextMultipleOf` (those can't be
/// specialised on a power-of-two divisor on stable Rust).
///
/// Owned results carry an associated [`Output`](Self::Output) so non-`Copy`
/// types can implement the trait for their reference type.
pub c0nst trait PowerOfTwoOps: Sized {
    /// The owned result type (`Self` for the primitive impls).
    type Output;

    /// `self / p`, computed as `self >> p.exp()`.
    fn div_pow2(self, p: PowerOfTwo<Self>) -> Self::Output;

    /// `self % p`, computed as `self & ((1 << p.exp()) - 1)`.
    fn rem_pow2(self, p: PowerOfTwo<Self>) -> Self::Output;

    /// `self % p == 0`, computed as a mask test (no division).
    fn is_multiple_of_pow2(self, p: PowerOfTwo<Self>) -> bool;
}
}

macro_rules! pow2_typestate_impl {
    ($($t:ty),+) => {$(
        impl PowerOfTwo<$t> {
            /// Checked constructor: `Some` iff `value` is a power of two.
            ///
            /// `const fn` on stable and nightly (delegates to the inherent
            /// `is_power_of_two`/`ilog2`, both const since ≤ 1.67).
            #[inline]
            pub const fn new(value: $t) -> Option<Self> {
                if value.is_power_of_two() {
                    Some(PowerOfTwo { exp: value.ilog2(), _t: PhantomData })
                } else {
                    None
                }
            }

            /// Reconstructs the proven value, `1 << exp`.
            #[inline]
            pub const fn get(self) -> $t {
                (1 as $t) << self.exp
            }
        }

        c0nst::c0nst! {
        c0nst impl PowerOfTwoOps for $t {
            type Output = $t;

            #[inline]
            fn div_pow2(self, p: PowerOfTwo<$t>) -> $t {
                self >> p.exp
            }

            #[inline]
            fn rem_pow2(self, p: PowerOfTwo<$t>) -> $t {
                let mask = ((1 as $t) << p.exp) - 1;
                self & mask
            }

            #[inline]
            fn is_multiple_of_pow2(self, p: PowerOfTwo<$t>) -> bool {
                let mask = ((1 as $t) << p.exp) - 1;
                (self & mask) == 0
            }
        }
        }
    )+};
}

pow2_typestate_impl!(u8, u16, u32, u64, u128, usize);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construction_and_accessors() {
        assert!(PowerOfTwo::<u32>::new(0).is_none());
        assert!(PowerOfTwo::<u32>::new(3).is_none());
        assert!(PowerOfTwo::<u32>::new(6).is_none());

        let p = PowerOfTwo::<u32>::new(64).unwrap();
        assert_eq!(p.exp(), 6);
        assert_eq!(p.get(), 64);

        // edges: 2^0 = 1, and the largest power of two for the type
        assert_eq!(PowerOfTwo::<u8>::new(1).unwrap().exp(), 0);
        assert_eq!(PowerOfTwo::<u8>::new(128).unwrap().get(), 128);
    }

    #[test]
    fn consuming_ops() {
        let p = PowerOfTwo::<u32>::new(16).unwrap();
        assert_eq!(100u32.div_pow2(p), 100 / 16);
        assert_eq!(100u32.rem_pow2(p), 100 % 16);
        assert!(!100u32.is_multiple_of_pow2(p));
        assert!(96u32.is_multiple_of_pow2(p));

        // 2^0 = 1: div is identity, rem is always 0
        let one = PowerOfTwo::<u64>::new(1).unwrap();
        assert_eq!(12345u64.div_pow2(one), 12345);
        assert_eq!(12345u64.rem_pow2(one), 0);
        assert!(12345u64.is_multiple_of_pow2(one));
    }

    #[test]
    fn matches_naive_exhaustive_u8() {
        for k in 0..8u32 {
            let d = 1u8 << k;
            let p = PowerOfTwo::<u8>::new(d).unwrap();
            for x in 0..=u8::MAX {
                assert_eq!(x.div_pow2(p), x >> k);
                assert_eq!(x.rem_pow2(p), x & (d - 1));
                assert_eq!(x.is_multiple_of_pow2(p), x % d == 0);
            }
        }
    }

    #[test]
    fn const_constructor_on_stable() {
        const P: Option<PowerOfTwo<u32>> = PowerOfTwo::<u32>::new(256);
        const GOT: u32 = match P {
            Some(p) => p.get(),
            None => 0,
        };
        assert_eq!(GOT, 256);
    }
}
