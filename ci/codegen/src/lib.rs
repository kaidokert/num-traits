//! Consumers of the typestate accessors, compiled to assembly by the `check`
//! binary. Each function names what the proof should remove; `check` fails if
//! it is still there.
#![no_std]

use const_num_traits::{BitIndex, NonMin, NonNegative, Positive, PowerOfTwo};

// ── no overflow or division-by-zero panic ──────────────────────────────────

/// `Positive` rules out zero and negative divisors, hence `MIN / -1`.
#[no_mangle]
pub fn nopanic_div_positive_i32(n: i32, d: Positive<i32>) -> i32 {
    n / d.get()
}

#[no_mangle]
pub fn nopanic_rem_positive_i32(n: i32, d: Positive<i32>) -> i32 {
    n % d.get()
}

#[no_mangle]
pub fn nopanic_div_positive_i64(n: i64, d: Positive<i64>) -> i64 {
    n / d.get()
}

/// A nearest-rounding division, ties away from zero.
#[no_mangle]
pub fn nopanic_divide_nearest_i64(n: i64, d: Positive<i64>) -> i64 {
    let d = d.get();
    let q = n / d;
    let r = n - q * d;
    let m = r.unsigned_abs();
    if m < d.unsigned_abs() - m {
        q
    } else if r > 0 {
        q + 1
    } else {
        q - 1
    }
}

/// `NonMin` rules out the only overflowing dividend.
#[no_mangle]
pub fn nopanic_nonmin_div(n: NonMin<i32>, d: core::num::NonZero<i32>) -> i32 {
    n.div_nonzero(d)
}

#[no_mangle]
pub fn nopanic_nonmin_neg_abs(n: NonMin<i32>) -> i32 {
    n.neg().wrapping_add(n.abs())
}

/// The signed `isqrt` panics on negative input; the proofs rule it out.
#[no_mangle]
pub fn nopanic_nonnegative_isqrt(n: NonNegative<i32>) -> i32 {
    n.isqrt()
}

#[no_mangle]
pub fn nopanic_positive_isqrt(n: Positive<i32>) -> i32 {
    n.isqrt()
}

// ── no masking of shift amounts ────────────────────────────────────────────

#[no_mangle]
pub fn nomask_shift_i64(v: i64, s: BitIndex<i64>) -> i64 {
    v >> s.get()
}

#[no_mangle]
pub fn nomask_shift_u32(v: u32, s: BitIndex<u32>) -> u32 {
    v << s.get()
}

/// `31 - s` is in range only because `s < 32`.
#[no_mangle]
pub fn nomask_complement_u32(v: u32, s: BitIndex<u32>) -> u32 {
    v << (31 - s.get())
}

#[no_mangle]
pub fn nomask_pow2_mask_u32(v: u32, p: PowerOfTwo<u32>) -> u32 {
    v & ((1u32 << p.exp()) - 1)
}

// Only on the bare-metal targets; the host build, for `check`, uses std's.
#[cfg(target_os = "none")]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
