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
    BitIndex, BitPos8, BitPos16, ConstOne, ConstZero, HasBitPos, IsPowerOfTwo, Odd, One, Parity,
    PowerOfTwo, PrimBits, Zero,
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

impl HasBitPos for Carrier {
    type BitPos = BitPos8;
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

fn pow2_from_caps<T: IsPowerOfTwo + PrimBits + HasBitPos>(v: T) -> Option<PowerOfTwo<T>> {
    PowerOfTwo::new_checked(v)
}
fn bitindex_from_caps<T: PrimBits + HasBitPos>(i: u32) -> Option<BitIndex<T>> {
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

    // Odd<Carrier>: already generic over `Parity` (the pattern the others follow).
    assert!(odd_from_caps(Carrier(7)).is_some());
    assert!(odd_from_caps(Carrier(8)).is_none());
}

/// `PowerOfTwo<T>` and `BitIndex<T>` compare and format whatever `T` is: this
/// compiles only while those impls carry no bound on `T` beyond `HasBitPos`.
#[allow(dead_code)]
fn proofs_are_eq_and_debug_for_any_carrier<T: HasBitPos>() {
    fn has<X: Copy + Eq + core::fmt::Debug>() {}
    has::<PowerOfTwo<T>>();
    has::<BitIndex<T>>();
}

#[test]
fn proofs_compare_over_a_carrier_without_partial_eq() {
    assert_eq!(pow2_from_caps(Carrier(16)), pow2_from_caps(Carrier(16)));
    assert_ne!(pow2_from_caps(Carrier(16)), pow2_from_caps(Carrier(32)));
    assert_eq!(
        bitindex_from_caps::<Carrier>(5),
        bitindex_from_caps::<Carrier>(5)
    );
    assert_ne!(
        bitindex_from_caps::<Carrier>(5),
        bitindex_from_caps::<Carrier>(6)
    );
}

// ── a carrier wider than 128 bits ─────────────────────────────────────────────

/// A 256-bit carrier, limbs little-endian. Only what `PrimBits` and
/// `IsPowerOfTwo` need; the methods the proofs never call are `unreachable!`.
#[derive(Copy, Clone, Debug)]
struct Wide([u128; 2]);

impl Wide {
    fn bit(k: u32) -> Self {
        if k < 128 {
            Wide([1 << k, 0])
        } else {
            Wide([0, 1 << (k - 128)])
        }
    }
}

impl Not for Wide {
    type Output = Self;
    fn not(self) -> Self {
        Wide([!self.0[0], !self.0[1]])
    }
}
impl BitAnd for Wide {
    type Output = Self;
    fn bitand(self, r: Self) -> Self {
        Wide([self.0[0] & r.0[0], self.0[1] & r.0[1]])
    }
}
impl BitOr for Wide {
    type Output = Self;
    fn bitor(self, r: Self) -> Self {
        Wide([self.0[0] | r.0[0], self.0[1] | r.0[1]])
    }
}
impl BitXor for Wide {
    type Output = Self;
    fn bitxor(self, r: Self) -> Self {
        Wide([self.0[0] ^ r.0[0], self.0[1] ^ r.0[1]])
    }
}
impl Shl<usize> for Wide {
    type Output = Self;
    fn shl(self, _: usize) -> Self {
        unreachable!("not used by the proofs")
    }
}
impl Shr<usize> for Wide {
    type Output = Self;
    fn shr(self, _: usize) -> Self {
        unreachable!("not used by the proofs")
    }
}
impl ConstZero for Wide {
    const ZERO: Self = Wide([0, 0]);
}
impl ConstOne for Wide {
    const ONE: Self = Wide([1, 0]);
}

impl PrimBits for Wide {
    fn count_ones(self) -> u32 {
        self.0[0].count_ones() + self.0[1].count_ones()
    }
    fn count_zeros(self) -> u32 {
        self.0[0].count_zeros() + self.0[1].count_zeros()
    }
    fn leading_zeros(self) -> u32 {
        if self.0[1] != 0 {
            self.0[1].leading_zeros()
        } else {
            128 + self.0[0].leading_zeros()
        }
    }
    fn trailing_zeros(self) -> u32 {
        if self.0[0] != 0 {
            self.0[0].trailing_zeros()
        } else {
            128 + self.0[1].trailing_zeros()
        }
    }
    fn rotate_left(self, _: u32) -> Self {
        unreachable!()
    }
    fn rotate_right(self, _: u32) -> Self {
        unreachable!()
    }
    fn signed_shl(self, _: u32) -> Self {
        unreachable!()
    }
    fn signed_shr(self, _: u32) -> Self {
        unreachable!()
    }
    fn unsigned_shl(self, _: u32) -> Self {
        unreachable!()
    }
    fn unsigned_shr(self, _: u32) -> Self {
        unreachable!()
    }
    fn swap_bytes(self) -> Self {
        unreachable!()
    }
    fn from_be(_: Self) -> Self {
        unreachable!()
    }
    fn from_le(_: Self) -> Self {
        unreachable!()
    }
    fn to_be(self) -> Self {
        unreachable!()
    }
    fn to_le(self) -> Self {
        unreachable!()
    }
}

impl HasBitPos for Wide {
    type BitPos = BitPos16;
}

impl IsPowerOfTwo for Wide {
    fn is_power_of_two(self) -> bool {
        self.count_ones() == 1
    }
}

/// A 256-bit carrier with two-byte positions proves every power of two and
/// every index it has, and `Option` of either proof is still tag-free.
#[test]
fn wide_carrier_proves_every_position() {
    for k in 0..256 {
        assert_eq!(pow2_from_caps(Wide::bit(k)).map(|p| p.exp()), Some(k));
        assert_eq!(bitindex_from_caps::<Wide>(k).map(|i| i.get()), Some(k));
    }
    assert!(bitindex_from_caps::<Wide>(256).is_none()); // == BITS
    assert!(pow2_from_caps(Wide([3, 0])).is_none());
    assert!(pow2_from_caps(Wide::ZERO).is_none());
    assert_eq!(core::mem::size_of::<Option<PowerOfTwo<Wide>>>(), 2);
    assert_eq!(core::mem::size_of::<Option<BitIndex<Wide>>>(), 2);
}

// ── a carrier whose safe trait impls lie ─────────────────────────────────────

/// One byte of storage that claims to be 64 bits wide, and claims every value
/// is a power of two with its top bit set. `PrimBits` and `IsPowerOfTwo` are
/// safe traits, so the proofs must stay sound against this: their accessors
/// tell the optimiser `position < 8 * size_of::<T>()`, and the generic
/// constructors check that bound instead of trusting the claimed width.
#[derive(Copy, Clone, Debug)]
struct Liar(u8);

impl Not for Liar {
    type Output = Self;
    fn not(self) -> Self {
        Liar(!self.0)
    }
}
impl BitAnd for Liar {
    type Output = Self;
    fn bitand(self, r: Self) -> Self {
        Liar(self.0 & r.0)
    }
}
impl BitOr for Liar {
    type Output = Self;
    fn bitor(self, r: Self) -> Self {
        Liar(self.0 | r.0)
    }
}
impl BitXor for Liar {
    type Output = Self;
    fn bitxor(self, r: Self) -> Self {
        Liar(self.0 ^ r.0)
    }
}
impl Shl<usize> for Liar {
    type Output = Self;
    fn shl(self, _: usize) -> Self {
        self
    }
}
impl Shr<usize> for Liar {
    type Output = Self;
    fn shr(self, _: usize) -> Self {
        self
    }
}
impl ConstZero for Liar {
    const ZERO: Self = Liar(0);
}
impl ConstOne for Liar {
    const ONE: Self = Liar(1);
}
impl PrimBits for Liar {
    fn count_ones(self) -> u32 {
        1
    }
    fn count_zeros(self) -> u32 {
        64
    }
    fn leading_zeros(self) -> u32 {
        self.0 as u32 % 64
    }
    fn trailing_zeros(self) -> u32 {
        0
    }
    fn rotate_left(self, _: u32) -> Self {
        self
    }
    fn rotate_right(self, _: u32) -> Self {
        self
    }
    fn signed_shl(self, _: u32) -> Self {
        self
    }
    fn signed_shr(self, _: u32) -> Self {
        self
    }
    fn unsigned_shl(self, _: u32) -> Self {
        self
    }
    fn unsigned_shr(self, _: u32) -> Self {
        self
    }
    fn swap_bytes(self) -> Self {
        self
    }
    fn from_be(x: Self) -> Self {
        x
    }
    fn from_le(x: Self) -> Self {
        x
    }
    fn to_be(self) -> Self {
        self
    }
    fn to_le(self) -> Self {
        self
    }
}
impl HasBitPos for Liar {
    type BitPos = BitPos8;
}

impl IsPowerOfTwo for Liar {
    fn is_power_of_two(self) -> bool {
        true
    }
}

#[test]
fn lying_carrier_cannot_prove_past_its_storage() {
    // Claimed width 64, storage 8 bits: only 0..8 is provable.
    for k in 0..8 {
        assert_eq!(bitindex_from_caps::<Liar>(k).map(|i| i.get()), Some(k));
    }
    for k in 8..64 {
        assert!(bitindex_from_caps::<Liar>(k).is_none(), "index {k}");
    }
    // `leading_zeros` = value % 64, so the claimed exponent is 63 - value % 64.
    for value in 0..=u8::MAX {
        let exp = 63 - u32::from(value) % 64;
        let proof = pow2_from_caps(Liar(value));
        if exp < 8 {
            assert_eq!(proof.map(|p| p.exp()), Some(exp), "value {value}");
        } else {
            // `exp()` on such a proof would be undefined behaviour.
            assert!(proof.is_none(), "value {value}");
        }
    }
}
