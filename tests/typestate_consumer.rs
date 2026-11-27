//! The proofs used the way a downstream fixed-point library uses them: a
//! nearest-rounding division by a `Positive` divisor, and a rounding shift by
//! a `BitIndex`, each with no error path. Checked against a wide reference,
//! exhaustively at `i8` and at the edges of `i64` and `i128`.

use const_num_traits::{BitIndex, Positive};

/// `numerator / denominator`, rounded to nearest with ties away from zero,
/// using one division. Total: a positive divisor excludes both division by
/// zero and `MIN / -1`.
macro_rules! divide_nearest {
    ($name:ident, $t:ty) => {
        const fn $name(numerator: $t, denominator: Positive<$t>) -> $t {
            let denominator = denominator.get();
            let quotient = numerator / denominator;
            let remainder = numerator - quotient * denominator;
            // Rounds away when `2|r| >= d`, compared as `|r| >= d - |r|`
            // because the doubling can overflow and the subtraction can't
            // (`|r| < d`).
            let magnitude = remainder.unsigned_abs();
            if magnitude < denominator.unsigned_abs() - magnitude {
                quotient
            } else if remainder > 0 {
                quotient + 1
            } else {
                quotient - 1
            }
        }
    };
}
divide_nearest!(divide_nearest_i8, i8);
divide_nearest!(divide_nearest_i128, i128);

/// `value / 2^shift`, rounded to nearest with ties away from zero, by shift
/// and mask. Total for every proven shift, `BITS - 1` included.
macro_rules! shift_nearest {
    ($name:ident, $t:ty, $u:ty) => {
        const fn $name(value: $t, shift: BitIndex<$t>) -> $t {
            let k = shift.get();
            if k == 0 {
                return value;
            }
            let magnitude = value.unsigned_abs();
            let half: $u = 1 << (k - 1);
            let rounded = (magnitude >> k) + ((magnitude & ((half << 1) - 1) >= half) as $u);
            if value < 0 {
                (rounded as $t).wrapping_neg()
            } else {
                rounded as $t
            }
        }
    };
}
shift_nearest!(shift_nearest_i8, i8, u8);
shift_nearest!(shift_nearest_i64, i64, u64);

/// Reference: round-half-away-from-zero of `n / d` in `i128`, for `|n|`, `d`
/// well inside its range.
fn reference(n: i128, d: i128) -> i128 {
    let (q, r) = (n.abs() / d, n.abs() % d);
    let rounded = if 2 * r >= d { q + 1 } else { q };
    if n < 0 { -rounded } else { rounded }
}

#[test]
fn divide_nearest_exhaustive_i8() {
    for d in 1..=i8::MAX {
        let divisor = Positive::<i8>::new(d).unwrap();
        for n in i8::MIN..=i8::MAX {
            let expected = reference(n.into(), d.into());
            assert_eq!(
                i128::from(divide_nearest_i8(n, divisor)),
                expected,
                "{n} / {d}"
            );
        }
    }
}

#[test]
fn divide_nearest_edges_i128() {
    let divisors = [1, 2, 3, 7, 1 << 64, i128::MAX - 1, i128::MAX];
    let numerators = [
        i128::MIN + 1,
        i128::MIN / 2,
        -3,
        -2,
        -1,
        0,
        1,
        2,
        3,
        i128::MAX / 2,
        i128::MAX,
    ];
    for d in divisors {
        let divisor = Positive::<i128>::new(d).unwrap();
        for n in numerators {
            // Independent of the code under test: `2|r| >= d`, in `u128`
            // where `2|r| < 2d <= 2^128` can't overflow.
            let (q, r) = (n / d, n % d);
            let away = 2 * r.unsigned_abs() >= d.unsigned_abs();
            let expected = match (away, r.signum()) {
                (true, 1) => q + 1,
                (true, -1) => q - 1,
                _ => q,
            };
            assert_eq!(divide_nearest_i128(n, divisor), expected, "{n} / {d}");
        }
    }
    // MIN itself divides exactly by 1 and 2.
    let one = Positive::<i128>::new(1).unwrap();
    let two = Positive::<i128>::new(2).unwrap();
    assert_eq!(divide_nearest_i128(i128::MIN, one), i128::MIN);
    assert_eq!(divide_nearest_i128(i128::MIN, two), i128::MIN / 2);
}

#[test]
fn shift_nearest_exhaustive_i8() {
    for k in 0..i8::BITS {
        let shift = BitIndex::<i8>::new(k).unwrap();
        for v in i8::MIN..=i8::MAX {
            let expected = reference(v.into(), 1 << k);
            let got = shift_nearest_i8(v, shift);
            assert_eq!(i128::from(got), expected, "{v} >> {k}");
        }
    }
}

#[test]
fn shift_nearest_edges_i64() {
    for k in 0..i64::BITS {
        let shift = BitIndex::<i64>::new(k).unwrap();
        for v in [i64::MIN + 1, -3, -1, 0, 1, 3, i64::MAX] {
            assert_eq!(
                i128::from(shift_nearest_i64(v, shift)),
                reference(v.into(), 1 << k),
                "{v} >> {k}"
            );
        }
    }
}

/// Builds the proof in a `const`, so an out-of-range shift is a compile error
/// and no check is left at runtime.
const fn accepted_shift(shift: u32) -> BitIndex<i64> {
    match BitIndex::<i64>::new(shift) {
        Some(index) => index,
        None => panic!("shift must be below 64"),
    }
}
const SHIFT: BitIndex<i64> = accepted_shift(12);
const DIVISOR: Positive<i128> = match Positive::<i128>::new(10) {
    Some(divisor) => divisor,
    None => panic!("divisor must be positive"),
};
const ROUNDED: i64 = shift_nearest_i64(6144, SHIFT);
const QUOTIENT: i128 = divide_nearest_i128(-25, DIVISOR);

#[test]
fn proofs_drive_const_evaluation() {
    assert_eq!(ROUNDED, 2); // 6144 / 4096 = 1.5, rounds away
    assert_eq!(QUOTIENT, -3); // -2.5 rounds away
}
