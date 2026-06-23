//! Proof of the `PrimBits` capability cut: a type with **no
//! `PartialEq`, no `Ord`, no `Div`/`Rem` and no panicking checked ops** can
//! implement `PrimBits` and use all of its bit-level machinery, including
//! the `leading_ones`/`trailing_ones`/`reverse_bits` defaults.
//!
//! This models a constant-time integer (like fixed-bigint's
//! `FixedUInt<_, _, Ct>` personality), which deliberately does not expose
//! comparison or division. If this file compiles, the extraction holds; if
//! someone re-bundles a comparison- or division-needing method into
//! `PrimBits`, it breaks.

use const_num_traits::{ConstOne, ConstZero, One, PrimBits, Zero};
use core::ops::{Add, BitAnd, BitOr, BitXor, Mul, Not, Shl, Shr};

/// A deliberately capability-starved wrapper: wrapping arithmetic only,
/// bit ops, and NO comparison/division.
#[derive(Copy, Clone, Debug)]
struct CtWord(u32);

impl Add for CtWord {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        CtWord(self.0.wrapping_add(rhs.0))
    }
}

impl Mul for CtWord {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        CtWord(self.0.wrapping_mul(rhs.0))
    }
}

impl Not for CtWord {
    type Output = Self;
    fn not(self) -> Self {
        CtWord(!self.0)
    }
}

impl BitAnd for CtWord {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        CtWord(self.0 & rhs.0)
    }
}

impl BitOr for CtWord {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        CtWord(self.0 | rhs.0)
    }
}

impl BitXor for CtWord {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self {
        CtWord(self.0 ^ rhs.0)
    }
}

impl Shl<usize> for CtWord {
    type Output = Self;
    fn shl(self, n: usize) -> Self {
        CtWord(self.0 << n)
    }
}

impl Shr<usize> for CtWord {
    type Output = Self;
    fn shr(self, n: usize) -> Self {
        CtWord(self.0 >> n)
    }
}

impl Zero for CtWord {
    fn zero() -> Self {
        CtWord(0)
    }
    fn set_zero(&mut self) {
        self.0 = 0;
    }
    // branchless: a mask-style check, no PartialEq involved
    fn is_zero(&self) -> bool {
        (self.0 | self.0.wrapping_neg()) >> 31 == 0
    }
}

impl One for CtWord {
    fn one() -> Self {
        CtWord(1)
    }
    fn set_one(&mut self) {
        self.0 = 1;
    }
    fn is_one(&self) -> bool {
        ((self.0 ^ 1) | (self.0 ^ 1).wrapping_neg()) >> 31 == 0
    }
}

impl ConstZero for CtWord {
    const ZERO: Self = CtWord(0);
}

impl ConstOne for CtWord {
    const ONE: Self = CtWord(1);
}

impl PrimBits for CtWord {
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
        CtWord(self.0.rotate_left(n))
    }
    fn rotate_right(self, n: u32) -> Self {
        CtWord(self.0.rotate_right(n))
    }
    fn signed_shl(self, n: u32) -> Self {
        CtWord(((self.0 as i32) << n) as u32)
    }
    fn signed_shr(self, n: u32) -> Self {
        CtWord(((self.0 as i32) >> n) as u32)
    }
    fn unsigned_shl(self, n: u32) -> Self {
        CtWord(self.0 << n)
    }
    fn unsigned_shr(self, n: u32) -> Self {
        CtWord(self.0 >> n)
    }
    fn swap_bytes(self) -> Self {
        CtWord(self.0.swap_bytes())
    }
    fn from_be(x: Self) -> Self {
        CtWord(u32::from_be(x.0))
    }
    fn from_le(x: Self) -> Self {
        CtWord(u32::from_le(x.0))
    }
    fn to_be(self) -> Self {
        CtWord(self.0.to_be())
    }
    fn to_le(self) -> Self {
        CtWord(self.0.to_le())
    }
    // leading_ones, trailing_ones, reverse_bits: defaults
}

// generic code bound on PrimBits alone — the supertrait methods resolve
// without any further imports
fn parity_of_reversed<T: PrimBits>(x: T) -> u32 {
    x.reverse_bits().count_ones() & 1
}

#[test]
fn ct_word_implements_prim_bits() {
    let x = CtWord(0x12345678);
    assert_eq!(x.count_ones(), 13);
    assert_eq!(x.leading_zeros(), 3);
    // defaults work through the trait
    assert_eq!(CtWord(0xF000_0001).leading_ones(), 4);
    assert_eq!(CtWord(0xF000_0001).trailing_ones(), 1);
    assert_eq!(x.reverse_bits().0, 0x12345678u32.reverse_bits());
    assert_eq!(parity_of_reversed(x), 13 & 1);
    assert_eq!(parity_of_reversed(0x12345678u32), 13 & 1);
}
