//! Float atoms outside the big verbatim-upstream `Float`/`FloatCore`
//! bundles: bit-pattern access, ULP stepping, IEEE 754-2019 min/max,
//! round-half-to-even, the `algebraic_*` fast-math family and the
//! libm-backed special functions.
//!
//! This is the first slice of the planned `Float` decomposition:
//! everything here that touches only bits and comparisons is a
//! `c0nst` trait with const impls on nightly — unlike the transcendental
//! bundle, which waits on std/libm.
//!
//! **CT tiers**: [`FloatBits`], [`NextUp`]/[`NextDown`] and [`Algebraic`]
//! are Tier A; [`Minimum`]/[`Maximum`] are Tier B-ish (comparison ladders,
//! but on floats CT rarely applies); [`RoundTiesEven`], [`Erf`] and
//! [`Gamma`] are Tier C (data-dependent branching / libm).

c0nst::c0nst! {
/// Raw IEEE 754 bit-pattern access.
pub c0nst trait FloatBits: Sized {
    /// The unsigned integer type with the same width (`u32` for `f32`,
    /// `u64` for `f64`).
    type Bits;

    /// Raw transmutation to the bit representation.
    ///
    /// ```
    /// use const_num_traits::FloatBits;
    ///
    /// assert_eq!(FloatBits::to_bits(1.0f32), 0x3F80_0000);
    /// ```
    fn to_bits(self) -> Self::Bits;

    /// Raw transmutation from the bit representation.
    ///
    /// ```
    /// use const_num_traits::FloatBits;
    ///
    /// let f: f32 = FloatBits::from_bits(0x4148_0000u32);
    /// assert_eq!(f, 12.5);
    /// ```
    fn from_bits(bits: Self::Bits) -> Self;
}
}

c0nst::c0nst! {
/// Steps one ULP towards positive infinity.
pub c0nst trait NextUp: Sized {
    /// Returns the least number greater than `self` (one ULP up). Follows
    /// the inherent `next_up` semantics for zeros, infinities and NaN.
    type Output;
    fn next_up(self) -> Self::Output;
}
}

c0nst::c0nst! {
/// Steps one ULP towards negative infinity.
pub c0nst trait NextDown: Sized {
    /// Returns the greatest number less than `self` (one ULP down). Follows
    /// the inherent `next_down` semantics for zeros, infinities and NaN.
    type Output;
    fn next_down(self) -> Self::Output;
}
}

c0nst::c0nst! {
/// IEEE 754-2019 `maximum` (NaN-propagating, unlike `max`).
pub c0nst trait Maximum: Sized {
    /// Returns the greater of two numbers, propagating NaN and treating
    /// `+0.0` as greater than `-0.0` — the IEEE 754-2019 `maximum`
    /// operation, in contrast to `max` which *ignores* NaN.
    ///
    /// ```
    /// use const_num_traits::Maximum;
    ///
    /// assert_eq!(Maximum::maximum(1.0f32, 2.0), 2.0);
    /// assert!(Maximum::maximum(1.0f32, f32::NAN).is_nan());
    /// ```
    type Output;
    fn maximum(self, other: Self) -> Self::Output;
}
}

c0nst::c0nst! {
/// IEEE 754-2019 `minimum` (NaN-propagating, unlike `min`).
pub c0nst trait Minimum: Sized {
    /// Returns the lesser of two numbers, propagating NaN and treating
    /// `-0.0` as less than `+0.0` — the IEEE 754-2019 `minimum` operation,
    /// in contrast to `min` which *ignores* NaN.
    type Output;
    fn minimum(self, other: Self) -> Self::Output;
}
}

/// Rounds half-way cases to the nearest even integer (banker's rounding).
pub trait RoundTiesEven: Sized {
    /// Returns the nearest integer to `self`, with half-way cases rounded
    /// to the even one (`2.5 -> 2.0`, `3.5 -> 4.0`).
    ///
    /// ```
    /// use const_num_traits::RoundTiesEven;
    ///
    /// assert_eq!(RoundTiesEven::round_ties_even(2.5f32), 2.0);
    /// assert_eq!(RoundTiesEven::round_ties_even(3.5f32), 4.0);
    /// assert_eq!(RoundTiesEven::round_ties_even(-2.5f32), -2.0);
    /// ```
    type Output;
    fn round_ties_even(self) -> Self::Output;
}

c0nst::c0nst! {
/// Arithmetic with an "algebraic" license: the compiler may reassociate,
/// use reciprocal shortcuts, or contract into fused operations.
///
/// std's `algebraic_*` methods (unstable `float_algebraic`) allow any
/// result reachable by real-number-algebra rewrites. The plain IEEE
/// operation is one such result, so the primitive impls here simply perform
/// it — a conforming implementation that loses only the optimization
/// license until std's intrinsics stabilize and the impls can delegate.
pub c0nst trait Algebraic: Sized {
    /// The (owned) result type.
    type Output;
    /// Addition with algebraic rewrite license.
    fn algebraic_add(self, rhs: Self) -> Self::Output;
    /// Subtraction with algebraic rewrite license.
    fn algebraic_sub(self, rhs: Self) -> Self::Output;
    /// Multiplication with algebraic rewrite license.
    fn algebraic_mul(self, rhs: Self) -> Self::Output;
    /// Division with algebraic rewrite license.
    fn algebraic_div(self, rhs: Self) -> Self::Output;
    /// Remainder with algebraic rewrite license.
    fn algebraic_rem(self, rhs: Self) -> Self::Output;
}
}

/// The error function and its complement.
///
/// Implementations for `f32`/`f64` require the `libm` cargo feature (std's
/// `erf` is still unstable, so there is nothing to delegate to otherwise).
pub trait Erf: Sized {
    /// The (owned) result type.
    type Output;
    /// The error function `erf(self)`.
    fn erf(self) -> Self::Output;
    /// The complementary error function `1 - erf(self)`.
    fn erfc(self) -> Self::Output;
}

/// The gamma function and the natural log of its absolute value.
///
/// Implementations for `f32`/`f64` require the `libm` cargo feature (std's
/// `gamma` is still unstable, so there is nothing to delegate to
/// otherwise).
pub trait Gamma: Sized {
    /// The (owned) result type.
    type Output;
    /// The gamma function `Γ(self)`.
    fn gamma(self) -> Self::Output;
    /// Returns `ln(|Γ(self)|)` together with the sign of `Γ(self)`,
    /// matching std's `ln_gamma` (and C's `lgamma_r`).
    fn ln_gamma(self) -> (Self::Output, i32);
}

macro_rules! float_bits_impl {
    ($($t:ty => $b:ty;)*) => {$(
        c0nst::c0nst! {
        c0nst impl FloatBits for $t {
            type Bits = $b;

            #[inline]
            fn to_bits(self) -> $b {
                <$t>::to_bits(self)
            }

            #[inline]
            fn from_bits(bits: $b) -> $t {
                <$t>::from_bits(bits)
            }
        }
        }

        c0nst::c0nst! {
        c0nst impl NextUp for $t {
            type Output = $t;
            #[inline]
            fn next_up(self) -> $t {
                <$t>::next_up(self)
            }
        }
        }

        c0nst::c0nst! {
        c0nst impl NextDown for $t {
            type Output = $t;
            #[inline]
            fn next_down(self) -> $t {
                <$t>::next_down(self)
            }
        }
        }

        // minimum/maximum are still unstable in std; same comparison ladder
        // as core, const-compatible (float comparison is const-stable)
        c0nst::c0nst! {
        c0nst impl Maximum for $t {
            type Output = $t;
            #[inline]
            fn maximum(self, other: Self) -> $t {
                if self > other {
                    self
                } else if other > self {
                    other
                } else if self == other {
                    if <$t>::is_sign_positive(self) && <$t>::is_sign_negative(other) {
                        self
                    } else {
                        other
                    }
                } else {
                    // at least one input is NaN; propagate it
                    self + other
                }
            }
        }
        }

        c0nst::c0nst! {
        c0nst impl Minimum for $t {
            type Output = $t;
            #[inline]
            fn minimum(self, other: Self) -> $t {
                if self < other {
                    self
                } else if other < self {
                    other
                } else if self == other {
                    if <$t>::is_sign_negative(self) && <$t>::is_sign_positive(other) {
                        self
                    } else {
                        other
                    }
                } else {
                    self + other
                }
            }
        }
        }

        c0nst::c0nst! {
        c0nst impl Algebraic for $t {
            type Output = $t;
            #[inline]
            fn algebraic_add(self, rhs: Self) -> $t { self + rhs }
            #[inline]
            fn algebraic_sub(self, rhs: Self) -> $t { self - rhs }
            #[inline]
            fn algebraic_mul(self, rhs: Self) -> $t { self * rhs }
            #[inline]
            fn algebraic_div(self, rhs: Self) -> $t { self / rhs }
            #[inline]
            fn algebraic_rem(self, rhs: Self) -> $t { self % rhs }
        }
        }
    )*};
}

float_bits_impl! {
    f32 => u32;
    f64 => u64;
}

// std's round_ties_even is std-only (libm-backed); the no-std fallback
// hand-rolls banker's rounding from FloatCore primitives.
macro_rules! round_ties_even_impl {
    ($($t:ty)*) => {$(
        #[cfg(feature = "std")]
        impl RoundTiesEven for $t {
            type Output = $t;
            #[inline]
            fn round_ties_even(self) -> $t {
                <$t>::round_ties_even(self)
            }
        }

        #[cfg(not(feature = "std"))]
        impl RoundTiesEven for $t {
            type Output = $t;
            #[inline]
            fn round_ties_even(self) -> $t {
                use crate::float::FloatCore;
                let f = FloatCore::fract(self);
                if FloatCore::abs(f) != 0.5 {
                    // no tie: ordinary round-half-away agrees
                    FloatCore::round(self)
                } else {
                    let t = FloatCore::trunc(self);
                    if t % 2.0 == 0.0 {
                        t
                    } else {
                        t + FloatCore::signum(self)
                    }
                }
            }
        }
    )*};
}

round_ties_even_impl!(f32 f64);

#[cfg(feature = "libm")]
impl Erf for f32 {
    type Output = f32;
    #[inline]
    fn erf(self) -> f32 {
        libm::erff(self)
    }
    #[inline]
    fn erfc(self) -> f32 {
        libm::erfcf(self)
    }
}

#[cfg(feature = "libm")]
impl Erf for f64 {
    type Output = f64;
    #[inline]
    fn erf(self) -> f64 {
        libm::erf(self)
    }
    #[inline]
    fn erfc(self) -> f64 {
        libm::erfc(self)
    }
}

#[cfg(feature = "libm")]
impl Gamma for f32 {
    type Output = f32;
    #[inline]
    fn gamma(self) -> f32 {
        libm::tgammaf(self)
    }
    #[inline]
    fn ln_gamma(self) -> (f32, i32) {
        libm::lgammaf_r(self)
    }
}

#[cfg(feature = "libm")]
impl Gamma for f64 {
    type Output = f64;
    #[inline]
    fn gamma(self) -> f64 {
        libm::tgamma(self)
    }
    #[inline]
    fn ln_gamma(self) -> (f64, i32) {
        libm::lgamma_r(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bits_and_ulps() {
        assert_eq!(FloatBits::to_bits(1.0f32), 0x3F80_0000u32);
        let x: f64 = FloatBits::from_bits(0x4029_0000_0000_0000u64);
        assert_eq!(x, 12.5);
        assert_eq!(NextUp::next_up(1.0f32), f32::from_bits(0x3F80_0001));
        assert_eq!(NextDown::next_down(1.0f32), f32::from_bits(0x3F7F_FFFF));
        assert_eq!(NextUp::next_up(0.0f32), f32::from_bits(1)); // smallest subnormal
    }

    #[test]
    fn minimum_maximum() {
        assert_eq!(Maximum::maximum(1.0f32, 2.0), 2.0);
        assert_eq!(Minimum::minimum(1.0f32, 2.0), 1.0);
        // NaN propagates (unlike max/min)
        assert!(Maximum::maximum(1.0f32, f32::NAN).is_nan());
        assert!(Minimum::minimum(f32::NAN, 1.0f32).is_nan());
        // signed zeros are ordered
        assert_eq!(Maximum::maximum(0.0f32, -0.0).to_bits(), 0.0f32.to_bits());
        assert_eq!(Minimum::minimum(0.0f32, -0.0).to_bits(), (-0.0f32).to_bits());
    }

    #[test]
    fn ties_even() {
        assert_eq!(RoundTiesEven::round_ties_even(2.5f32), 2.0);
        assert_eq!(RoundTiesEven::round_ties_even(3.5f32), 4.0);
        assert_eq!(RoundTiesEven::round_ties_even(-2.5f64), -2.0);
        assert_eq!(RoundTiesEven::round_ties_even(-3.5f64), -4.0);
        assert_eq!(RoundTiesEven::round_ties_even(2.4f32), 2.0);
        assert_eq!(RoundTiesEven::round_ties_even(2.6f32), 3.0);
        assert_eq!(RoundTiesEven::round_ties_even(0.5f64), 0.0);
        assert_eq!(RoundTiesEven::round_ties_even(-0.5f64), -0.0);
    }

    #[test]
    fn algebraic_is_conforming() {
        assert_eq!(Algebraic::algebraic_add(1.5f64, 2.25), 3.75);
        assert_eq!(Algebraic::algebraic_mul(3.0f32, 0.5), 1.5);
        assert_eq!(Algebraic::algebraic_rem(7.5f64, 2.0), 1.5);
    }

    #[cfg(feature = "libm")]
    #[test]
    fn special_functions() {
        assert!((Erf::erf(0.0f64)).abs() < 1e-15);
        assert!((Erf::erf(10.0f64) - 1.0).abs() < 1e-15);
        assert!((Erf::erfc(0.0f64) - 1.0).abs() < 1e-15);
        assert!((Gamma::gamma(5.0f64) - 24.0).abs() < 1e-10);
        let (lg, sign) = Gamma::ln_gamma(5.0f64);
        assert!((lg - 24.0f64.ln()).abs() < 1e-10);
        assert_eq!(sign, 1);
    }
}
