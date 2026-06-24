//! Zero-cost typestate proofs.
//!
//! A *typestate* is a zero-runtime-cost proof that a value satisfies a
//! structural property, built once by a checked constructor and *spent* by an
//! op that exploits it to delete a branch, an `Option`, or a division. Every
//! type here ships at least one such consuming op. Pure-`core` and always
//! available — it costs nothing unless its types are named.
//!
//! Resident families:
//! - [`PowerOfTwo`] + [`PowerOfTwoOps`] (unsigned): div/rem/align as shifts/masks.
//! - [`BitIndex`] + [`BitIndexOps`] (all ints): shift by an amount proven `< BITS`.
//! - [`HasNonZero`] bridge to [`core::num::NonZero`] + [`DivNonZero`] (unsigned):
//!   infallible division.
//! - [`NonNegative`] / [`Positive`] (signed): unsigned cast, `abs`, `isqrt`, plus
//!   `const` narrowings into each other / `NonZero` / `NonMin`.
//! - [`NonMin`] (signed): total `neg`/`abs` and total signed division.
//! - [`Odd`] / [`Even`]: bare proofs (consumer lives in the modmath layer).
//! - [`Finite`] (floats): a total order (`Ord`/`Eq`) that bare floats lack.
//!
//! With the `ct` feature, families with a *secret-derived* predicate also gain a
//! masked `new_ct`. [`BitIndex`] (public shift amounts) and [`Finite`] (floats
//! are outside the CT model) have none.

use crate::int::PrimBits;
use crate::ops::parity::Parity;
use crate::ops::pow2::IsPowerOfTwo;
use core::marker::PhantomData;

/// Error returned by the `TryFrom` constructors of the typestate proofs when
/// the value fails the proof's predicate (e.g. a non-odd value into [`Odd`]).
///
/// A single shared, zero-sized error: the target type at the call site already
/// names which predicate failed. The fallible *reference* narrowing keeps using
/// the lighter inherent `from_ref` (returning `Option`) — `core` has no trait
/// for fallible reference conversion, so there is nothing to mirror there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypestateError;

impl core::fmt::Display for TypestateError {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("value does not satisfy the typestate's predicate")
    }
}

// `core::error::Error` is available in `no_std` since Rust 1.81 (< our MSRV).
impl core::error::Error for TypestateError {}

/// Proof that a value is a power of two (`2^k`, `k ≥ 0`), for an unsigned
/// integer type `T`.
///
/// **Representation is the exponent `k`, not the value** — so the consuming
/// operations in [`PowerOfTwoOps`] are pure shifts and masks with nothing
/// recomputed per call, and a (future) big-integer backend carries a tiny
/// `u32` proof regardless of its width. The field is private, so the
/// representation is an implementation detail. Consequently `PowerOfTwo` does
/// **not** deref to `T`; use [`get`](Self::get) (which reconstructs `1 << k`)
/// or [`exp`](Self::exp).
///
/// `PowerOfTwo<T>` is always `Copy` (it stores only a `u32`), independent of
/// whether `T` is.
///
/// # Examples
///
/// ```
/// use const_num_traits::{PowerOfTwo, PowerOfTwoOps};
///
/// let p = PowerOfTwo::<u32>::new(16).unwrap();
/// assert_eq!(p.exp(), 4);
/// assert_eq!(p.get(), 16);
/// assert_eq!(100u32.div_pow2(p), 6); // 100 / 16
/// assert_eq!(100u32.rem_pow2(p), 4); // 100 % 16
/// assert!(PowerOfTwo::<u32>::new(6).is_none());
/// ```
pub struct PowerOfTwo<T> {
    // Invariant: `1 << exp` is a valid power of two of `T` (`exp < T::BITS`).
    exp: u32,
    _t: PhantomData<T>,
}

impl<T> Clone for PowerOfTwo<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for PowerOfTwo<T> {}

impl<T> PowerOfTwo<T> {
    /// Constructs the proof directly from an exponent, without checking.
    ///
    /// # Safety
    ///
    /// `exp` must be `< T::BITS` (so that `1 << exp` is a valid power of two of
    /// `T`). Passing a too-large exponent makes the consuming ops produce
    /// nonsense or overflow.
    #[inline]
    pub const unsafe fn from_exp_unchecked(exp: u32) -> Self {
        PowerOfTwo {
            exp,
            _t: PhantomData,
        }
    }

    /// The exponent `k` such that the proven value is `2^k`. Free (no recompute).
    #[inline]
    pub const fn exp(self) -> u32 {
        self.exp
    }
}

c0nst::c0nst! {
impl<T> PowerOfTwo<T> {
    /// Safe constructor for any carrier, not just the primitives: `Some` iff
    /// `value` is a power of two. Keeps the `unsafe` crate-internal; the
    /// per-primitive [`new`](Self::new) stays the `const` fast path.
    #[inline]
    pub c0nst fn new_checked(value: T) -> Option<Self>
    where
        T: [c0nst] IsPowerOfTwo + [c0nst] PrimBits,
    {
        if value.is_power_of_two() {
            let width = T::ZERO.count_zeros();
            let exp = width - 1 - value.leading_zeros();
            // SAFETY: `value` is a power of two, so `exp < width` (= `T::BITS`).
            Some(unsafe { Self::from_exp_unchecked(exp) })
        } else {
            None
        }
    }
}
}

c0nst::c0nst! {
/// Operations that *consume* a [`PowerOfTwo`] proof to replace division and
/// remainder with a shift and a mask. These are **new** methods, not a
/// speed-up of the blanket `MultipleOf`/`NextMultipleOf` (those can't be
/// specialised on a power-of-two divisor on stable Rust).
///
/// Owned results carry an associated [`Output`](Self::Output) so non-`Copy`
/// types can implement the trait for their reference type.
pub c0nst trait PowerOfTwoOps: Sized {
    /// The owned result type (`Self` for the primitive impls).
    type Output;

    /// `self / p`, computed as `self >> p.exp()`.
    fn div_pow2(self, p: PowerOfTwo<Self>) -> Self::Output;

    /// `self % p`, computed as `self & ((1 << p.exp()) - 1)`.
    fn rem_pow2(self, p: PowerOfTwo<Self>) -> Self::Output;

    /// `self % p == 0`, computed as a mask test (no division).
    fn is_multiple_of_pow2(self, p: PowerOfTwo<Self>) -> bool;

    /// Smallest multiple of `p` that is `>= self` ("align up"), computed
    /// branch-free as `(self + mask) & !mask`. Uses `+`, so it **panics on
    /// overflow in debug** (and the const-eval form errors) when `self` is
    /// within `p` of the maximum — use
    /// [`checked_next_multiple_of_pow2`](Self::checked_next_multiple_of_pow2)
    /// for untrusted input.
    fn next_multiple_of_pow2(self, p: PowerOfTwo<Self>) -> Self::Output;

    /// [`next_multiple_of_pow2`](Self::next_multiple_of_pow2), returning `None`
    /// on overflow instead of panicking.
    fn checked_next_multiple_of_pow2(self, p: PowerOfTwo<Self>) -> Option<Self::Output>;
}
}

macro_rules! pow2_typestate_impl {
    ($($t:ty),+) => {$(
        impl PowerOfTwo<$t> {
            /// Checked constructor: `Some` iff `value` is a power of two.
            ///
            /// `const fn` on stable and nightly (delegates to the inherent
            /// `is_power_of_two`/`ilog2`, both const since ≤ 1.67).
            #[inline]
            pub const fn new(value: $t) -> Option<Self> {
                if value.is_power_of_two() {
                    Some(PowerOfTwo { exp: value.ilog2(), _t: PhantomData })
                } else {
                    None
                }
            }

            /// Reconstructs the proven value, `1 << exp`.
            #[inline]
            pub const fn get(self) -> $t {
                (1 as $t) << self.exp
            }
        }

        c0nst::c0nst! {
        c0nst impl PowerOfTwoOps for $t {
            type Output = $t;

            #[inline]
            fn div_pow2(self, p: PowerOfTwo<$t>) -> $t {
                self >> p.exp
            }

            #[inline]
            fn rem_pow2(self, p: PowerOfTwo<$t>) -> $t {
                let mask = ((1 as $t) << p.exp) - 1;
                self & mask
            }

            #[inline]
            fn is_multiple_of_pow2(self, p: PowerOfTwo<$t>) -> bool {
                let mask = ((1 as $t) << p.exp) - 1;
                (self & mask) == 0
            }

            #[inline]
            fn next_multiple_of_pow2(self, p: PowerOfTwo<$t>) -> $t {
                let mask = ((1 as $t) << p.exp) - 1;
                (self + mask) & !mask
            }

            #[inline]
            fn checked_next_multiple_of_pow2(self, p: PowerOfTwo<$t>) -> Option<$t> {
                let mask = ((1 as $t) << p.exp) - 1;
                match self.checked_add(mask) {
                    Some(s) => Some(s & !mask),
                    None => None,
                }
            }
        }
        }

        /// Checked construction by value; mirrors [`PowerOfTwo::new`]. (The
        /// generic carrier path stays [`PowerOfTwo::new_checked`].)
        impl TryFrom<$t> for PowerOfTwo<$t> {
            type Error = TypestateError;
            #[inline]
            fn try_from(value: $t) -> Result<Self, TypestateError> {
                Self::new(value).ok_or(TypestateError)
            }
        }
    )+};
}

pow2_typestate_impl!(u8, u16, u32, u64, u128, usize);

// ─────────────────────────────── BitIndex (shift amount) ──────────────────

/// Proof that a value is a valid bit index for `T` (`0 <= index < T::BITS`), so
/// a shift by it can't overflow, panic, or hit the "shift `>= BITS`" UB.
///
/// Like [`PowerOfTwo`], the rep is the `u32` index (never `T`), so the proof is
/// `Copy` regardless of `T`; consuming ops live in [`BitIndexOps`] (signed and
/// unsigned). No constant-time constructor — shift amounts are public.
///
/// ```
/// use const_num_traits::{BitIndex, BitIndexOps};
///
/// let i = BitIndex::<u8>::new(3).unwrap();
/// assert_eq!(1u8.shl_index(i), 8);
/// assert_eq!(0x80u8.shr_index(i), 0x10);
/// assert!(BitIndex::<u8>::new(8).is_none()); // == BITS, rejected
/// ```
pub struct BitIndex<T> {
    // Invariant: `index < T::BITS`.
    index: u32,
    _t: PhantomData<T>,
}

impl<T> Clone for BitIndex<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for BitIndex<T> {}

impl<T> BitIndex<T> {
    /// Constructs the proof directly from an index, without checking.
    ///
    /// # Safety
    ///
    /// `index` must be `< T::BITS`; otherwise the consuming shifts are
    /// undefined / panic.
    #[inline]
    pub const unsafe fn from_u32_unchecked(index: u32) -> Self {
        BitIndex {
            index,
            _t: PhantomData,
        }
    }

    /// The proven index. Free (no recompute).
    #[inline]
    pub const fn get(self) -> u32 {
        self.index
    }
}

c0nst::c0nst! {
impl<T> BitIndex<T> {
    /// Safe constructor for any [`PrimBits`] carrier, not just the primitives:
    /// `Some` iff `index < T::BITS`. The per-primitive [`new`](Self::new) stays
    /// the `const` fast path.
    #[inline]
    pub c0nst fn new_checked(index: u32) -> Option<Self>
    where
        T: [c0nst] PrimBits,
    {
        if index < T::ZERO.count_zeros() {
            Some(BitIndex { index, _t: PhantomData })
        } else {
            None
        }
    }
}
}

c0nst::c0nst! {
/// Operations that *consume* a [`BitIndex`] proof to shift without the
/// overflow-check branch: the amount is proven `< BITS`, so there is no
/// debug-time panic and no `unbounded_*` masking.
///
/// Owned results carry an associated [`Output`](Self::Output) so non-`Copy`
/// types can implement the trait for their reference type.
pub c0nst trait BitIndexOps: Sized {
    /// The owned result type (`Self` for the primitive impls).
    type Output;

    /// `self << index`, total because `index < BITS`.
    fn shl_index(self, index: BitIndex<Self>) -> Self::Output;

    /// `self >> index`, total because `index < BITS`.
    fn shr_index(self, index: BitIndex<Self>) -> Self::Output;
}
}

macro_rules! bit_index_impl {
    ($($t:ty),+) => {$(
        impl BitIndex<$t> {
            /// Checked constructor: `Some` iff `index < <$t>::BITS`.
            #[inline]
            pub const fn new(index: u32) -> Option<Self> {
                if index < <$t>::BITS {
                    Some(BitIndex { index, _t: PhantomData })
                } else {
                    None
                }
            }
        }

        c0nst::c0nst! {
        c0nst impl BitIndexOps for $t {
            type Output = $t;

            #[inline]
            fn shl_index(self, index: BitIndex<$t>) -> $t {
                self << index.index
            }

            #[inline]
            fn shr_index(self, index: BitIndex<$t>) -> $t {
                self >> index.index
            }
        }
        }

        /// Checked construction from an index (`< BITS`); mirrors
        /// [`BitIndex::new`]. The source is the `u32` index, not the carrier `$t`.
        impl TryFrom<u32> for BitIndex<$t> {
            type Error = TypestateError;
            #[inline]
            fn try_from(index: u32) -> Result<Self, TypestateError> {
                Self::new(index).ok_or(TypestateError)
            }
        }
    )+};
}

bit_index_impl!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
);

// ─────────────────────────────── NonZero bridge ───────────────────────────

c0nst::c0nst! {
/// Bridge to [`core::num::NonZero`] — "the non-zero form of `Self`".
///
/// This does **not** duplicate `core::num::NonZero`: for the integer primitives
/// [`NonZero`](Self::NonZero) *is* `core::num::NonZero<Self>`, with its real
/// niche. (`core::num::NonZero<T>` is sealed to primitives, so a future bignum
/// backend supplies its own non-zero type as the associated type instead.)
///
/// The value is recovered through the crate-owned const accessor
/// [`nonzero_get`](Self::nonzero_get) rather than an `Into<Self>` bound:
/// `From<NonZero<T>> for T` is not `const`, so such a bound would poison every
/// generic `const` consumer.
pub c0nst trait HasNonZero: Sized {
    /// The non-zero form of `Self` (`core::num::NonZero<Self>` for primitives).
    type NonZero: Copy;

    /// `Some` iff `self != 0`.
    fn into_nonzero(self) -> Option<Self::NonZero>;

    /// Recover the underlying value from its non-zero form.
    fn nonzero_get(nz: Self::NonZero) -> Self;
}
}

c0nst::c0nst! {
/// Infallible division / remainder by a proven-non-zero divisor.
///
/// `div_nonzero` / `rem_nonzero` have no divide-by-zero branch and never return
/// `Option`. **Honest note:** for the *primitives* the codegen win is ≈ 0 —
/// LLVM already elides the check via `NonZero`'s niche. The real wins are the
/// `Option`-free API and big-integer backends' hand-written no-branch division.
///
/// **Unsigned only.** Signed `MIN / -1` still overflows, so the total signed
/// form lives on [`NonMin::div_nonzero`] / [`NonMin::rem_nonzero`] (the dividend
/// proven `!= MIN`), not here.
pub c0nst trait DivNonZero: [c0nst] HasNonZero {
    /// Owned result. A fresh `Output` — the divisor is `Self::NonZero`, not
    /// `Self`, so `Div::Output` cannot be reused.
    type Output;

    /// `self / d`, infallibly.
    fn div_nonzero(self, d: Self::NonZero) -> Self::Output;

    /// `self % d`, infallibly.
    fn rem_nonzero(self, d: Self::NonZero) -> Self::Output;
}
}

macro_rules! nonzero_bridge_impl {
    ($($t:ty),+) => {$(
        c0nst::c0nst! {
        c0nst impl HasNonZero for $t {
            type NonZero = core::num::NonZero<$t>;

            #[inline]
            fn into_nonzero(self) -> Option<core::num::NonZero<$t>> {
                core::num::NonZero::new(self)
            }

            #[inline]
            fn nonzero_get(nz: core::num::NonZero<$t>) -> $t {
                nz.get()
            }
        }
        }

        c0nst::c0nst! {
        c0nst impl DivNonZero for $t {
            type Output = $t;

            #[inline]
            fn div_nonzero(self, d: core::num::NonZero<$t>) -> $t {
                self / d.get()
            }

            #[inline]
            fn rem_nonzero(self, d: core::num::NonZero<$t>) -> $t {
                self % d.get()
            }
        }
        }
    )+};
}

nonzero_bridge_impl!(u8, u16, u32, u64, u128, usize);

// ─────────────────────────────── sign typestates ──────────────────────────

/// Proof that a signed value is `>= 0`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NonNegative<T>(T);

/// Proof that a signed value is `> 0`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Positive<T>(T);

macro_rules! sign_typestate_impl {
    ($($t:ty => $u:ty),+) => {$(
        impl NonNegative<$t> {
            /// `Some` iff `value >= 0`.
            #[inline]
            pub const fn new(value: $t) -> Option<Self> {
                if value >= 0 { Some(NonNegative(value)) } else { None }
            }
            /// # Safety
            /// `value` must be `>= 0`.
            #[inline]
            pub const unsafe fn new_unchecked(value: $t) -> Self { NonNegative(value) }
            /// The proven value.
            #[inline]
            pub const fn get(self) -> $t { self.0 }
            /// Zero-cost borrowed proof (repr(transparent) reinterpret).
            #[inline]
            pub fn from_ref(value: &$t) -> Option<&Self> {
                if *value >= 0 {
                    Some(unsafe { &*(value as *const $t as *const Self) })
                } else { None }
            }
            /// Infallible cast to the unsigned counterpart — the value is `>= 0`
            /// so the bit pattern is preserved; no `Option`, no panic.
            #[inline]
            pub const fn to_unsigned(self) -> $u { self.0 as $u }
            /// `abs` is the identity on a non-negative value (branch-free).
            #[inline]
            pub const fn abs(self) -> $t { self.0 }
            /// `isqrt` is total on a non-negative value (the inherent signed
            /// `isqrt` would otherwise panic on negatives).
            #[inline]
            pub const fn isqrt(self) -> $t { self.0.isqrt() }
            /// `NonNegative ⊂ NonMin` (`>= 0`, and `MIN < 0`, so `!= MIN`) —
            /// narrows with no recheck so it can feed total `neg`/`abs`/division.
            #[inline]
            pub const fn into_nonmin(self) -> NonMin<$t> { NonMin(self.0) }
        }

        impl Positive<$t> {
            /// `Some` iff `value > 0`.
            #[inline]
            pub const fn new(value: $t) -> Option<Self> {
                if value > 0 { Some(Positive(value)) } else { None }
            }
            /// # Safety
            /// `value` must be `> 0`.
            #[inline]
            pub const unsafe fn new_unchecked(value: $t) -> Self { Positive(value) }
            /// The proven value.
            #[inline]
            pub const fn get(self) -> $t { self.0 }
            /// Zero-cost borrowed proof (repr(transparent) reinterpret).
            #[inline]
            pub fn from_ref(value: &$t) -> Option<&Self> {
                if *value > 0 {
                    Some(unsafe { &*(value as *const $t as *const Self) })
                } else { None }
            }
            /// Infallible cast to the unsigned counterpart.
            #[inline]
            pub const fn to_unsigned(self) -> $u { self.0 as $u }
            /// `abs` is the identity on a positive value.
            #[inline]
            pub const fn abs(self) -> $t { self.0 }
            /// `isqrt` is total on a positive value.
            #[inline]
            pub const fn isqrt(self) -> $t { self.0.isqrt() }

            // ── refinement lattice (cheap const-fn narrowings) ──
            /// `Positive ⊂ NonNegative`.
            #[inline]
            pub const fn into_nonnegative(self) -> NonNegative<$t> {
                NonNegative(self.0)
            }
            /// `Positive ⊂ NonZero` — narrows to `core::num::NonZero` with no
            /// recheck, so a `Positive` divisor can feed `DivNonZero` directly.
            #[inline]
            pub const fn into_nonzero(self) -> core::num::NonZero<$t> {
                // SAFETY: value > 0, hence non-zero.
                unsafe { core::num::NonZero::new_unchecked(self.0) }
            }
            /// `Positive ⊂ NonMin` (`> 0`, so `!= MIN`).
            #[inline]
            pub const fn into_nonmin(self) -> NonMin<$t> { NonMin(self.0) }
        }

        /// Checked construction by value; mirrors [`NonNegative::new`].
        impl TryFrom<$t> for NonNegative<$t> {
            type Error = TypestateError;
            #[inline]
            fn try_from(value: $t) -> Result<Self, TypestateError> {
                Self::new(value).ok_or(TypestateError)
            }
        }

        /// Checked construction by value; mirrors [`Positive::new`].
        impl TryFrom<$t> for Positive<$t> {
            type Error = TypestateError;
            #[inline]
            fn try_from(value: $t) -> Result<Self, TypestateError> {
                Self::new(value).ok_or(TypestateError)
            }
        }
    )+};
}

sign_typestate_impl!(
    i8 => u8, i16 => u16, i32 => u32, i64 => u64, i128 => u128, isize => usize
);

// ─────────────────────────────── NonMin (signed) ──────────────────────────

/// Proof that a signed value is **not** `T::MIN`.
///
/// `neg`/`abs` overflow only at `MIN`, and the only signed-division overflow is
/// `MIN / -1`. So with the dividend proven `!= MIN`, [`neg`](Self::neg) /
/// [`abs`](Self::abs) and [`div_nonzero`](Self::div_nonzero) /
/// [`rem_nonzero`](Self::rem_nonzero) by any non-zero divisor are total — the
/// co-proof the unsigned [`DivNonZero`] doesn't need. [`Positive`] /
/// [`NonNegative`] narrow here for free (`into_nonmin`).
///
/// ```
/// use const_num_traits::NonMin;
/// use core::num::NonZero;
///
/// let a = NonMin::<i32>::new(-7).unwrap();
/// assert_eq!(a.abs(), 7);
/// assert_eq!(a.neg(), 7);
/// let d = NonZero::new(2).unwrap();
/// assert_eq!(a.div_nonzero(d), -3); // -7 / 2, truncating
/// assert_eq!(a.rem_nonzero(d), -1); // -7 % 2
/// assert!(NonMin::<i32>::new(i32::MIN).is_none());
/// ```
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NonMin<T>(T);

macro_rules! nonmin_impl {
    ($($t:ty),+) => {$(
        impl NonMin<$t> {
            /// `Some` iff `value != <$t>::MIN`.
            #[inline]
            pub const fn new(value: $t) -> Option<Self> {
                if value != <$t>::MIN { Some(NonMin(value)) } else { None }
            }
            /// # Safety
            /// `value` must not be `<$t>::MIN`.
            #[inline]
            pub const unsafe fn new_unchecked(value: $t) -> Self { NonMin(value) }
            /// The proven value.
            #[inline]
            pub const fn get(self) -> $t { self.0 }
            /// Zero-cost borrowed proof (repr(transparent) reinterpret).
            #[inline]
            pub fn from_ref(value: &$t) -> Option<&Self> {
                if *value != <$t>::MIN {
                    Some(unsafe { &*(value as *const $t as *const Self) })
                } else { None }
            }
            /// Total negation — `MIN` (the sole overflow) is excluded.
            #[inline]
            pub const fn neg(self) -> $t { -self.0 }
            /// Total `abs` — `MIN` (the sole overflow) is excluded.
            #[inline]
            pub const fn abs(self) -> $t { self.0.abs() }
            /// Total signed division by a non-zero divisor: the dividend is
            /// proven `!= MIN`, so `MIN / -1` (the only overflow) can't occur,
            /// and the divisor's niche rules out divide-by-zero.
            #[inline]
            pub const fn div_nonzero(self, d: core::num::NonZero<$t>) -> $t {
                self.0 / d.get()
            }
            /// Total signed remainder by a non-zero divisor (same reasoning as
            /// [`div_nonzero`](Self::div_nonzero); `MIN % -1` is excluded).
            #[inline]
            pub const fn rem_nonzero(self, d: core::num::NonZero<$t>) -> $t {
                self.0 % d.get()
            }
        }

        /// Checked construction by value; mirrors [`NonMin::new`].
        impl TryFrom<$t> for NonMin<$t> {
            type Error = TypestateError;
            #[inline]
            fn try_from(value: $t) -> Result<Self, TypestateError> {
                Self::new(value).ok_or(TypestateError)
            }
        }
    )+};
}

nonmin_impl!(i8, i16, i32, i64, i128, isize);

// ─────────────────────────────────── Odd ──────────────────────────────────

/// Proof that a value is odd.
///
/// A **bare** typestate: it has no consuming op *in this crate*. Its consumer
/// (e.g. an odd-modulus Montgomery constructor) lives in the modular-arithmetic
/// layer, where the precondition is actually spent. It ships here because that
/// is where the predicate ([`Parity`]) lives. `Odd` covers both the "non-zero"
/// and "odd" preconditions a Montgomery modulus needs — zero is even — so no
/// separate `OddNonZero` is required.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Odd<T>(T);

impl<T> Odd<T> {
    /// # Safety
    /// `value` must be odd.
    #[inline]
    pub const unsafe fn new_unchecked(value: T) -> Self {
        Odd(value)
    }

    /// The proven-odd value.
    #[inline]
    pub fn get(self) -> T {
        self.0
    }
}

/// Borrows the proven-odd value without consuming the proof.
impl<T> AsRef<T> for Odd<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        &self.0
    }
}

c0nst::c0nst! {
impl<T: Parity + Copy> Odd<T> {
    /// `Some` iff `value` is odd. `const`-callable on nightly when
    /// `T: [const] Parity`, so a const modulus can be proven odd in a `const`
    /// block (no runtime panic path); plain on stable.
    #[inline]
    pub c0nst fn new(value: T) -> Option<Self>
    where
        T: [c0nst] Parity,
    {
        if value.is_odd() { Some(Odd(value)) } else { None }
    }
}
}

impl<T> Odd<T>
where
    for<'a> &'a T: Parity,
{
    /// Zero-cost borrowed proof — relies on the blanket `impl Parity for &T`
    /// (for `Copy` `T`) or a non-`Copy` type's own `Parity for &Self`.
    #[inline]
    pub fn from_ref(value: &T) -> Option<&Self> {
        if value.is_odd() {
            Some(unsafe { &*(value as *const T as *const Odd<T>) })
        } else {
            None
        }
    }
}

/// Proof that a value is even — the parity sibling of [`Odd`].
///
/// Also a **bare** proof (no consuming op in this crate). The synthesis cut
/// `Even` for lacking a compelling consumer; it is included here for parity
/// symmetry and to round out the masked-constructor set (`Even::new_ct`).
/// Drop it if the scope cut should stand.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Even<T>(T);

impl<T> Even<T> {
    /// # Safety
    /// `value` must be even.
    #[inline]
    pub const unsafe fn new_unchecked(value: T) -> Self {
        Even(value)
    }

    /// The proven-even value.
    #[inline]
    pub fn get(self) -> T {
        self.0
    }
}

/// Borrows the proven-even value without consuming the proof.
impl<T> AsRef<T> for Even<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        &self.0
    }
}

// Checked construction by value — the `core`-idiomatic fallible inverse of
// `From`, mirroring `Odd::new` / `Even::new`. Per-primitive (not generic over
// `Parity`): a generic `impl<T> TryFrom<T> for Odd<T>` collides with `core`'s
// reflexive `TryFrom` blanket. Bignum carriers use the generic `new`/`from_ref`.
macro_rules! parity_try_from {
    ($($t:ty),+) => {$(
        impl TryFrom<$t> for Odd<$t> {
            type Error = TypestateError;
            #[inline]
            fn try_from(value: $t) -> Result<Self, TypestateError> {
                Self::new(value).ok_or(TypestateError)
            }
        }
        impl TryFrom<$t> for Even<$t> {
            type Error = TypestateError;
            #[inline]
            fn try_from(value: $t) -> Result<Self, TypestateError> {
                Self::new(value).ok_or(TypestateError)
            }
        }
    )+};
}
parity_try_from!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
);

c0nst::c0nst! {
impl<T: Parity + Copy> Even<T> {
    /// `Some` iff `value` is even. `const`-callable on nightly when
    /// `T: [const] Parity` (sibling of [`Odd::new`]); plain on stable.
    #[inline]
    pub c0nst fn new(value: T) -> Option<Self>
    where
        T: [c0nst] Parity,
    {
        if value.is_even() { Some(Even(value)) } else { None }
    }
}
}

impl<T> Even<T>
where
    for<'a> &'a T: Parity,
{
    /// Zero-cost borrowed proof.
    #[inline]
    pub fn from_ref(value: &T) -> Option<&Self> {
        if value.is_even() {
            Some(unsafe { &*(value as *const T as *const Even<T>) })
        } else {
            None
        }
    }
}

// ─────────────────────────────── Finite (float) ───────────────────────────

/// Proof that a float is finite (neither `NaN` nor `±∞`).
///
/// Spent by a **total order**: `Finite<T>` implements [`Ord`]/[`Eq`], which bare
/// `f32`/`f64` can't (`NaN` breaks reflexivity and trichotomy). With `NaN` gone,
/// `partial_cmp` is always `Some`, so sorting / `min`/`max` / `BTreeMap` keys are
/// total. `repr(transparent)`; no `new_ct` (floats are outside the CT model).
///
/// ```
/// use const_num_traits::Finite;
///
/// let a = Finite::<f64>::new(1.0).unwrap();
/// let b = Finite::<f64>::new(2.0).unwrap();
/// assert!(a < b);
/// assert_eq!(a.max(b).get(), 2.0);
/// assert!(Finite::<f64>::new(f64::NAN).is_none());
/// assert!(Finite::<f64>::new(f64::INFINITY).is_none());
/// ```
#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct Finite<T>(T);

macro_rules! finite_impl {
    ($($t:ty => $exp_mask:expr),+) => {$(
        impl Finite<$t> {
            /// `Some` iff `value` is finite (not `NaN`, not `±∞`).
            ///
            /// `const fn`: finiteness is a bit test — the exponent field is
            /// *not* all-ones — which sidesteps the non-`const` `is_finite`.
            #[inline]
            pub const fn new(value: $t) -> Option<Self> {
                if (value.to_bits() & $exp_mask) != $exp_mask {
                    Some(Finite(value))
                } else {
                    None
                }
            }
            /// # Safety
            /// `value` must be finite.
            #[inline]
            pub const unsafe fn new_unchecked(value: $t) -> Self { Finite(value) }
            /// The proven-finite value.
            #[inline]
            pub const fn get(self) -> $t { self.0 }
            /// Zero-cost borrowed proof (repr(transparent) reinterpret).
            #[inline]
            pub fn from_ref(value: &$t) -> Option<&Self> {
                if (value.to_bits() & $exp_mask) != $exp_mask {
                    Some(unsafe { &*(value as *const $t as *const Self) })
                } else {
                    None
                }
            }
        }

        /// Checked construction by value; mirrors [`Finite::new`]. The natural
        /// boundary for rejecting `NaN`/`±∞` from external float input with `?`.
        impl TryFrom<$t> for Finite<$t> {
            type Error = TypestateError;
            #[inline]
            fn try_from(value: $t) -> Result<Self, TypestateError> {
                Self::new(value).ok_or(TypestateError)
            }
        }

        impl PartialEq for Finite<$t> {
            #[inline]
            fn eq(&self, other: &Self) -> bool { self.0 == other.0 }
        }
        // With `NaN` excluded, `==` is reflexive, so `Eq` is sound.
        impl Eq for Finite<$t> {}

        impl PartialOrd for Finite<$t> {
            #[inline]
            fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }
        impl Ord for Finite<$t> {
            #[inline]
            fn cmp(&self, other: &Self) -> core::cmp::Ordering {
                // Both operands are finite ⇒ `partial_cmp` is always `Some`;
                // this `unwrap` is the proof being spent and can never panic.
                self.0.partial_cmp(&other.0).unwrap()
            }
        }
    )+};
}

finite_impl!(
    f32 => 0x7F80_0000u32,
    f64 => 0x7FF0_0000_0000_0000u64
);

// ──────────────── constant-time constructors (the `ct` feature) ────────────
//
// Masked (`subtle::CtOption`) counterparts of the plain branching constructors,
// one per typestate family, built on the existing `ct` predicates. The plain
// constructors are the documented Tier-B baseline; these keep the secret-derived
// predicate masked. Plain (never-`const`) like the rest of `ops::ct`.

#[cfg(feature = "ct")]
pub use ct_constructors::CtNonZero;

#[cfg(feature = "ct")]
mod ct_constructors {
    use super::{Even, HasNonZero, NonMin, NonNegative, Odd, Positive, PowerOfTwo};
    use crate::ops::ct::{CtIsPowerOfTwo, CtIsZero, CtParity};
    use subtle::{Choice, ConstantTimeEq, CtOption};

    macro_rules! ct_pow2 {
        ($($t:ty),+) => {$(
            impl PowerOfTwo<$t> {
                /// Constant-time [`new`](Self::new): a `CtOption` that is
                /// `None`-masked when `value` is not a power of two.
                #[inline]
                pub fn new_ct(value: $t) -> CtOption<PowerOfTwo<$t>> {
                    let choice = value.ct_is_power_of_two();
                    // Branchless exponent: for a power of two `trailing_zeros`
                    // IS the exponent. For `0` it returns BITS, and `BITS % BITS`
                    // wraps to 0 (still `< BITS`), so the unchecked invariant
                    // holds even though that value is masked out below.
                    let exp = value.trailing_zeros() % <$t>::BITS;
                    // SAFETY: `exp < BITS` always; meaningful only when `choice`
                    // is set (value is a power of two), masked out otherwise.
                    let proof = unsafe { PowerOfTwo::from_exp_unchecked(exp) };
                    CtOption::new(proof, choice)
                }
            }
        )+};
    }
    ct_pow2!(u8, u16, u32, u64, u128, usize);

    macro_rules! ct_sign {
        ($($t:ty),+) => {$(
            impl NonNegative<$t> {
                /// Constant-time [`new`](Self::new): `None`-masked when `value < 0`.
                #[inline]
                pub fn new_ct(value: $t) -> CtOption<NonNegative<$t>> {
                    let neg = Choice::from(((value >> (<$t>::BITS - 1)) & 1) as u8);
                    CtOption::new(NonNegative(value), !neg)
                }
            }
            impl Positive<$t> {
                /// Constant-time [`new`](Self::new): `None`-masked when `value <= 0`.
                #[inline]
                pub fn new_ct(value: $t) -> CtOption<Positive<$t>> {
                    let neg = Choice::from(((value >> (<$t>::BITS - 1)) & 1) as u8);
                    let zero = value.ct_is_zero();
                    CtOption::new(Positive(value), !neg & !zero)
                }
            }
        )+};
    }
    ct_sign!(i8, i16, i32, i64, i128, isize);

    macro_rules! ct_nonmin {
        ($($t:ty),+) => {$(
            impl NonMin<$t> {
                /// Constant-time [`new`](Self::new): `None`-masked when
                /// `value == MIN`.
                #[inline]
                pub fn new_ct(value: $t) -> CtOption<NonMin<$t>> {
                    let is_min = value.ct_eq(&<$t>::MIN);
                    CtOption::new(NonMin(value), !is_min)
                }
            }
        )+};
    }
    ct_nonmin!(i8, i16, i32, i64, i128, isize);

    impl<T: CtParity + Copy> Odd<T> {
        /// Constant-time [`new`](Self::new): `None`-masked when `value` is even.
        #[inline]
        pub fn new_ct(value: T) -> CtOption<Odd<T>> {
            let choice = value.ct_is_odd();
            CtOption::new(Odd(value), choice)
        }
    }

    impl<T: CtParity + Copy> Even<T> {
        /// Constant-time [`new`](Self::new): `None`-masked when `value` is odd.
        #[inline]
        pub fn new_ct(value: T) -> CtOption<Even<T>> {
            let choice = value.ct_is_even();
            CtOption::new(Even(value), choice)
        }
    }

    /// Constant-time bridge to [`core::num::NonZero`] — the masked counterpart
    /// of [`HasNonZero::into_nonzero`](super::HasNonZero::into_nonzero).
    pub trait CtNonZero: HasNonZero {
        /// `None`-masked when `self == 0`.
        fn into_nonzero_ct(self) -> CtOption<Self::NonZero>;
    }

    macro_rules! ct_nonzero {
        ($($t:ty),+) => {$(
            impl CtNonZero for $t {
                #[inline]
                fn into_nonzero_ct(self) -> CtOption<core::num::NonZero<$t>> {
                    let zero = self.ct_is_zero();
                    // Force non-zero in the masked branch so new_unchecked stays
                    // sound (a `NonZero` holding 0 is UB even when masked out).
                    let safe = self | (zero.unwrap_u8() as $t);
                    let nz = unsafe { core::num::NonZero::new_unchecked(safe) };
                    CtOption::new(nz, !zero)
                }
            }
        )+};
    }
    ct_nonzero!(u8, u16, u32, u64, u128, usize);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construction_and_accessors() {
        assert!(PowerOfTwo::<u32>::new(0).is_none());
        assert!(PowerOfTwo::<u32>::new(3).is_none());
        assert!(PowerOfTwo::<u32>::new(6).is_none());

        let p = PowerOfTwo::<u32>::new(64).unwrap();
        assert_eq!(p.exp(), 6);
        assert_eq!(p.get(), 64);

        // edges: 2^0 = 1, and the largest power of two for the type
        assert_eq!(PowerOfTwo::<u8>::new(1).unwrap().exp(), 0);
        assert_eq!(PowerOfTwo::<u8>::new(128).unwrap().get(), 128);

        // the generic safe constructor agrees with the per-primitive `new`
        assert_eq!(PowerOfTwo::<u32>::new_checked(64).unwrap().exp(), 6);
        assert!(PowerOfTwo::<u32>::new_checked(63).is_none());
        assert_eq!(PowerOfTwo::<u8>::new_checked(128).unwrap().get(), 128);
        assert_eq!(BitIndex::<u32>::new_checked(5).unwrap().get(), 5);
        assert!(BitIndex::<u32>::new_checked(32).is_none());
    }

    #[test]
    fn consuming_ops() {
        let p = PowerOfTwo::<u32>::new(16).unwrap();
        assert_eq!(100u32.div_pow2(p), 100 / 16);
        assert_eq!(100u32.rem_pow2(p), 100 % 16);
        assert!(!100u32.is_multiple_of_pow2(p));
        assert!(96u32.is_multiple_of_pow2(p));

        // 2^0 = 1: div is identity, rem is always 0
        let one = PowerOfTwo::<u64>::new(1).unwrap();
        assert_eq!(12345u64.div_pow2(one), 12345);
        assert_eq!(12345u64.rem_pow2(one), 0);
        assert!(12345u64.is_multiple_of_pow2(one));
    }

    #[test]
    fn matches_naive_exhaustive_u8() {
        for k in 0..8u32 {
            let d = 1u8 << k;
            let p = PowerOfTwo::<u8>::new(d).unwrap();
            for x in 0..=u8::MAX {
                assert_eq!(x.div_pow2(p), x >> k);
                assert_eq!(x.rem_pow2(p), x & (d - 1));
                assert_eq!(x.is_multiple_of_pow2(p), x % d == 0);
            }
        }
    }

    #[test]
    fn const_constructor_on_stable() {
        const P: Option<PowerOfTwo<u32>> = PowerOfTwo::<u32>::new(256);
        const GOT: u32 = match P {
            Some(p) => p.get(),
            None => 0,
        };
        assert_eq!(GOT, 256);
    }

    #[test]
    fn next_multiple_of_pow2_works() {
        let p = PowerOfTwo::<u32>::new(8).unwrap();
        assert_eq!(5u32.next_multiple_of_pow2(p), 8);
        assert_eq!(8u32.next_multiple_of_pow2(p), 8);
        assert_eq!(9u32.next_multiple_of_pow2(p), 16);
        assert_eq!(0u32.next_multiple_of_pow2(p), 0);
        assert_eq!(100u32.checked_next_multiple_of_pow2(p), Some(104));
        assert_eq!(u32::MAX.checked_next_multiple_of_pow2(p), None);
    }

    #[test]
    fn nonzero_bridge_and_div() {
        let d = 17u32.into_nonzero().unwrap();
        assert_eq!(u32::nonzero_get(d), 17);
        assert_eq!(100u32.div_nonzero(d), 100 / 17);
        assert_eq!(100u32.rem_nonzero(d), 100 % 17);
        assert!(0u32.into_nonzero().is_none());
    }

    #[test]
    fn sign_typestates() {
        assert!(NonNegative::<i32>::new(-1).is_none());
        assert_eq!(NonNegative::<i32>::new(5).unwrap().to_unsigned(), 5u32);
        assert_eq!(NonNegative::<i32>::new(0).unwrap().abs(), 0);
        assert_eq!(NonNegative::<i32>::new(81).unwrap().isqrt(), 9);

        assert!(Positive::<i32>::new(0).is_none());
        assert!(Positive::<i32>::new(-1).is_none());
        let p = Positive::<i32>::new(7).unwrap();
        assert_eq!(p.to_unsigned(), 7u32);
        assert_eq!(p.into_nonnegative().get(), 7);
        assert_eq!(p.into_nonzero().get(), 7);

        // borrowed proofs
        let v = 5i32;
        assert!(NonNegative::<i32>::from_ref(&v).is_some());
        let n = -5i32;
        assert!(Positive::<i32>::from_ref(&n).is_none());
    }

    #[test]
    fn odd_typestate() {
        assert!(Odd::<u32>::new(4).is_none());
        assert_eq!(Odd::<u32>::new(7).unwrap().get(), 7);
        assert!(Odd::<i32>::new(-3).is_some()); // -3 is odd
        // from_ref via the blanket `Parity for &T` (D1)
        let v = 9u32;
        assert!(Odd::from_ref(&v).is_some());
        let e = 8u32;
        assert!(Odd::from_ref(&e).is_none());
        // Even sibling
        assert!(Even::<u32>::new(4).is_some());
        assert!(Even::<u32>::new(7).is_none());
        assert_eq!(Even::<u32>::new(8).unwrap().get(), 8);
        assert!(Even::from_ref(&8u32).is_some());
        // borrow the proven value without consuming the proof
        let o = Odd::<u32>::new(7).unwrap();
        assert_eq!(*o.as_ref(), 7);
        let ev = Even::<u32>::new(8).unwrap();
        assert_eq!(*ev.as_ref(), 8);
    }

    #[test]
    fn try_from_checked_constructors() {
        // Odd / Even (generic over Parity)
        assert_eq!(Odd::<u32>::try_from(7).map(|o| o.get()), Ok(7));
        assert_eq!(Odd::<u32>::try_from(8), Err(TypestateError));
        assert_eq!(Even::<u32>::try_from(8).map(|e| e.get()), Ok(8));
        // via the TryInto sugar (the point of using the std trait)
        let o: Odd<i32> = (-3i32).try_into().unwrap();
        assert_eq!(o.get(), -3);
        // sign types + NonMin (per-primitive)
        assert_eq!(NonNegative::<i32>::try_from(0).map(|n| n.get()), Ok(0));
        assert_eq!(NonNegative::<i32>::try_from(-1), Err(TypestateError));
        assert_eq!(Positive::<i32>::try_from(0), Err(TypestateError));
        assert_eq!(Positive::<i32>::try_from(5).map(|p| p.get()), Ok(5));
        assert_eq!(NonMin::<i8>::try_from(i8::MIN), Err(TypestateError));
        assert_eq!(NonMin::<i8>::try_from(5).map(|n| n.get()), Ok(5));
        // PowerOfTwo / BitIndex (no Debug/PartialEq on the proof — project first)
        assert_eq!(PowerOfTwo::<u32>::try_from(64).map(|p| p.exp()), Ok(6));
        assert!(PowerOfTwo::<u32>::try_from(63).is_err());
        assert_eq!(BitIndex::<u64>::try_from(40u32).map(|i| i.get()), Ok(40));
        assert!(BitIndex::<u8>::try_from(8u32).is_err());
        // Finite (derives Debug; manual PartialEq)
        let x: Finite<f64> = 1.5f64.try_into().unwrap();
        assert_eq!(x.get(), 1.5);
        assert_eq!(Finite::<f64>::try_from(f64::NAN), Err(TypestateError));
    }

    #[test]
    fn bit_index_ops() {
        assert!(BitIndex::<u8>::new(8).is_none()); // == BITS
        assert!(BitIndex::<u8>::new(7).is_some());
        assert!(BitIndex::<u32>::new(32).is_none());
        let i = BitIndex::<u8>::new(3).unwrap();
        assert_eq!(i.get(), 3);
        assert_eq!(1u8.shl_index(i), 8);
        assert_eq!(0x80u8.shr_index(i), 0x10);
        // signed shifts too (arithmetic shr)
        let j = BitIndex::<i32>::new(2).unwrap();
        assert_eq!((-16i32).shr_index(j), -4);
        assert_eq!(3i32.shl_index(j), 12);
        // exhaustive vs raw shift for u8
        for n in 0..8u32 {
            let bi = BitIndex::<u8>::new(n).unwrap();
            for x in 0..=u8::MAX {
                assert_eq!(x.shl_index(bi), x << n);
                assert_eq!(x.shr_index(bi), x >> n);
            }
        }
    }

    #[test]
    fn nonmin_ops() {
        assert!(NonMin::<i32>::new(i32::MIN).is_none());
        assert!(NonMin::<i8>::new(i8::MIN).is_none());
        let a = NonMin::<i32>::new(-7).unwrap();
        assert_eq!(a.get(), -7);
        assert_eq!(a.abs(), 7);
        assert_eq!(a.neg(), 7);
        let d = core::num::NonZero::new(2i32).unwrap();
        assert_eq!(a.div_nonzero(d), -7 / 2);
        assert_eq!(a.rem_nonzero(d), -7 % 2);
        // the dangerous case is now well-typed: MIN+1 / -1 is fine, MIN is unrepresentable
        let near = NonMin::<i32>::new(i32::MIN + 1).unwrap();
        let neg1 = core::num::NonZero::new(-1i32).unwrap();
        assert_eq!(near.div_nonzero(neg1), i32::MAX);
        // borrowed proof
        assert!(NonMin::<i32>::from_ref(&i32::MIN).is_none());
        assert!(NonMin::<i32>::from_ref(&5).is_some());
        // lattice: Positive / NonNegative narrow into NonMin for free
        let p = Positive::<i32>::new(9).unwrap();
        assert_eq!(p.into_nonmin().abs(), 9);
        let nn = NonNegative::<i32>::new(0).unwrap();
        assert_eq!(nn.into_nonmin().neg(), 0);
    }

    #[test]
    fn finite_total_order() {
        assert!(Finite::<f64>::new(f64::NAN).is_none());
        assert!(Finite::<f64>::new(f64::INFINITY).is_none());
        assert!(Finite::<f64>::new(f64::NEG_INFINITY).is_none());
        assert!(Finite::<f32>::new(f32::NAN).is_none());
        assert!(Finite::<f64>::new(1.5).is_some());
        assert!(Finite::<f64>::new(0.0).is_some());

        let a = Finite::<f64>::new(1.0).unwrap();
        let b = Finite::<f64>::new(2.0).unwrap();
        assert!(a < b);
        assert_eq!(a.max(b).get(), 2.0);
        assert_eq!(a.min(b).get(), 1.0);
        assert_eq!(a.cmp(&b), core::cmp::Ordering::Less);
        // -0.0 and 0.0 compare equal under the total order
        let z = Finite::<f64>::new(0.0).unwrap();
        let nz = Finite::<f64>::new(-0.0).unwrap();
        assert_eq!(z, nz);
        assert_eq!(z.cmp(&nz), core::cmp::Ordering::Equal);
        // sortability is the payoff
        let mut v = [b, a, z];
        v.sort();
        assert_eq!([v[0].get(), v[1].get(), v[2].get()], [0.0, 1.0, 2.0]);
    }

    #[test]
    fn const_typestate_constructors_on_stable() {
        const I: Option<BitIndex<u32>> = BitIndex::<u32>::new(5);
        const NM: Option<NonMin<i32>> = NonMin::<i32>::new(-3);
        const F: Option<Finite<f64>> = Finite::<f64>::new(1.0);
        assert!(I.is_some() && NM.is_some() && F.is_some());
    }

    #[test]
    #[cfg(feature = "ct")]
    fn ct_constructors_mask_correctly() {
        // PowerOfTwo
        assert!(bool::from(PowerOfTwo::<u32>::new_ct(16).is_some()));
        assert!(bool::from(PowerOfTwo::<u32>::new_ct(15).is_none()));
        assert!(bool::from(PowerOfTwo::<u32>::new_ct(0).is_none()));
        assert_eq!(PowerOfTwo::<u32>::new_ct(16).unwrap().get(), 16);
        // sign
        assert!(bool::from(NonNegative::<i32>::new_ct(0).is_some()));
        assert!(bool::from(NonNegative::<i32>::new_ct(-1).is_none()));
        assert!(bool::from(Positive::<i32>::new_ct(0).is_none()));
        assert!(bool::from(Positive::<i32>::new_ct(5).is_some()));
        // Odd / Even
        assert!(bool::from(Odd::<u32>::new_ct(7).is_some()));
        assert!(bool::from(Odd::<u32>::new_ct(8).is_none()));
        assert!(bool::from(Even::<u32>::new_ct(8).is_some()));
        assert!(bool::from(Even::<u32>::new_ct(7).is_none()));
        // NonZero bridge
        assert!(bool::from(17u32.into_nonzero_ct().is_some()));
        assert!(bool::from(0u32.into_nonzero_ct().is_none()));
        assert_eq!(17u32.into_nonzero_ct().unwrap().get(), 17);
        // NonMin
        assert!(bool::from(NonMin::<i32>::new_ct(5).is_some()));
        assert!(bool::from(NonMin::<i32>::new_ct(i32::MIN).is_none()));
        assert_eq!(NonMin::<i32>::new_ct(-9).unwrap().get(), -9);
    }
}
