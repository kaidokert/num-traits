//! Compile-time proof that the new trait impls are really `impl const` on
//! nightly: every value here is computed in a `const` initializer, which
//! fails to compile if any called impl is not const-callable.
//!
//! Only built with `cargo +nightly test --features nightly`.
#![cfg(feature = "nightly")]
#![feature(const_trait_impl)]

use const_num_traits::ops::overflowing::{OverflowingAdd, OverflowingShl};
use const_num_traits::{Algebraic, FloatBits, FromAscii, FromBytes, Maximum, NextUp, ToBytes};
use const_num_traits::{
    AbsDiff, BorrowingSub, CarrylessMul, CarryingAdd, CarryingMul, CastSigned, CastUnsigned,
    CheckedAddSigned, CheckedCast, CheckedPow, CheckedSignedDiff, ClampMagnitude, DepositBits,
    DivCeil, DivExact, DivFloor, FunnelShl, HighestOne, Ilog2, Isqrt, Midpoint, MultipleOf,
    NextMultipleOf, NextPowerOfTwo, OverflowingSubUnsigned, Parity, SaturatingAbs, SaturatingCast,
    ShlExact, StrictAdd, StrictEuclid, Truncate, UnboundedShr, UnsignedAbs, Widen, WideningMul,
    WrappingPow,
};

#[test]
fn bytes_in_const() {
    // `ToBytes::to_be_bytes` is by value; `FromBytes::from_*_bytes` borrows a
    // `&[u8; N]` and deref-copies it — both must be const-callable.
    const BE: [u8; 4] = ToBytes::to_be_bytes(0x12345678u32);
    const FROM_BE: u32 = FromBytes::from_be_bytes(&[0x12, 0x34, 0x56, 0x78]);
    const FROM_LE: u32 = FromBytes::from_le_bytes(&[0x78, 0x56, 0x34, 0x12]);
    assert_eq!(BE, [0x12, 0x34, 0x56, 0x78]);
    assert_eq!(FROM_BE, 0x12345678);
    assert_eq!(FROM_LE, 0x12345678);
}

#[test]
#[cfg(feature = "typestate")]
fn power_of_two_typestate_in_const() {
    use const_num_traits::{PowerOfTwo, PowerOfTwoOps};
    // Proof constructed and consumed entirely in `const` (the ops are const
    // only on nightly; `new`/`get` are const on stable too).
    const P: PowerOfTwo<u32> = PowerOfTwo::<u32>::new(16).unwrap();
    const Q: u32 = 100u32.div_pow2(P);
    const R: u32 = 100u32.rem_pow2(P);
    const M: bool = 96u32.is_multiple_of_pow2(P);
    assert_eq!(P.get(), 16);
    assert_eq!(Q, 6);
    assert_eq!(R, 4);
    assert!(M);
}

#[test]
fn variant_matrix_in_const() {
    const POW: Option<u32> = CheckedPow::checked_pow(3u32, 9);
    const WPOW: i8 = WrappingPow::wrapping_pow(3i8, 5);
    const SABS: i16 = SaturatingAbs::saturating_abs(i16::MIN);
    const OADD: (u8, bool) = OverflowingAdd::overflowing_add(255u8, 1);
    const OSHL: (u8, bool) = OverflowingShl::overflowing_shl(1u8, 9);
    const SADD: u8 = StrictAdd::strict_add(250u8, 5);
    const SEUC: i32 = StrictEuclid::strict_div_euclid(-7i32, 4);
    assert_eq!(POW, Some(19683));
    assert_eq!(WPOW, -13);
    assert_eq!(SABS, i16::MAX);
    assert_eq!(OADD, (0, true));
    assert_eq!(OSHL, (2, true));
    assert_eq!(SADD, 255);
    assert_eq!(SEUC, -2);
}

#[test]
fn bigint_helpers_in_const() {
    const CADD: (u64, bool) = CarryingAdd::carrying_add(u64::MAX, 0, true);
    const BSUB: (u64, bool) = BorrowingSub::borrowing_sub(0u64, 0, true);
    const CMUL: (u64, u64) = CarryingMul::carrying_mul(u64::MAX, u64::MAX, u64::MAX);
    const CMUL128: (u128, u128) = CarryingMul::carrying_mul(u128::MAX, u128::MAX, 0);
    const WMUL: u128 = WideningMul::widening_mul(u64::MAX, u64::MAX);
    const CLMUL: u8 = CarrylessMul::carryless_mul(0b11u8, 0b11);
    assert_eq!(CADD, (0, true));
    assert_eq!(BSUB, (u64::MAX, true));
    assert_eq!(CMUL, (0, u64::MAX));
    assert_eq!(CMUL128, (1, u128::MAX - 1));
    assert_eq!(WMUL, u64::MAX as u128 * u64::MAX as u128);
    assert_eq!(CLMUL, 0b101);
}

#[test]
fn rounding_log_sqrt_in_const() {
    const CEIL: i32 = DivCeil::div_ceil(-7i32, 2);
    const FLOOR: i32 = DivFloor::div_floor(-7i32, 2);
    const EXACT: Option<u32> = DivExact::div_exact(64u32, 4);
    const MULT: bool = MultipleOf::is_multiple_of(12u8, 4);
    const ODD: bool = Parity::is_odd(3u8);
    const NEXT: Option<u8> = NextMultipleOf::checked_next_multiple_of(23u8, 8);
    const MID: i8 = Midpoint::midpoint(-7i8, 0);
    const LOG: u32 = Ilog2::ilog2(1024u32);
    const SQRT: u64 = Isqrt::isqrt(99u64);
    const P2: Option<u8> = NextPowerOfTwo::checked_next_power_of_two(100u8);
    assert_eq!(CEIL, -3);
    assert_eq!(FLOOR, -4);
    assert_eq!(EXACT, Some(16));
    assert!(MULT);
    assert!(ODD);
    assert_eq!(NEXT, Some(24));
    assert_eq!(MID, -3);
    assert_eq!(LOG, 10);
    assert_eq!(SQRT, 9);
    assert_eq!(P2, Some(128));
}

#[test]
fn bits_in_const() {
    const USHR: i8 = UnboundedShr::unbounded_shr(-16i8, 200);
    const FUN: u8 = FunnelShl::funnel_shl(0x01u8, 0x80, 1);
    const EXSHL: Option<u8> = ShlExact::shl_exact(0x11u8, 3);
    const HI: Option<u32> = HighestOne::highest_one(0b0101_0000u8);
    const DEP: u8 = DepositBits::deposit_bits(0b101u8, 0b1111_0000);
    assert_eq!(USHR, -1);
    assert_eq!(FUN, 0x03);
    assert_eq!(EXSHL, Some(0x88));
    assert_eq!(HI, Some(6));
    assert_eq!(DEP, 0b0101_0000);
}

#[test]
fn ascii_and_floats_in_const() {
    const HEX: u32 = match <u32 as FromAscii>::from_ascii_radix(b"DeadBeef", 16) {
        Ok(v) => v,
        Err(_) => panic!("parse failed"),
    };
    const NEG: i16 = match <i16 as FromAscii>::from_ascii(b"-1234") {
        Ok(v) => v,
        Err(_) => panic!("parse failed"),
    };
    const BITS: u32 = FloatBits::to_bits(1.0f32);
    const ONE: f32 = FloatBits::from_bits(0x3F80_0000u32);
    const UP: f32 = NextUp::next_up(1.0f32);
    const MAX: f64 = Maximum::maximum(1.5f64, 2.5);
    const ALG: f64 = Algebraic::algebraic_mul(3.0f64, 0.5);
    assert_eq!(HEX, 0xdead_beef);
    assert_eq!(NEG, -1234);
    assert_eq!(BITS, 0x3F80_0000);
    assert_eq!(ONE, 1.0);
    assert_eq!(UP, f32::from_bits(0x3F80_0001));
    assert_eq!(MAX, 2.5);
    assert_eq!(ALG, 1.5);
}

#[test]
fn mixed_and_convert_in_const() {
    const ADDS: Option<u8> = CheckedAddSigned::checked_add_signed(1u8, -2);
    const SUBU: (i8, bool) = OverflowingSubUnsigned::overflowing_sub_unsigned(1i8, 2);
    const DIFF: Option<i8> = CheckedSignedDiff::checked_signed_diff(10u8, 14);
    const AD: u8 = AbsDiff::abs_diff(i8::MIN, i8::MAX);
    const UA: u128 = UnsignedAbs::unsigned_abs(i128::MIN);
    const CS: i8 = CastSigned::cast_signed(255u8);
    const CU: u8 = CastUnsigned::cast_unsigned(-1i8);
    const CM: i8 = ClampMagnitude::clamp_magnitude(120i8, 100);
    const W: u32 = Widen::widen(200u8);
    const T: u8 = Truncate::truncate(0x1234u16);
    const CC: Option<i8> = CheckedCast::checked_cast(200u8);
    const SC: u8 = SaturatingCast::saturating_cast(-5i32);
    assert_eq!(ADDS, None);
    assert_eq!(SUBU, (-1, false));
    assert_eq!(DIFF, Some(-4));
    assert_eq!(AD, 255);
    assert_eq!(UA, 1u128 << 127);
    assert_eq!(CS, -1);
    assert_eq!(CU, 255);
    assert_eq!(CM, 100);
    assert_eq!(W, 200);
    assert_eq!(T, 0x34);
    assert_eq!(CC, None);
    assert_eq!(SC, 0);
}
