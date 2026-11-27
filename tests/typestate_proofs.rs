//! Behaviour of the typestate proofs as a downstream crate sees them: equality,
//! formatting, and the `HasNonZero` bridge, checked exhaustively where the
//! domain allows.

use const_num_traits::{BitIndex, HasNonZero, PowerOfTwo};

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
