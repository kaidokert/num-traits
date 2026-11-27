//! Behaviour of the typestate proofs as a downstream crate sees them: equality,
//! formatting, and the `HasNonZero` bridge, checked exhaustively where the
//! domain allows.

use const_num_traits::{BitIndex, HasNonZero, Positive, PowerOfTwo, TypestateError};
use core::mem::size_of;

/// A struct that holds both proofs and derives `PartialEq`, `Eq` and `Debug`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Plan {
    block: PowerOfTwo<u32>,
    shift: BitIndex<i64>,
}

#[test]
fn power_of_two_equality_is_exponent_equality() {
    for a in 0..8u32 {
        for b in 0..8u32 {
            let pa = PowerOfTwo::<u8>::new(1 << a).unwrap();
            let pb = PowerOfTwo::<u8>::new(1 << b).unwrap();
            assert_eq!(pa == pb, a == b);
        }
    }
    for a in 0..128u32 {
        let p = PowerOfTwo::<u128>::new(1 << a).unwrap();
        assert_eq!(p, PowerOfTwo::<u128>::new_checked(1 << a).unwrap());
        for b in 0..128u32 {
            assert_eq!(p == PowerOfTwo::<u128>::new(1 << b).unwrap(), a == b);
        }
    }
}

#[test]
fn bit_index_equality_is_index_equality() {
    for a in 0..i8::BITS {
        for b in 0..i8::BITS {
            let ia = BitIndex::<i8>::new(a).unwrap();
            let ib = BitIndex::<i8>::new(b).unwrap();
            assert_eq!(ia == ib, a == b);
        }
    }
    for a in 0..u64::BITS {
        assert_eq!(
            BitIndex::<u64>::new(a).unwrap(),
            BitIndex::<u64>::new_checked(a).unwrap()
        );
    }
}

#[test]
fn proofs_format_their_representation() {
    assert_eq!(
        format!("{:?}", PowerOfTwo::<u32>::new(16).unwrap()),
        "PowerOfTwo { exp: 4 }"
    );
    assert_eq!(
        format!("{:?}", BitIndex::<u8>::new(3).unwrap()),
        "BitIndex(3)"
    );
    assert_eq!(
        format!("{:?}", PowerOfTwo::<u32>::new(3)),
        "None",
        "Option<PowerOfTwo> formats through the proof's Debug"
    );
}

#[test]
fn derived_plan_compares_and_formats() {
    let plan = Plan {
        block: PowerOfTwo::<u32>::new(64).unwrap(),
        shift: BitIndex::<i64>::new(12).unwrap(),
    };
    let copy = plan;
    assert_eq!(plan, copy);
    assert_ne!(
        plan,
        Plan {
            shift: BitIndex::<i64>::new(13).unwrap(),
            ..plan
        }
    );
    assert_eq!(
        format!("{plan:?}"),
        "Plan { block: PowerOfTwo { exp: 6 }, shift: BitIndex(12) }"
    );
}

fn round_trip<T: HasNonZero + Copy + PartialEq + Default>(value: T) {
    match value.into_nonzero() {
        Some(nz) => {
            assert!(value != T::default());
            assert!(T::nonzero_get(nz) == value);
        }
        None => assert!(value == T::default()),
    }
}

#[test]
fn signed_nonzero_bridge_exhaustive_narrow() {
    for value in i8::MIN..=i8::MAX {
        round_trip(value);
    }
    for value in i16::MIN..=i16::MAX {
        round_trip(value);
    }
}

#[test]
fn signed_nonzero_bridge_edges_wide() {
    macro_rules! edges {
        ($($t:ty),+) => {$(
            for value in [<$t>::MIN, <$t>::MIN + 1, -1, 0, 1, <$t>::MAX - 1, <$t>::MAX] {
                round_trip(value);
            }
            // The bridge's type is `core::num::NonZero`, niche included.
            assert_eq!(
                core::mem::size_of::<Option<<$t as HasNonZero>::NonZero>>(),
                core::mem::size_of::<$t>()
            );
        )+};
    }
    edges!(i32, i64, i128, isize);
}

/// The masked bridge agrees with `into_nonzero` for signed values, `MIN` and
/// negatives included.
#[cfg(feature = "ct")]
#[test]
fn signed_nonzero_ct_matches_checked_bridge() {
    use const_num_traits::CtNonZero;

    for value in (i8::MIN..=i8::MAX).step_by(if cfg!(miri) { 7 } else { 1 }) {
        let ct = value.into_nonzero_ct();
        assert_eq!(bool::from(ct.is_some()), value != 0, "{value}");
        assert_eq!(Option::from(ct), value.into_nonzero(), "{value}");
    }
    macro_rules! edges {
        ($($t:ty),+) => {$(
            for value in [<$t>::MIN, -1, 0, 1, <$t>::MAX] {
                assert_eq!(Option::from(value.into_nonzero_ct()), value.into_nonzero(), "{value}");
            }
        )+};
    }
    edges!(i16, i32, i64, i128, isize);
}

// ─────────────────────────────── Positive ───────────────────────────────────

// `Positive<T>` stores `NonZero<T>`, so `Option` of it needs no tag.
const _: () = {
    assert!(size_of::<Positive<i8>>() == 1);
    assert!(size_of::<Option<Positive<i8>>>() == 1);
    assert!(size_of::<Option<Positive<i16>>>() == 2);
    assert!(size_of::<Option<Positive<i32>>>() == 4);
    assert!(size_of::<Option<Positive<i64>>>() == 8);
    assert!(size_of::<Option<Positive<i128>>>() == 16);
    assert!(size_of::<Option<Positive<isize>>>() == size_of::<isize>());
    assert!(size_of::<Result<Positive<i32>, TypestateError>>() == 4);
};

// The constructors are `const` on stable.
const FIVE: Option<Positive<i32>> = Positive::<i32>::new(5);
const NONE: Option<Positive<i32>> = Positive::<i32>::new(0);
// SAFETY: 7 > 0.
const SEVEN: Positive<i64> = unsafe { Positive::<i64>::new_unchecked(7) };

#[test]
fn positive_const_constructors() {
    assert_eq!(FIVE.map(Positive::<i32>::get), Some(5));
    assert!(NONE.is_none());
    assert_eq!(SEVEN.get(), 7);
    assert_eq!(SEVEN.into_nonzero().get(), 7);
}

/// Checks every constructor, accessor and narrowing of `Positive<$t>` against
/// the plain value, for each of `$values`.
macro_rules! check_positive {
    ($t:ty, $u:ty, $values:expr) => {
        for value in $values {
            let proof = Positive::<$t>::new(value);
            assert_eq!(proof.is_some(), value > 0, "new({value})");
            assert_eq!(Positive::<$t>::try_from(value).ok(), proof);
            assert_eq!(Positive::<$t>::from_ref(&value).copied(), proof);
            let Some(p) = proof else { continue };
            // SAFETY: `value > 0` on this branch.
            assert_eq!(unsafe { Positive::<$t>::new_unchecked(value) }, p);
            assert_eq!(p.get(), value);
            assert_eq!(p.into_nonzero().get(), value);
            assert_eq!(p.to_unsigned(), value as $u);
            assert_eq!(p.abs(), value);
            assert_eq!(p.isqrt(), value.isqrt());
            assert_eq!(p.into_nonnegative().get(), value);
            assert_eq!(p.into_nonmin().get(), value);
            // The borrowed proof is the same memory, not a copy.
            let borrowed = Positive::<$t>::from_ref(&value).unwrap();
            assert!(core::ptr::eq(
                borrowed as *const Positive<$t> as *const $t,
                &value
            ));
        }
    };
}

/// Exhaustive natively; a stride under Miri, which is slow.
fn stride() -> usize {
    if cfg!(miri) { 251 } else { 1 }
}

#[test]
fn positive_exhaustive_narrow() {
    check_positive!(
        i8,
        u8,
        (i8::MIN..=i8::MAX).step_by(if cfg!(miri) { 7 } else { 1 })
    );
    check_positive!(i16, u16, (i16::MIN..=i16::MAX).step_by(stride()));
}

#[test]
fn positive_edges_wide() {
    macro_rules! edges {
        ($($t:ty => $u:ty),+) => {$(
            let edges = [<$t>::MIN, <$t>::MIN + 1, -2, -1, 0, 1, 2, <$t>::MAX - 1, <$t>::MAX];
            check_positive!($t, $u, edges);
        )+};
    }
    edges!(i32 => u32, i64 => u64, i128 => u128, isize => usize);
}

#[test]
fn positive_formats_as_its_value() {
    assert_eq!(
        format!("{:?}", Positive::<i32>::new(7).unwrap()),
        "Positive(7)"
    );
}

#[cfg(feature = "ct")]
#[test]
fn positive_ct_matches_checked_constructor() {
    for value in (i8::MIN..=i8::MAX).step_by(if cfg!(miri) { 3 } else { 1 }) {
        let ct = Positive::<i8>::new_ct(value);
        assert_eq!(bool::from(ct.is_some()), value > 0, "new_ct({value})");
        assert_eq!(Option::<Positive<i8>>::from(ct), Positive::<i8>::new(value));
    }
    for value in [i128::MIN, -1, 0, 1, i128::MAX] {
        let ct = Positive::<i128>::new_ct(value);
        assert_eq!(
            Option::<Positive<i128>>::from(ct),
            Positive::<i128>::new(value)
        );
    }
}
