use core::num::Wrapping;
use core::ops::{Add, Mul};

#[cfg(has_num_saturating)]
use core::num::Saturating;

c0nst::c0nst! {
/// Defines an additive identity element for `Self`.
///
/// This is a pure identity-*value* trait: it provides the `0` and a zero-test,
/// but does **not** require [`Add`](core::ops::Add). The bundling of `Add` is
/// upstream convention, not a law — `0` is equally the identity under
/// subtraction (`a - 0 = a`) — so consumers state the operators they actually
/// use explicitly (`T: Zero + Add`, `T: Zero + Sub`, …). Numeric code that
/// needs the full operator set takes [`Num`](crate::Num).
///
/// # Laws
///
/// ```text
/// a + 0 = a       ∀ a ∈ Self
/// 0 + a = a       ∀ a ∈ Self
/// ```
pub c0nst trait Zero: Sized {
    /// Returns the additive identity element of `Self`, `0`.
    /// # Purity
    ///
    /// This function should return the same result at all times regardless of
    /// external mutable state, for example values stored in TLS or in
    /// `static mut`s.
    // This cannot be an associated constant, because of bignums.
    fn zero() -> Self;

    /// Sets `self` to the additive identity element of `Self`, `0`.
    fn set_zero(&mut self);

    /// Returns `true` if `self` is equal to the additive identity.
    fn is_zero(&self) -> bool;
}
}

/// Defines an associated constant representing the additive identity element
/// for `Self`.
///
/// This is a pure constant-carrier: it requires only that `Self` *has* a
/// compile-time `0`, **not** that it implements [`Zero`] (and hence not
/// [`Add`](core::ops::Add)). Keeping it decoupled lets capability-restricted
/// types — e.g. a constant-time integer with bit ops but no arithmetic — supply
/// the `0` constant that [`PrimBits`](crate::PrimBits) needs without being
/// forced to implement addition. Numeric types still implement both.
pub trait ConstZero: Sized {
    /// The additive identity element of `Self`, `0`.
    const ZERO: Self;
}

macro_rules! zero_impl {
    ($t:ty, $v:expr) => {
        c0nst::c0nst! {
        c0nst impl Zero for $t {
            #[inline]
            fn zero() -> $t {
                $v
            }
            #[inline]
            fn set_zero(&mut self) {
                *self = $v;
            }
            #[inline]
            fn is_zero(&self) -> bool {
                *self == $v
            }
        }
        }

        impl ConstZero for $t {
            const ZERO: Self = $v;
        }
    };
}

zero_impl!(usize, 0);
zero_impl!(u8, 0);
zero_impl!(u16, 0);
zero_impl!(u32, 0);
zero_impl!(u64, 0);
zero_impl!(u128, 0);

zero_impl!(isize, 0);
zero_impl!(i8, 0);
zero_impl!(i16, 0);
zero_impl!(i32, 0);
zero_impl!(i64, 0);
zero_impl!(i128, 0);

zero_impl!(f32, 0.0);
zero_impl!(f64, 0.0);

c0nst::c0nst! {
c0nst impl<T: [c0nst] Zero> Zero for Wrapping<T>
where
    Wrapping<T>: [c0nst] Add<Output = Wrapping<T>>,
{
    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    fn set_zero(&mut self) {
        self.0.set_zero();
    }

    fn zero() -> Self {
        Wrapping(T::zero())
    }
}
}

impl<T: ConstZero> ConstZero for Wrapping<T>
where
    Wrapping<T>: Add<Output = Wrapping<T>>,
{
    const ZERO: Self = Wrapping(T::ZERO);
}

#[cfg(has_num_saturating)]
c0nst::c0nst! {
c0nst impl<T: [c0nst] Zero> Zero for Saturating<T>
where
    Saturating<T>: [c0nst] Add<Output = Saturating<T>>,
{
    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    fn set_zero(&mut self) {
        self.0.set_zero();
    }

    fn zero() -> Self {
        Saturating(T::zero())
    }
}
}

#[cfg(has_num_saturating)]
impl<T: ConstZero> ConstZero for Saturating<T>
where
    Saturating<T>: Add<Output = Saturating<T>>,
{
    const ZERO: Self = Saturating(T::ZERO);
}

c0nst::c0nst! {
/// Defines a multiplicative identity element for `Self`.
///
/// A pure identity-*value* trait, decoupled from [`Mul`](core::ops::Mul) for the
/// same reason as [`Zero`]: `1` is equally the identity under division
/// (`a / 1 = a`), so privileging `Mul` is arbitrary. Consumers state what they
/// need (`T: One + Mul`, `T: One + Div`, …); full numeric code takes
/// [`Num`](crate::Num).
///
/// # Laws
///
/// ```text
/// a * 1 = a       ∀ a ∈ Self
/// 1 * a = a       ∀ a ∈ Self
/// ```
pub c0nst trait One: Sized {
    /// Returns the multiplicative identity element of `Self`, `1`.
    ///
    /// # Purity
    ///
    /// This function should return the same result at all times regardless of
    /// external mutable state, for example values stored in TLS or in
    /// `static mut`s.
    // This cannot be an associated constant, because of bignums.
    fn one() -> Self;

    /// Sets `self` to the multiplicative identity element of `Self`, `1`.
    fn set_one(&mut self);

    /// Returns `true` if `self` is equal to the multiplicative identity.
    fn is_one(&self) -> bool;
}
}

/// Defines an associated constant representing the multiplicative identity
/// element for `Self`.
///
/// A pure constant-carrier, decoupled from [`One`] (and hence
/// [`Mul`](core::ops::Mul)) for the same reason as [`ConstZero`]: a type can
/// supply the compile-time `1` that [`PrimBits`](crate::PrimBits) needs without
/// implementing multiplication. Numeric types still implement both.
pub trait ConstOne: Sized {
    /// The multiplicative identity element of `Self`, `1`.
    const ONE: Self;
}

macro_rules! one_impl {
    ($t:ty, $v:expr) => {
        c0nst::c0nst! {
        c0nst impl One for $t {
            #[inline]
            fn one() -> $t {
                $v
            }
            #[inline]
            fn set_one(&mut self) {
                *self = $v;
            }
            #[inline]
            fn is_one(&self) -> bool {
                *self == $v
            }
        }
        }

        impl ConstOne for $t {
            const ONE: Self = $v;
        }
    };
}

one_impl!(usize, 1);
one_impl!(u8, 1);
one_impl!(u16, 1);
one_impl!(u32, 1);
one_impl!(u64, 1);
one_impl!(u128, 1);

one_impl!(isize, 1);
one_impl!(i8, 1);
one_impl!(i16, 1);
one_impl!(i32, 1);
one_impl!(i64, 1);
one_impl!(i128, 1);

one_impl!(f32, 1.0);
one_impl!(f64, 1.0);

c0nst::c0nst! {
c0nst impl<T: [c0nst] One> One for Wrapping<T>
where
    Wrapping<T>: [c0nst] Mul<Output = Wrapping<T>>,
{
    fn set_one(&mut self) {
        self.0.set_one();
    }

    fn one() -> Self {
        Wrapping(T::one())
    }

    fn is_one(&self) -> bool {
        self.0.is_one()
    }
}
}

impl<T: ConstOne> ConstOne for Wrapping<T>
where
    Wrapping<T>: Mul<Output = Wrapping<T>>,
{
    const ONE: Self = Wrapping(T::ONE);
}

#[cfg(has_num_saturating)]
c0nst::c0nst! {
c0nst impl<T: [c0nst] One> One for Saturating<T>
where
    Saturating<T>: [c0nst] Mul<Output = Saturating<T>>,
{
    fn set_one(&mut self) {
        self.0.set_one();
    }

    fn one() -> Self {
        Saturating(T::one())
    }

    fn is_one(&self) -> bool {
        self.0.is_one()
    }
}
}

#[cfg(has_num_saturating)]
impl<T: ConstOne> ConstOne for Saturating<T>
where
    Saturating<T>: Mul<Output = Saturating<T>>,
{
    const ONE: Self = Saturating(T::ONE);
}

// Some helper functions provided for backwards compatibility.

c0nst::c0nst! {
/// Returns the additive identity, `0`.
#[inline(always)]
pub c0nst fn zero<T: [c0nst] Zero>() -> T {
    Zero::zero()
}
}

c0nst::c0nst! {
/// Returns the multiplicative identity, `1`.
#[inline(always)]
pub c0nst fn one<T: [c0nst] One>() -> T {
    One::one()
}
}

#[test]
fn wrapping_identities() {
    macro_rules! test_wrapping_identities {
        ($($t:ty)+) => {
            $(
                assert_eq!(zero::<$t>(), zero::<Wrapping<$t>>().0);
                assert_eq!(one::<$t>(), one::<Wrapping<$t>>().0);
                assert_eq!((0 as $t).is_zero(), Wrapping(0 as $t).is_zero());
                assert_eq!((1 as $t).is_zero(), Wrapping(1 as $t).is_zero());
            )+
        };
    }

    test_wrapping_identities!(isize i8 i16 i32 i64 usize u8 u16 u32 u64);
}

#[test]
fn wrapping_is_zero() {
    fn require_zero<T: Zero>(_: &T) {}
    require_zero(&Wrapping(42));
}
#[test]
fn wrapping_is_one() {
    fn require_one<T: One>(_: &T) {}
    require_one(&Wrapping(42));
}

#[test]
#[cfg(has_num_saturating)]
fn saturating_identities() {
    macro_rules! test_saturating_identities {
        ($($t:ty)+) => {
            $(
                assert_eq!(zero::<$t>(), zero::<Saturating<$t>>().0);
                assert_eq!(one::<$t>(), one::<Saturating<$t>>().0);
                assert_eq!((0 as $t).is_zero(), Saturating(0 as $t).is_zero());
                assert_eq!((1 as $t).is_zero(), Saturating(1 as $t).is_zero());
            )+
        };
    }

    test_saturating_identities!(isize i8 i16 i32 i64 usize u8 u16 u32 u64);
}

#[test]
#[cfg(has_num_saturating)]
fn saturating_is_zero() {
    fn require_zero<T: Zero>(_: &T) {}
    require_zero(&Saturating(42));
}
#[test]
#[cfg(has_num_saturating)]
fn saturating_is_one() {
    fn require_one<T: One>(_: &T) {}
    require_one(&Saturating(42));
}
