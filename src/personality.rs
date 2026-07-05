//! Personality typestate: a zero-size marker that selects which implementation
//! of an operation is picked at monomorphization.
//!
//! Two personalities ship here:
//!
//! - [`Nct`] (default): standard "non-constant-time" implementation.
//! - [`Ct`]: constant-time implementation. Slower bodies whose timing is
//!   independent of operand values, defensible against an adversarial
//!   optimizer. Used by signing paths and any code handling secret values.
//!
//! This is a pure-`core` marker (it depends only on [`PhantomData`]); it is
//! deliberately **not** gated behind the `ct` feature, which governs only the
//! `subtle`-backed constant-time *implementations*. It is intended to apply to
//! this crate's numeric operations, but nothing about the marker itself is
//! numeric — if a non-numeric consumer ever needs it, lifting it into its own
//! crate (re-exported here) is mechanical.
//!
//! ## Const-eval dispatch via `match P::TAG`
//!
//! Composite `const fn` methods that need to dispatch on personality can
//! `match` on the [`PersonalityTag`] associated constant:
//!
//! ```ignore
//! const fn dispatched<P: Personality>(x: u32) -> u32 {
//!     match P::TAG {
//!         PersonalityTag::Nct => fast_body(x),
//!         PersonalityTag::Ct  => ct_body(x),
//!     }
//! }
//! ```
//!
//! Trait *method* calls (e.g. `P::widening_mul(x, y)`) require unstable
//! `const_trait_impl` in const contexts and are not used by this pattern. Tag
//! dispatch works on stable today and composes through multi-level generic
//! `const fn` chains without loss of const-eval.

use core::marker::PhantomData;

mod sealed {
    pub trait Sealed {}
}

/// Marker trait for personalities. Sealed — implementations live in this
/// crate. Downstream code uses the [`Nct`] and [`Ct`] types directly or
/// writes `match P::TAG` dispatch using [`PersonalityTag`].
pub trait Personality: sealed::Sealed + Copy + 'static {
    /// Compile-time tag used by `const fn` dispatch:
    /// `match P::TAG { PersonalityTag::Nct => …, PersonalityTag::Ct => … }`.
    const TAG: PersonalityTag;
}

/// Compile-time tag identifying the active personality. Returned by
/// [`Personality::TAG`] and used as the discriminant for `match P::TAG`
/// dispatch in `const fn` bodies.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum PersonalityTag {
    /// Non-constant-time.
    Nct,
    /// Constant-time. Defensible against optimizer-introduced timing leaks.
    Ct,
}

/// Non-constant-time marker. The default personality.
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct Nct;
impl sealed::Sealed for Nct {}
impl Personality for Nct {
    const TAG: PersonalityTag = PersonalityTag::Nct;
}

/// Constant-time marker.
///
/// Selects constant-time implementations of operation primitives — bodies
/// whose execution time and memory access pattern do not depend on operand
/// values. Appropriate for code that handles secret values, where
/// operand-dependent timing or memory access would leak them.
///
/// Constant-time-only APIs (e.g. `subtle::ConditionallySelectable`,
/// `ConstantTimeEq`) should be exposed only for the `Ct` variant — wrong-
/// variant calls then become compile errors, not silent non-constant-time
/// execution.
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct Ct;
impl sealed::Sealed for Ct {}
impl Personality for Ct {
    const TAG: PersonalityTag = PersonalityTag::Ct;
}

/// `PhantomData` helper for storing a personality marker as a zero-size field.
/// Use as `_p: PersonalityMarker<P>` in struct definitions.
pub type PersonalityMarker<P> = PhantomData<fn() -> P>;

/// Projects a carrier type's [`Personality`] at the type level.
///
/// Implemented by carrier types whose implementation shape is selected
/// by a personality parameter (a bigint parameterized over its
/// personality projects that parameter), and by the primitive integers,
/// which project [`Nct`]: they expose a single hardware-backed
/// implementation, select no constant-time variant, and make no
/// constant-time guarantee — integer division in particular has
/// operand-dependent latency on common CPUs.
///
/// Consumers bound on the projection to gate algorithm choice by
/// personality: `T: HasPersonality<P = Nct>` admits a type into
/// variable-time code paths, `P = Ct` into constant-time-only ones.
/// The declaration is the carrier author's contract; there is no way
/// to verify it structurally.
pub trait HasPersonality {
    /// The carrier's personality.
    type P: Personality;
}

macro_rules! impl_has_personality_nct {
    ($($t:ty),*) => {
        $(
            impl HasPersonality for $t {
                type P = Nct;
            }
        )*
    };
}

impl_has_personality_nct!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tags_and_defaults() {
        assert_eq!(Nct::TAG, PersonalityTag::Nct);
        assert_eq!(Ct::TAG, PersonalityTag::Ct);
        assert_eq!(Nct, Nct);
        assert_eq!(Ct, Ct);
    }

    #[test]
    fn has_personality_projects_nct_for_primitives() {
        fn assert_nct<T: HasPersonality<P = Nct>>() {}
        assert_nct::<u8>();
        assert_nct::<u32>();
        assert_nct::<u128>();
        assert_nct::<usize>();
        assert_nct::<i64>();
    }

    #[test]
    fn const_tag_dispatch() {
        const fn dispatched<P: Personality>() -> u32 {
            match P::TAG {
                PersonalityTag::Nct => 1,
                PersonalityTag::Ct => 2,
            }
        }
        assert_eq!(dispatched::<Nct>(), 1);
        assert_eq!(dispatched::<Ct>(), 2);
    }
}
