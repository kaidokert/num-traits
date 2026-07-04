//! Typestate proofs in use.
//!
//! Each typestate is a zero-cost newtype: you construct the proof once at a
//! boundary (a parse, a config value, a constant), then *spend* it with an op
//! that exploits the invariant to drop a branch, an `Option`, or a division.
//! Everything below is straight-line after the proof is built.
//!
//! Run with: `cargo run --example typestates`
//! (the constant-time section also needs `--features ct`).

use std::collections::BTreeMap;

use const_num_traits::{
    BitIndex, BitIndexOps, DivNonZero, Finite, HasNonZero, NonMin, NonNegative, Odd, Positive,
    PowerOfTwo, PowerOfTwoOps,
};

/// `PowerOfTwo`: reduce / divide / align by a `2^k` modulus as masks and
/// shifts, with no division and no divide-by-zero branch.
fn power_of_two() {
    let radix = 256u64;
    let p = PowerOfTwo::<u64>::new(radix).expect("256 is 2^8");

    // `x % radix` and `x / radix` without a division instruction
    assert_eq!(700u64.rem_pow2(p), 700 % 256); // & 0xFF
    assert_eq!(700u64.div_pow2(p), 700 / 256); // >> 8

    // align-up to the next multiple, branch-free; checked form for the edge
    assert_eq!(700u64.next_multiple_of_pow2(p), 768);
    assert_eq!(u64::MAX.checked_next_multiple_of_pow2(p), None);

    // built once from a runtime value, spent in a loop with no per-iter recheck
    let mut folded = 0u64;
    for limb in [10u64, 600, 70_000] {
        folded ^= limb.rem_pow2(p);
    }
    assert_eq!(folded, (10 ^ 600 ^ 70_000) & 0xFF);
}

/// `BitIndex`: shift by an amount proven `< BITS`, so the shift carries no
/// "is the amount too large?" overflow-check branch.
fn bit_index() {
    let pos = BitIndex::<u64>::new(40).expect("40 < 64");
    assert_eq!((1u64).shl_index(pos), 1u64 << 40);
    assert_eq!((1u64 << 63).shr_index(pos), (1u64 << 63) >> 40);

    // out-of-range index is rejected at construction, not at the shift
    assert!(BitIndex::<u64>::new(64).is_none());
}

/// `HasNonZero` + `DivNonZero`: division by a value proven non-zero — no
/// `Option` result, no divide-by-zero path.
fn nonzero_division() {
    let d = 17u32.into_nonzero().expect("17 != 0");
    assert_eq!(100u32.div_nonzero(d), 100 / 17);
    assert_eq!(100u32.rem_nonzero(d), 100 % 17);
    assert!(0u32.into_nonzero().is_none());
}

/// `NonNegative` / `Positive`: a value proven `>= 0` casts to unsigned with no
/// `TryFrom`/`Option`, and takes `isqrt` without the negative-input panic path.
fn signed_range() {
    let nn = NonNegative::<i32>::new(81).expect("81 >= 0");
    let u: u32 = nn.to_unsigned(); // infallible, bit-preserving
    assert_eq!(u, 81);
    assert_eq!(nn.isqrt(), 9); // total: input is known non-negative

    // a positive value narrows straight into a NonZero divisor
    let nz = Positive::<i32>::new(7).expect("7 > 0").into_nonzero();
    assert_eq!(nz.get(), 7);
}

/// `NonMin`: a signed value proven `!= MIN` has total `abs`/`neg` and total
/// signed division — the `MIN.abs()` and `MIN / -1` overflow cases are gone.
fn nonmin() {
    let a = NonMin::<i32>::new(-7).expect("-7 != MIN");
    assert_eq!(a.abs(), 7); // no overflow branch
    assert_eq!(a.neg(), 7);

    // signed division by NonZero: MIN/-1 is impossible by construction
    let d = core::num::NonZero::new(2).unwrap();
    assert_eq!(a.div_nonzero(d), -7 / 2);

    // Positive/NonNegative narrow into NonMin for free
    let q = Positive::<i32>::new(9).unwrap().into_nonmin();
    assert_eq!(q.abs(), 9);

    assert!(NonMin::<i32>::new(i32::MIN).is_none());
}

/// `Odd`: the precondition lives in the type. The proof is const-constructible
/// (on nightly with `--features nightly`), so a constant modulus is checked at
/// compile time rather than via a runtime `.unwrap()`.
fn odd_modulus() {
    // a parsed/odd modulus, proven once
    let m = Odd::<u64>::new(0xFFFF_FFFF_FFFF_FFC5).expect("a large odd prime");
    // `Odd ⇒ non-zero` (zero is even), so this one proof covers both the
    // "odd" and "non-zero" preconditions at once.
    assert_eq!(m.get() & 1, 1);

    assert!(Odd::<u64>::new(0).is_none()); // zero is even
    assert!(Odd::<u64>::new(10).is_none());
}

/// `Finite`: finite floats gain a total order, so they sort and key a
/// `BTreeMap` — neither of which bare `f32`/`f64` can do (NaN breaks `Ord`).
fn finite_floats() {
    let raw = [2.0f64, f64::NAN, 1.0, f64::INFINITY, 0.5];

    // keep only the finite values; each carries its proof
    let mut xs: Vec<Finite<f64>> = raw.iter().copied().filter_map(Finite::<f64>::new).collect();
    xs.sort(); // total Ord — no `partial_cmp().unwrap()`
    let sorted: Vec<f64> = xs.iter().map(|f| f.get()).collect();
    assert_eq!(sorted, [0.5, 1.0, 2.0]);

    // finite floats as map keys
    let mut table: BTreeMap<Finite<f64>, &str> = BTreeMap::new();
    table.insert(Finite::<f64>::new(1.0).unwrap(), "one");
    table.insert(Finite::<f64>::new(2.0).unwrap(), "two");
    assert_eq!(table.get(&Finite::<f64>::new(1.0).unwrap()), Some(&"one"));
}

/// Constant-time constructors (`--features ct`): the same proofs, but the
/// *check* is masked into a `subtle::CtOption` instead of branching — for when
/// the value is secret.
#[cfg(feature = "ct")]
fn constant_time() {
    use const_num_traits::CtNonZero; // brings `into_nonzero_ct` into scope

    // no `bool` is produced from the secret; the option is masked
    assert!(bool::from(Odd::<u64>::new_ct(0xABCD_1235).is_some()));
    assert!(bool::from(Odd::<u64>::new_ct(0xABCD_1234).is_none()));
    assert!(bool::from(42u32.into_nonzero_ct().is_some()));
    assert!(bool::from(0u32.into_nonzero_ct().is_none()));
}

fn main() {
    power_of_two();
    bit_index();
    nonzero_division();
    signed_range();
    nonmin();
    odd_modulus();
    finite_floats();
    #[cfg(feature = "ct")]
    constant_time();

    println!("all typestate examples ran");
}
