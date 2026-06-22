//! TRIPWIRE: every carrier-generic typestate proof must be constructible from a
//! **non-primitive** type using only its capability-trait bounds — with **no
//! `unsafe` at the call site**.
//!
//! The mistake this guards against: shipping a typestate (`PowerOfTwo`,
//! `BitIndex`, …) whose only *safe* constructor is the per-primitive macro
//! impl, so a non-primitive carrier (a fixed big-integer, a constant-time word)
//! is forced to reach for `from_*_unchecked` — i.e. `unsafe` leaks downstream.
//! A proof that can't be built safely for the crate's own carrier types is
//! upstream-broken, not a downstream problem.
//!
//! If this file fails to compile, a generic safe constructor was removed or
//! re-narrowed to primitives. Mirrors `tests/prim_bits.rs`; `Carrier` is a
//! deliberately capability-starved newtype (bit ops only, NO `PartialOrd` /
//! `Div`) that nonetheless constructs every proof below.

use const_num_traits::{
    BitIndex, ConstOne, ConstZero, IsPowerOfTwo, Odd, One, Parity, PowerOfTwo, PrimBits, Zero,
};
use core::ops::{Add, BitAnd, BitOr, BitXor, Mul, Not, Shl, Shr};

/// A non-primitive carrier delegating to an inner `u32`: wrapping arithmetic +
/// bit ops only, NO comparison/division.
#[derive(Copy, Clone, Debug)]
struct Carrier(u32);

impl Add for Carrier {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Carrier(self.0.wrapping_add(rhs.0))
    }
}
impl Mul for Carrier {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Carrier(self.0.wrapping_mul(rhs.0))
    }
}
impl Not for Carrier {
    type Output = Self;
    fn not(self) -> Self {
        Carrier(!self.0)
    }
}
impl BitAnd for Carrier {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Carrier(self.0 & rhs.0)
    }
}
impl BitOr for Carrier {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Carrier(self.0 | rhs.0)
    }
}
impl BitXor for Carrier {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self {
        Carrier(self.0 ^ rhs.0)
    }
}
impl Shl<usize> for Carrier {
    type Output = Self;
    fn shl(self, n: usize) -> Self {
        Carrier(self.0 << n)
    }
}
impl Shr<usize> for Carrier {
    type Output = Self;
    fn shr(self, n: usize) -> Self {
        Carrier(self.0 >> n)
    }
}

impl Zero for Carrier {
    fn zero() -> Self {
        Carrier(0)
    }
    fn set_zero(&mut self) {
        self.0 = 0;
    }
    fn is_zero(&self) -> bool {
        (self.0 | self.0.wrapping_neg()) >> 31 == 0
    }
}
impl One for Carrier {
    fn one() -> Self {
        Carrier(1)
    }
    fn set_one(&mut self) {
        self.0 = 1;
    }
    fn is_one(&self) -> bool {
        ((self.0 ^ 1) | (self.0 ^ 1).wrapping_neg()) >> 31 == 0
    }
}
impl ConstZero for Carrier {
    const ZERO: Self = Carrier(0);
}
impl ConstOne for Carrier {
    const ONE: Self = Carrier(1);
}

impl PrimBits for Carrier {
    fn count_ones(self) -> u32 {
        self.0.count_ones()
    }
    fn count_zeros(self) -> u32 {
        self.0.count_zeros()
    }
    fn leading_zeros(self) -> u32 {
        self.0.leading_zeros()
    }
    fn trailing_zeros(self) -> u32 {
        self.0.trailing_zeros()
    }
    fn rotate_left(self, n: u32) -> Self {
        Carrier(self.0.rotate_left(n))
    }
    fn rotate_right(self, n: u32) -> Self {
        Carrier(self.0.rotate_right(n))
    }
    fn signed_shl(self, n: u32) -> Self {
        Carrier(((self.0 as i32) << n) as u32)
    }
    fn signed_shr(self, n: u32) -> Self {
        Carrier(((self.0 as i32) >> n) as u32)
    }
    fn unsigned_shl(self, n: u32) -> Self {
        Carrier(self.0 << n)
    }
    fn unsigned_shr(self, n: u32) -> Self {
        Carrier(self.0 >> n)
    }
    fn swap_bytes(self) -> Self {
        Carrier(self.0.swap_bytes())
    }
    fn from_be(x: Self) -> Self {
        Carrier(u32::from_be(x.0))
    }
    fn from_le(x: Self) -> Self {
        Carrier(u32::from_le(x.0))
    }
    fn to_be(self) -> Self {
        Carrier(self.0.to_be())
    }
    fn to_le(self) -> Self {
        Carrier(self.0.to_le())
    }
}

impl IsPowerOfTwo for Carrier {
    fn is_power_of_two(self) -> bool {
        self.0.is_power_of_two()
    }
}

impl Parity for Carrier {
    fn is_odd(self) -> bool {
        self.0 & 1 == 1
    }
    fn is_even(self) -> bool {
        self.0 & 1 == 0
    }
}

// ── the tripwire ──────────────────────────────────────────────────────────
//
// Each helper is generic over the *capability bounds alone* — it proves the
// constructor needs only the traits, never the concrete primitive set. They are
// instantiated below at `Carrier`, a type with no `core` integer in its public
// API. If any of these stops resolving, a proof has regressed to primitive-only
// construction.

fn pow2_from_caps<T: IsPowerOfTwo + PrimBits>(v: T) -> Option<PowerOfTwo<T>> {
    PowerOfTwo::new_checked(v)
}
fn bitindex_from_caps<T: PrimBits>(i: u32) -> Option<BitIndex<T>> {
    BitIndex::<T>::new_checked(i)
}
fn odd_from_caps<T: Parity + Copy>(v: T) -> Option<Odd<T>> {
    Odd::new(v)
}

#[test]
fn typestates_construct_from_non_primitive_carrier() {
    // PowerOfTwo<Carrier>: exponent is recovered correctly for a 32-bit carrier.
    let p = pow2_from_caps(Carrier(16)).expect("16 is a power of two");
    assert_eq!(p.exp(), 4);
    assert_eq!(pow2_from_caps(Carrier(1)).unwrap().exp(), 0);
    assert_eq!(pow2_from_caps(Carrier(1 << 31)).unwrap().exp(), 31);
    assert!(pow2_from_caps(Carrier(17)).is_none());
    assert!(pow2_from_caps(Carrier(0)).is_none());

    // BitIndex<Carrier>: width is the carrier's 32 bits.
    assert_eq!(bitindex_from_caps::<Carrier>(31).unwrap().get(), 31);
    assert!(bitindex_from_caps::<Carrier>(32).is_none()); // == BITS

    // Odd<Carrier>: already generic over `Parity` (the pattern the above match).
    assert!(odd_from_caps(Carrier(7)).is_some());
    assert!(odd_from_caps(Carrier(8)).is_none());
}
