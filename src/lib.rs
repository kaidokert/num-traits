// Copyright 2013-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Numeric traits for generic mathematics
//!
//! ## Constant-time (CT) tiers
//!
//! Every operation trait in this crate is classified by how implementable it
//! is for constant-time integer types (types whose execution time must not
//! depend on operand *values*):
//!
//! - **Tier A — CT-implementable**: branchless on the data; secret values
//!   flow only through arithmetic/bitwise instructions. Examples: the
//!   `Wrapping*`/`Overflowing*`/`Carrying*` families, [`PrimBits`],
//!   [`Midpoint`], [`AbsDiff`], [`CastSigned`], lossy [`Truncate`].
//! - **Tier B — caller-leaky**: the operation itself can be implemented
//!   branchlessly, but the `bool`/`Option` return type invites branching on
//!   secret-derived data at the call site. Examples: the `Checked*` family,
//!   [`Parity`], [`IsPowerOfTwo`], [`HighestOne`]. With the `ct` cargo
//!   feature, masked counterparts returning `subtle::Choice`/`CtOption`
//!   are available in `ops::ct`.
//! - **Tier C — CT-hostile**: data-dependent control flow, division, or
//!   data-dependent panics. Examples: everything `Div`/`Rem`-based
//!   ([`Euclid`], [`DivCeil`], [`Ilog10`], [`NextMultipleOf`]), the
//!   `Strict*` family (panics on a data-dependent condition),
//!   [`Num::from_str_radix`].
//!
//! **Public-parameter convention**: shift amounts, rotation counts,
//! exponents and logarithm bases are treated as *public* values — branching
//! on them does not demote a trait from Tier A/B. If your shift amount is
//! itself a secret, none of these traits are appropriate as-is.
//!
//! The tier of each trait family is noted in its module documentation
//! under [`ops`]. Rule of thumb maintained by this crate: a Tier-A trait
//! never has a Tier-C supertrait, and never requires
//! `PartialEq`/`Ord`/`Div`/`Rem`.
//!
//! ## Compatibility
//!
//! The `const-num-traits` crate is tested for rustc 1.86 and greater.

#![deny(unconditional_recursion)]
// By-value receivers mirror core's inherent methods, so `is_*` predicates take
// `self` by value — intentional, though clippy's `wrong_self_convention` flags it.
#![allow(clippy::wrong_self_convention)]
// The const parsers (`from_ascii`, the float `from_str_radix`) can't call the
// non-const `RangeInclusive::contains`, so they spell range checks out longhand.
#![allow(clippy::manual_range_contains)]
#![no_std]
#![cfg_attr(
    feature = "nightly",
    feature(const_trait_impl, const_ops, const_cmp, const_destruct)
)]
// docs.rs sets `--cfg docsrs`; enable `doc(cfg(...))` so feature-gated items
// (e.g. the `ct` module) render with a "Available on crate feature …" label.
#![cfg_attr(docsrs, feature(doc_cfg))]

// Need to explicitly bring the crate in for inherent float methods
#[cfg(feature = "std")]
extern crate std;

use core::fmt;
use core::num::Wrapping;
use core::ops::{Add, Div, Mul, Rem, Sub};
use core::ops::{AddAssign, DivAssign, MulAssign, RemAssign, SubAssign};

pub use crate::bounds::Bounded;
#[cfg(any(feature = "std", feature = "libm"))]
pub use crate::float::Float;
pub use crate::float::FloatConst;
// pub use real::{FloatCore, Real}; // NOTE: Don't do this, it breaks `use const_num_traits::*;`.
pub use crate::cast::{AsPrimitive, FromPrimitive, NumCast, ToPrimitive, cast};
pub use crate::identities::{ConstOne, ConstZero, One, Zero, one, zero};
pub use crate::int::{PrimBits, PrimInt};
pub use crate::ops::bits::{
    BitWidth, BitsPrecision, DepositBits, ExtractBits, FunnelShl, FunnelShr, HighestOne,
    IsolateHighestOne, IsolateLowestOne, LowestOne, ShlExact, ShrExact, UnboundedShl, UnboundedShr,
    WithPrecision,
};
pub use crate::ops::byte_slice::{ByteSliceError, ByteSliceErrorKind, FromByteSlice};
pub use crate::ops::bytes::{FromBytes, ToBytes};
pub use crate::ops::carrying::{BorrowingSub, CarryingAdd, CarryingMul, WideningMul};
pub use crate::ops::checked::{
    CheckedAbs, CheckedAdd, CheckedDiv, CheckedMul, CheckedNeg, CheckedPow, CheckedRem, CheckedShl,
    CheckedShr, CheckedSub,
};
pub use crate::ops::clmul::{CarryingCarrylessMul, CarrylessMul, WideningCarrylessMul};
pub use crate::ops::convert::{
    AbsDiff, CastSigned, CastUnsigned, CheckedCast, ClampMagnitude, SaturatingCast, StrictCast,
    Truncate, UnsignedAbs, Widen, WrappingCast,
};
#[cfg(feature = "ct")]
#[cfg_attr(docsrs, doc(cfg(feature = "ct")))]
pub use crate::ops::ct::{
    CtCheckedAdd, CtCheckedMul, CtCheckedNeg, CtCheckedSignedDiff, CtCheckedSub, CtIsPowerOfTwo,
    CtIsZero, CtParity,
};
pub use crate::ops::euclid::{CheckedEuclid, Euclid, OverflowingEuclid, WrappingEuclid};
pub use crate::ops::float_ops::{
    Algebraic, Erf, FloatBits, Gamma, Maximum, Minimum, NextDown, NextUp, RoundTiesEven,
};
pub use crate::ops::from_ascii::{AsciiErrorKind, AsciiParseError, FromAscii};
pub use crate::ops::inv::Inv;
pub use crate::ops::log::{Ilog, Ilog2, Ilog10};
pub use crate::ops::mixed::{
    CheckedAddSigned, CheckedAddUnsigned, CheckedSignedDiff, CheckedSubSigned, CheckedSubUnsigned,
    OverflowingAddSigned, OverflowingAddUnsigned, OverflowingSubSigned, OverflowingSubUnsigned,
    SaturatingAddSigned, SaturatingAddUnsigned, SaturatingSubSigned, SaturatingSubUnsigned,
    StrictAddSigned, StrictAddUnsigned, StrictSubSigned, StrictSubUnsigned, WrappingAddSigned,
    WrappingAddUnsigned, WrappingSubSigned, WrappingSubUnsigned,
};
pub use crate::ops::mul_add::{MulAdd, MulAddAssign};
pub use crate::ops::overflowing::{
    OverflowingAbs, OverflowingAdd, OverflowingDiv, OverflowingMul, OverflowingNeg, OverflowingPow,
    OverflowingRem, OverflowingShl, OverflowingShr, OverflowingSub,
};
pub use crate::ops::parity::Parity;
pub use crate::ops::pow2::{IsPowerOfTwo, NextPowerOfTwo};
pub use crate::ops::rounding::{DivCeil, DivExact, DivFloor, Midpoint, MultipleOf, NextMultipleOf};
pub use crate::ops::saturating::{
    Saturating, SaturatingAbs, SaturatingAdd, SaturatingDiv, SaturatingMul, SaturatingNeg,
    SaturatingPow, SaturatingSub,
};
pub use crate::ops::sqrt::{CheckedIsqrt, Isqrt};
pub use crate::ops::strict::{
    StrictAbs, StrictAdd, StrictDiv, StrictEuclid, StrictMul, StrictNeg, StrictPow, StrictRem,
    StrictShl, StrictShr, StrictSub,
};
#[cfg(feature = "ct")]
#[cfg_attr(docsrs, doc(cfg(feature = "ct")))]
pub use crate::ops::typestate::CtNonZero;
pub use crate::ops::typestate::{
    BitIndex, BitIndexOps, DivNonZero, Even, Finite, HasNonZero, NonMin, NonNegative, Odd,
    Positive, PowerOfTwo, PowerOfTwoOps, TypestateError,
};
pub use crate::ops::wrapping::{
    WrappingAbs, WrappingAdd, WrappingDiv, WrappingMul, WrappingNeg, WrappingPow, WrappingRem,
    WrappingShl, WrappingShr, WrappingSub,
};
pub use crate::personality::{
    Ct, HasPersonality, Nct, Personality, PersonalityMarker, PersonalityTag,
};
pub use crate::pow::{Pow, checked_pow, pow};
pub use crate::sign::{Signed, Signum, Unsigned, abs, abs_sub, signum};

#[macro_use]
mod macros;

pub mod bounds;
pub mod cast;
pub mod float;
pub mod identities;
pub mod int;
pub mod ops;
pub mod personality;
pub mod pow;
pub mod real;
pub mod sign;

/// One-stop trait import: `use const_num_traits::prelude::*;`
///
/// Brings every trait in the crate into scope — both the num-traits-compatible
/// bundles and the fine-grained modern atoms. This matters after the
/// bundle-to-supertrait extractions: with only a bundle
/// imported (e.g. `PrimInt`), method-syntax calls to methods that moved to a
/// supertrait (e.g. `count_ones` on `PrimBits`) don't resolve on concrete
/// non-primitive types. Importing the prelude makes that a non-issue.
pub mod prelude {
    pub use crate::bounds::*;
    pub use crate::cast::*;
    pub use crate::float::*;
    pub use crate::identities::*;
    pub use crate::int::*;
    pub use crate::ops::bits::*;
    pub use crate::ops::byte_slice::*;
    pub use crate::ops::bytes::*;
    pub use crate::ops::carrying::*;
    pub use crate::ops::checked::*;
    pub use crate::ops::clmul::*;
    pub use crate::ops::convert::*;
    #[cfg(feature = "ct")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ct")))]
    pub use crate::ops::ct::*;
    pub use crate::ops::euclid::*;
    pub use crate::ops::float_ops::*;
    pub use crate::ops::from_ascii::*;
    pub use crate::ops::inv::*;
    pub use crate::ops::log::*;
    pub use crate::ops::mixed::*;
    pub use crate::ops::mul_add::*;
    pub use crate::ops::overflowing::*;
    pub use crate::ops::parity::*;
    pub use crate::ops::pow2::*;
    pub use crate::ops::rounding::*;
    pub use crate::personality::*;
    // typestate *traits* only (for method resolution); the wrapper *types*
    // (`PowerOfTwo`/`Odd`/`Even`/`Positive`/`NonNegative`) stay crate-root-only,
    // so the glob doesn't inject those generic names into consumers.
    pub use crate::ops::saturating::*;
    pub use crate::ops::sqrt::*;
    pub use crate::ops::strict::*;
    #[cfg(feature = "ct")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ct")))]
    pub use crate::ops::typestate::CtNonZero;
    pub use crate::ops::typestate::{BitIndexOps, DivNonZero, HasNonZero, PowerOfTwoOps};
    pub use crate::ops::wrapping::*;
    pub use crate::pow::*;
    #[cfg(any(feature = "std", feature = "libm"))]
    pub use crate::real::*;
    pub use crate::sign::*;
    pub use crate::{
        FromStrRadix, Num, NumAssign, NumAssignOps, NumAssignRef, NumOps, NumRef, RefNum, RingOps,
    };
}

c0nst::c0nst! {
/// The base trait for numeric types, covering `0` and `1` values,
/// comparisons, basic numeric operations, and string conversion.
pub c0nst trait Num: [c0nst] PartialEq + [c0nst] Zero + [c0nst] One + [c0nst] NumOps {
    type FromStrRadixErr;

    /// Convert from a string and radix (typically `2..=36`).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use const_num_traits::Num;
    ///
    /// let result = <i32 as Num>::from_str_radix("27", 10);
    /// assert_eq!(result, Ok(27));
    ///
    /// let result = <i32 as Num>::from_str_radix("foo", 10);
    /// assert!(result.is_err());
    /// ```
    ///
    /// # Supported radices
    ///
    /// The exact range of supported radices is at the discretion of each type implementation. For
    /// primitive integers, this is implemented by the inherent `from_str_radix` methods in the
    /// standard library, which **panic** if the radix is not in the range from 2 to 36. The
    /// implementation in this crate for primitive floats is similar.
    ///
    /// For third-party types, it is suggested that implementations should follow suit and at least
    /// accept `2..=36` without panicking, but an `Err` may be returned for any unsupported radix.
    /// It's possible that a type might not even support the common radix 10, nor any, if string
    /// parsing doesn't make sense for that type.
    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr>;
}
}

c0nst::c0nst! {
/// Generic trait for types implementing the division-free basic numeric
/// operations: addition, subtraction and multiplication.
///
/// This is the aggregation to bound on when the type may not expose
/// division — e.g. constant-time integers, where division is inherently
/// data-dependent. [`NumOps`] extends it with `Div` and `Rem`.
///
/// This is automatically implemented for types which implement the operators.
pub c0nst trait RingOps<Rhs = Self, Output = Self>:
    [c0nst] Add<Rhs, Output = Output>
    + [c0nst] Sub<Rhs, Output = Output>
    + [c0nst] Mul<Rhs, Output = Output>
{
}
}

c0nst::c0nst! {
c0nst impl<T, Rhs, Output> RingOps<Rhs, Output> for T where
    T: [c0nst] Add<Rhs, Output = Output>
        + [c0nst] Sub<Rhs, Output = Output>
        + [c0nst] Mul<Rhs, Output = Output>
{
}
}

c0nst::c0nst! {
/// Generic trait for types implementing basic numeric operations
///
/// This is automatically implemented for types which implement the operators.
pub c0nst trait NumOps<Rhs = Self, Output = Self>:
    [c0nst] RingOps<Rhs, Output>
    + [c0nst] Div<Rhs, Output = Output>
    + [c0nst] Rem<Rhs, Output = Output>
{
}
}

c0nst::c0nst! {
c0nst impl<T, Rhs, Output> NumOps<Rhs, Output> for T where
    T: [c0nst] Add<Rhs, Output = Output>
        + [c0nst] Sub<Rhs, Output = Output>
        + [c0nst] Mul<Rhs, Output = Output>
        + [c0nst] Div<Rhs, Output = Output>
        + [c0nst] Rem<Rhs, Output = Output>
{
}
}

c0nst::c0nst! {
/// The trait for `Num` types which also implement numeric operations taking
/// the second operand by reference.
///
/// This is automatically implemented for types which implement the operators.
pub c0nst trait NumRef: [c0nst] Num + for<'r> [c0nst] NumOps<&'r Self> {}
}
c0nst::c0nst! {
c0nst impl<T> NumRef for T where T: [c0nst] Num + for<'r> [c0nst] NumOps<&'r T> {}
}

c0nst::c0nst! {
/// The trait for `Num` references which implement numeric operations, taking the
/// second operand either by value or by reference.
///
/// This is automatically implemented for all types which implement the operators. It covers
/// every type implementing the operations though, regardless of it being a reference or
/// related to `Num`.
pub c0nst trait RefNum<Base>: [c0nst] NumOps<Base, Base> + for<'r> [c0nst] NumOps<&'r Base, Base> {}
}
c0nst::c0nst! {
c0nst impl<T, Base> RefNum<Base> for T where T: [c0nst] NumOps<Base, Base> + for<'r> [c0nst] NumOps<&'r Base, Base> {}
}

c0nst::c0nst! {
/// Generic trait for types implementing numeric assignment operators (like `+=`).
///
/// This is automatically implemented for types which implement the operators.
pub c0nst trait NumAssignOps<Rhs = Self>:
    [c0nst] AddAssign<Rhs> + [c0nst] SubAssign<Rhs> + [c0nst] MulAssign<Rhs> + [c0nst] DivAssign<Rhs> + [c0nst] RemAssign<Rhs>
{
}
}

c0nst::c0nst! {
c0nst impl<T, Rhs> NumAssignOps<Rhs> for T where
    T: [c0nst] AddAssign<Rhs> + [c0nst] SubAssign<Rhs> + [c0nst] MulAssign<Rhs> + [c0nst] DivAssign<Rhs> + [c0nst] RemAssign<Rhs>
{
}
}

c0nst::c0nst! {
/// The trait for `Num` types which also implement assignment operators.
///
/// This is automatically implemented for types which implement the operators.
pub c0nst trait NumAssign: [c0nst] Num + [c0nst] NumAssignOps {}
}
c0nst::c0nst! {
c0nst impl<T> NumAssign for T where T: [c0nst] Num + [c0nst] NumAssignOps {}
}

c0nst::c0nst! {
/// The trait for `NumAssign` types which also implement assignment operations
/// taking the second operand by reference.
///
/// This is automatically implemented for types which implement the operators.
pub c0nst trait NumAssignRef: [c0nst] NumAssign + for<'r> [c0nst] NumAssignOps<&'r Self> {}
}
c0nst::c0nst! {
c0nst impl<T> NumAssignRef for T where T: [c0nst] NumAssign + for<'r> [c0nst] NumAssignOps<&'r T> {}
}

/// Conversion from a string in a given radix.
///
/// This is the standalone atom for the parsing capability that [`Num`]
/// bundles; implement both for full compatibility. (`Num` keeps its own
/// `FromStrRadixErr` associated type and method because associated types
/// can't be re-exported through supertraits.)
///
/// This is a plain (never-const) trait: string parsing is not
/// const-evaluable for any of the primitive types today.
pub trait FromStrRadix: Sized {
    /// The parse error type.
    type Err;

    /// Convert from a string and radix (typically `2..=36`); see
    /// [`Num::from_str_radix`] for the conventions around supported
    /// radices.
    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::Err>;
}

macro_rules! from_str_radix_atom_impl {
    ($($t:ty)*) => ($(
        impl FromStrRadix for $t {
            type Err = ::core::num::ParseIntError;
            #[inline]
            fn from_str_radix(s: &str, radix: u32) -> Result<Self, Self::Err> {
                <$t>::from_str_radix(s, radix)
            }
        }
    )*)
}
from_str_radix_atom_impl!(usize u8 u16 u32 u64 u128);
from_str_radix_atom_impl!(isize i8 i16 i32 i64 i128);

macro_rules! from_str_radix_atom_float_impl {
    ($($t:ty)*) => ($(
        impl FromStrRadix for $t {
            type Err = ParseFloatError;
            #[inline]
            fn from_str_radix(s: &str, radix: u32) -> Result<Self, Self::Err> {
                // reuse the crate's float radix parser through the Num impl
                <$t as Num>::from_str_radix(s, radix)
            }
        }
    )*)
}

impl<T: FromStrRadix> FromStrRadix for Wrapping<T> {
    type Err = T::Err;
    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::Err> {
        T::from_str_radix(str, radix).map(Wrapping)
    }
}

impl<T: FromStrRadix> FromStrRadix for core::num::Saturating<T> {
    type Err = T::Err;
    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::Err> {
        T::from_str_radix(str, radix).map(core::num::Saturating)
    }
}

macro_rules! int_trait_impl {
    ($name:ident for $($t:ty)*) => ($(
        c0nst::c0nst! {
        c0nst impl $name for $t {
            type FromStrRadixErr = ::core::num::ParseIntError;
            #[inline]
            fn from_str_radix(s: &str, radix: u32)
                              -> Result<Self, ::core::num::ParseIntError>
            {
                <$t>::from_str_radix(s, radix)
            }
        }
        }
    )*)
}
int_trait_impl!(Num for usize u8 u16 u32 u64 u128);
int_trait_impl!(Num for isize i8 i16 i32 i64 i128);

// Wrapping<T>'s `PartialEq` impl in std isn't const yet, so this stays a
// non-const impl of the (otherwise const) `Num` trait.
impl<T: Num> Num for Wrapping<T>
where
    Wrapping<T>: NumOps,
{
    type FromStrRadixErr = T::FromStrRadixErr;
    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        T::from_str_radix(str, radix).map(Wrapping)
    }
}

// Same caveat as Wrapping<T>: no const PartialEq impl in std.
impl<T: Num> Num for core::num::Saturating<T>
where
    core::num::Saturating<T>: NumOps,
{
    type FromStrRadixErr = T::FromStrRadixErr;
    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        T::from_str_radix(str, radix).map(core::num::Saturating)
    }
}

#[derive(Debug)]
pub enum FloatErrorKind {
    Empty,
    Invalid,
}
// FIXME: core::num::ParseFloatError is stable in 1.0, but opaque to us,
// so there's not really any way for us to reuse it.
#[derive(Debug)]
pub struct ParseFloatError {
    pub kind: FloatErrorKind,
}

impl fmt::Display for ParseFloatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self.kind {
            FloatErrorKind::Empty => "cannot parse float from empty string",
            FloatErrorKind::Invalid => "invalid float literal",
        };

        description.fmt(f)
    }
}

fn str_to_ascii_lower_eq_str(a: &str, b: &str) -> bool {
    a.len() == b.len()
        && a.bytes().zip(b.bytes()).all(|(a, b)| {
            let a_to_ascii_lower = a | ((a.is_ascii_uppercase() as u8) << 5);
            a_to_ascii_lower == b
        })
}

// FIXME: The standard library from_str_radix on floats was deprecated, so we're stuck
// with this implementation ourselves until we want to make a breaking change.
// (would have to drop it from `Num` though)
//
// Non-const impl of the const `Num` trait: this parser uses iterators,
// `Result::map_err`, `?`, and `str::parse`, none of which are const.
macro_rules! float_trait_impl {
    ($name:ident for $($t:ident)*) => ($(
        impl $name for $t {
            type FromStrRadixErr = ParseFloatError;

            fn from_str_radix(src: &str, radix: u32)
                              -> Result<Self, Self::FromStrRadixErr>
            {
                use self::FloatErrorKind::*;
                use self::ParseFloatError as PFE;

                // Special case radix 10 to use more accurate standard library implementation
                if radix == 10 {
                    return src.parse().map_err(|_| PFE {
                        kind: if src.is_empty() { Empty } else { Invalid },
                    });
                }

                // Special values
                if str_to_ascii_lower_eq_str(src, "inf")
                    || str_to_ascii_lower_eq_str(src, "infinity")
                {
                    return Ok($t::INFINITY);
                } else if str_to_ascii_lower_eq_str(src, "-inf")
                    || str_to_ascii_lower_eq_str(src, "-infinity")
                {
                    return Ok($t::NEG_INFINITY);
                } else if str_to_ascii_lower_eq_str(src, "nan") {
                    return Ok($t::NAN);
                } else if str_to_ascii_lower_eq_str(src, "-nan") {
                    return Ok(-$t::NAN);
                }

                fn slice_shift_char(src: &str) -> Option<(char, &str)> {
                    let mut chars = src.chars();
                    Some((chars.next()?, chars.as_str()))
                }

                let (is_positive, src) =  match slice_shift_char(src) {
                    None             => return Err(PFE { kind: Empty }),
                    Some(('-', ""))  => return Err(PFE { kind: Empty }),
                    Some(('-', src)) => (false, src),
                    Some((_, _))     => (true,  src),
                };

                // The significand to accumulate
                let mut sig = if is_positive { 0.0 } else { -0.0 };
                // Necessary to detect overflow
                let mut prev_sig = sig;
                let mut cs = src.chars().enumerate();
                // Exponent prefix and exponent index offset
                let mut exp_info = None::<(char, usize)>;

                // Parse the integer part of the significand
                for (i, c) in cs.by_ref() {
                    match c.to_digit(radix) {
                        Some(digit) => {
                            // shift significand one digit left
                            sig *= radix as $t;

                            // add/subtract current digit depending on sign
                            if is_positive {
                                sig += (digit as isize) as $t;
                            } else {
                                sig -= (digit as isize) as $t;
                            }

                            // Detect overflow by comparing to last value, except
                            // if we've not seen any non-zero digits.
                            if prev_sig != 0.0 {
                                if is_positive && sig <= prev_sig
                                    { return Ok($t::INFINITY); }
                                if !is_positive && sig >= prev_sig
                                    { return Ok($t::NEG_INFINITY); }

                                // Detect overflow by reversing the shift-and-add process
                                if is_positive && (prev_sig != (sig - digit as $t) / radix as $t)
                                    { return Ok($t::INFINITY); }
                                if !is_positive && (prev_sig != (sig + digit as $t) / radix as $t)
                                    { return Ok($t::NEG_INFINITY); }
                            }
                            prev_sig = sig;
                        },
                        None => match c {
                            'e' | 'E' | 'p' | 'P' => {
                                exp_info = Some((c, i + 1));
                                break;  // start of exponent
                            },
                            '.' => {
                                break;  // start of fractional part
                            },
                            _ => {
                                return Err(PFE { kind: Invalid });
                            },
                        },
                    }
                }

                // If we are not yet at the exponent parse the fractional
                // part of the significand
                if exp_info.is_none() {
                    let mut power = 1.0;
                    for (i, c) in cs.by_ref() {
                        match c.to_digit(radix) {
                            Some(digit) => {
                                // Decrease power one order of magnitude
                                power /= radix as $t;
                                // add/subtract current digit depending on sign
                                sig = if is_positive {
                                    sig + (digit as $t) * power
                                } else {
                                    sig - (digit as $t) * power
                                };
                                // Detect overflow by comparing to last value
                                if is_positive && sig < prev_sig
                                    { return Ok($t::INFINITY); }
                                if !is_positive && sig > prev_sig
                                    { return Ok($t::NEG_INFINITY); }
                                prev_sig = sig;
                            },
                            None => match c {
                                'e' | 'E' | 'p' | 'P' => {
                                    exp_info = Some((c, i + 1));
                                    break; // start of exponent
                                },
                                _ => {
                                    return Err(PFE { kind: Invalid });
                                },
                            },
                        }
                    }
                }

                // Parse and calculate the exponent
                let exp = match exp_info {
                    Some((c, offset)) => {
                        let base = match c {
                            'E' | 'e' if radix == 10 => 10.0,
                            'P' | 'p' if radix == 16 => 2.0,
                            _ => return Err(PFE { kind: Invalid }),
                        };

                        // Parse the exponent as decimal integer
                        let src = &src[offset..];
                        let (is_positive, exp) = match slice_shift_char(src) {
                            Some(('-', src)) => (false, src.parse::<usize>()),
                            Some(('+', src)) => (true,  src.parse::<usize>()),
                            Some((_, _))     => (true,  src.parse::<usize>()),
                            None             => return Err(PFE { kind: Invalid }),
                        };

                        #[cfg(feature = "std")]
                        fn pow(base: $t, exp: usize) -> $t {
                            Float::powi(base, exp as i32)
                        }
                        // otherwise uses the generic `pow` from the root

                        match (is_positive, exp) {
                            (true,  Ok(exp)) => pow(base, exp),
                            (false, Ok(exp)) => 1.0 / pow(base, exp),
                            (_, Err(_))      => return Err(PFE { kind: Invalid }),
                        }
                    },
                    None => 1.0, // no exponent
                };

                Ok(sig * exp)
            }
        }
    )*)
}
float_trait_impl!(Num for f32 f64);
from_str_radix_atom_float_impl!(f32 f64);

c0nst::c0nst! {
/// A value bounded by a minimum and a maximum
///
///  If input is less than min then this returns min.
///  If input is greater than max then this returns max.
///  Otherwise this returns input.
///
/// **Panics** in debug mode if `!(min <= max)`.
#[inline]
pub c0nst fn clamp<T: [c0nst] PartialOrd + [c0nst] Destruct>(input: T, min: T, max: T) -> T {
    debug_assert!(min <= max, "min must be less than or equal to max");
    if input < min {
        min
    } else if input > max {
        max
    } else {
        input
    }
}
}

c0nst::c0nst! {
/// A value bounded by a minimum value
///
///  If input is less than min then this returns min.
///  Otherwise this returns input.
///  `clamp_min(std::f32::NAN, 1.0)` preserves `NAN` different from `f32::min(std::f32::NAN, 1.0)`.
///
/// **Panics** in debug mode if `!(min == min)`. (This occurs if `min` is `NAN`.)
#[inline]
#[allow(clippy::eq_op)]
pub c0nst fn clamp_min<T: [c0nst] PartialOrd + [c0nst] Destruct>(input: T, min: T) -> T {
    debug_assert!(min == min, "min must not be NAN");
    if input < min {
        min
    } else {
        input
    }
}
}

c0nst::c0nst! {
/// A value bounded by a maximum value
///
///  If input is greater than max then this returns max.
///  Otherwise this returns input.
///  `clamp_max(std::f32::NAN, 1.0)` preserves `NAN` different from `f32::max(std::f32::NAN, 1.0)`.
///
/// **Panics** in debug mode if `!(max == max)`. (This occurs if `max` is `NAN`.)
#[inline]
#[allow(clippy::eq_op)]
pub c0nst fn clamp_max<T: [c0nst] PartialOrd + [c0nst] Destruct>(input: T, max: T) -> T {
    debug_assert!(max == max, "max must not be NAN");
    if input > max {
        max
    } else {
        input
    }
}
}

#[test]
fn clamp_test() {
    // Int test
    assert_eq!(1, clamp(1, -1, 2));
    assert_eq!(-1, clamp(-2, -1, 2));
    assert_eq!(2, clamp(3, -1, 2));
    assert_eq!(1, clamp_min(1, -1));
    assert_eq!(-1, clamp_min(-2, -1));
    assert_eq!(-1, clamp_max(1, -1));
    assert_eq!(-2, clamp_max(-2, -1));

    // Float test
    assert_eq!(1.0, clamp(1.0, -1.0, 2.0));
    assert_eq!(-1.0, clamp(-2.0, -1.0, 2.0));
    assert_eq!(2.0, clamp(3.0, -1.0, 2.0));
    assert_eq!(1.0, clamp_min(1.0, -1.0));
    assert_eq!(-1.0, clamp_min(-2.0, -1.0));
    assert_eq!(-1.0, clamp_max(1.0, -1.0));
    assert_eq!(-2.0, clamp_max(-2.0, -1.0));
    assert!(clamp(f32::NAN, -1.0, 1.0).is_nan());
    assert!(clamp_min(f32::NAN, 1.0).is_nan());
    assert!(clamp_max(f32::NAN, 1.0).is_nan());
}

#[test]
#[should_panic]
#[cfg(debug_assertions)]
fn clamp_nan_min() {
    clamp(0., f32::NAN, 1.);
}

#[test]
#[should_panic]
#[cfg(debug_assertions)]
fn clamp_nan_max() {
    clamp(0., -1., f32::NAN);
}

#[test]
#[should_panic]
#[cfg(debug_assertions)]
fn clamp_nan_min_max() {
    clamp(0., f32::NAN, f32::NAN);
}

#[test]
#[should_panic]
#[cfg(debug_assertions)]
fn clamp_min_nan_min() {
    clamp_min(0., f32::NAN);
}

#[test]
#[should_panic]
#[cfg(debug_assertions)]
fn clamp_max_nan_max() {
    clamp_max(0., f32::NAN);
}

#[test]
fn from_str_radix_unwrap() {
    // The Result error must impl Debug to allow unwrap()

    let i: i32 = Num::from_str_radix("0", 10).unwrap();
    assert_eq!(i, 0);

    let f: f32 = Num::from_str_radix("0.0", 10).unwrap();
    assert_eq!(f, 0.0);
}

#[test]
fn from_str_radix_multi_byte_fail() {
    // Ensure parsing doesn't panic, even on invalid sign characters
    assert!(<f32 as Num>::from_str_radix("™0.2", 10).is_err());

    // Even when parsing the exponent sign
    assert!(<f32 as Num>::from_str_radix("0.2E™1", 10).is_err());
}

#[test]
fn from_str_radix_ignore_case() {
    assert_eq!(
        <f32 as Num>::from_str_radix("InF", 16).unwrap(),
        f32::INFINITY
    );
    assert_eq!(
        <f32 as Num>::from_str_radix("InfinitY", 16).unwrap(),
        f32::INFINITY
    );
    assert_eq!(
        <f32 as Num>::from_str_radix("-InF", 8).unwrap(),
        f32::NEG_INFINITY
    );
    assert_eq!(
        <f32 as Num>::from_str_radix("-InfinitY", 8).unwrap(),
        f32::NEG_INFINITY
    );
    assert!(<f32 as Num>::from_str_radix("nAn", 4).unwrap().is_nan());
    assert!(<f32 as Num>::from_str_radix("-nAn", 4).unwrap().is_nan());
}

#[test]
fn wrapping_is_num() {
    fn require_num<T: Num>(_: &T) {}
    require_num(&Wrapping(42_u32));
    require_num(&Wrapping(-42));
}

#[test]
fn wrapping_from_str_radix() {
    macro_rules! test_wrapping_from_str_radix {
        ($($t:ty)+) => {
            $(
                for &(s, r) in &[("42", 10), ("42", 2), ("-13.0", 10), ("foo", 10)] {
                    let w = <Wrapping<$t> as Num>::from_str_radix(s, r).map(|w| w.0);
                    assert_eq!(w, <$t as Num>::from_str_radix(s, r));
                }
            )+
        };
    }

    test_wrapping_from_str_radix!(usize u8 u16 u32 u64 isize i8 i16 i32 i64);
}

#[test]
fn saturating_is_num() {
    fn require_num<T: Num>(_: &T) {}
    require_num(&core::num::Saturating(42_u32));
    require_num(&core::num::Saturating(-42));
}

#[test]
fn saturating_from_str_radix() {
    macro_rules! test_saturating_from_str_radix {
        ($($t:ty)+) => {
            $(
                for &(s, r) in &[("42", 10), ("42", 2), ("-13.0", 10), ("foo", 10)] {
                    let w = <core::num::Saturating<$t> as Num>::from_str_radix(s, r).map(|w| w.0);
                    assert_eq!(w, <$t as Num>::from_str_radix(s, r));
                }
            )+
        };
    }

    test_saturating_from_str_radix!(usize u8 u16 u32 u64 isize i8 i16 i32 i64);
}

#[test]
fn check_num_ops() {
    fn compute<T: Num + Copy>(x: T, y: T) -> T {
        x * y / y % y + y - y
    }
    assert_eq!(compute(1, 2), 1)
}

#[test]
fn check_numref_ops() {
    fn compute<T: NumRef>(x: T, y: &T) -> T {
        x * y / y % y + y - y
    }
    assert_eq!(compute(1, &2), 1)
}

#[test]
fn check_refnum_ops() {
    fn compute<T: Copy>(x: &T, y: T) -> T
    where
        for<'a> &'a T: RefNum<T>,
    {
        &(&(&(&(x * y) / y) % y) + y) - y
    }
    assert_eq!(compute(&1, 2), 1)
}

#[test]
fn check_refref_ops() {
    fn compute<T>(x: &T, y: &T) -> T
    where
        for<'a> &'a T: RefNum<T>,
    {
        &(&(&(&(x * y) / y) % y) + y) - y
    }
    assert_eq!(compute(&1, &2), 1)
}

#[test]
fn check_numassign_ops() {
    fn compute<T: NumAssign + Copy>(mut x: T, y: T) -> T {
        x *= y;
        x /= y;
        x %= y;
        x += y;
        x -= y;
        x
    }
    assert_eq!(compute(1, 2), 1)
}

#[test]
fn check_numassignref_ops() {
    fn compute<T: NumAssignRef + Copy>(mut x: T, y: &T) -> T {
        x *= y;
        x /= y;
        x %= y;
        x += y;
        x -= y;
        x
    }
    assert_eq!(compute(1, &2), 1)
}
