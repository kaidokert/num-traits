//! Carry/borrow-propagating and widening integer-arithmetic primitives for
//! chaining multi-word ("bigint") arithmetic, mirroring std's unstable
//! `bigint_helper_methods` (`carrying_add` / `borrowing_sub` / `carrying_mul`
//! / `widening_mul`). For unsigned types these stabilized in Rust 1.91; the
//! signed forms and `widening_mul` are still unstable.
//!
//! All bodies are hand-rolled (ported from core's portable intrinsic
//! fallbacks) so they work on the crate's MSRV and inside `const` on nightly.
//!
//! **CT tier A (CT-implementable)**: carries and borrows are returned for
//! arithmetic consumption; all bodies are branchless on the data.

#[cfg(target_pointer_width = "16")]
type UDoubleSize = u32;
#[cfg(target_pointer_width = "32")]
type UDoubleSize = u64;
#[cfg(target_pointer_width = "64")]
type UDoubleSize = u128;

#[cfg(target_pointer_width = "16")]
type IDoubleSize = i32;
#[cfg(target_pointer_width = "32")]
type IDoubleSize = i64;
#[cfg(target_pointer_width = "64")]
type IDoubleSize = i128;

c0nst::c0nst! {
/// Performs addition with a carry bit, for chaining multi-word additions.
pub c0nst trait CarryingAdd: Sized {
    /// The sum type (`Self` for the primitive impls).
    type Output;
    /// Calculates `self + rhs + carry` and returns a tuple containing the sum
    /// and the output carry, performing the full addition rather than
    /// stopping at the first overflow.
    ///
    /// For unsigned types this allows chaining together multiple additions
    /// to create a wider addition. For signed types the second tuple field
    /// signals two's-complement overflow instead of a carry.
    ///
    /// ```
    /// use const_num_traits::CarryingAdd;
    ///
    /// // u8::MAX + 1 + carry
    /// assert_eq!(CarryingAdd::carrying_add(u8::MAX, 1, true), (1, true));
    /// assert_eq!(CarryingAdd::carrying_add(5u8, 2, false), (7, false));
    /// ```
    fn carrying_add(self, rhs: Self, carry: bool) -> (Self::Output, bool);
}
}

macro_rules! carrying_add_impl {
    // unsigned: carries combine with OR
    (unsigned $($t:ty)*) => {$(
        c0nst::c0nst! {
        c0nst impl CarryingAdd for $t {
            type Output = $t;
            #[inline]
            fn carrying_add(self, rhs: Self, carry: bool) -> ($t, bool) {
                let (a, c1) = <$t>::overflowing_add(self, rhs);
                let (b, c2) = <$t>::overflowing_add(a, carry as $t);
                // only one overflow can happen, so c1 | c2 never double-counts
                (b, c1 | c2)
            }
        }
        }
    )*};
    // signed: overflows cancel out pairwise, so XOR
    (signed $($t:ty)*) => {$(
        c0nst::c0nst! {
        c0nst impl CarryingAdd for $t {
            type Output = $t;
            #[inline]
            fn carrying_add(self, rhs: Self, carry: bool) -> ($t, bool) {
                let (a, b) = <$t>::overflowing_add(self, rhs);
                let (c, d) = <$t>::overflowing_add(a, carry as $t);
                (c, b != d)
            }
        }
        }
    )*};
}

carrying_add_impl!(unsigned usize u8 u16 u32 u64 u128);
carrying_add_impl!(signed isize i8 i16 i32 i64 i128);

c0nst::c0nst! {
/// Performs subtraction with a borrow bit, for chaining multi-word subtractions.
pub c0nst trait BorrowingSub: Sized {
    /// The difference type (`Self` for the primitive impls).
    type Output;
    /// Calculates `self - rhs - borrow` and returns a tuple containing the
    /// difference and the output borrow, performing the full subtraction
    /// rather than stopping at the first overflow.
    ///
    /// For unsigned types this allows chaining together multiple
    /// subtractions to create a wider subtraction. For signed types the
    /// second tuple field signals two's-complement overflow instead.
    ///
    /// ```
    /// use const_num_traits::BorrowingSub;
    ///
    /// assert_eq!(BorrowingSub::borrowing_sub(0u8, 0, true), (u8::MAX, true));
    /// assert_eq!(BorrowingSub::borrowing_sub(7u8, 2, false), (5, false));
    /// ```
    fn borrowing_sub(self, rhs: Self, borrow: bool) -> (Self::Output, bool);
}
}

macro_rules! borrowing_sub_impl {
    (unsigned $($t:ty)*) => {$(
        c0nst::c0nst! {
        c0nst impl BorrowingSub for $t {
            type Output = $t;
            #[inline]
            fn borrowing_sub(self, rhs: Self, borrow: bool) -> ($t, bool) {
                let (a, c1) = <$t>::overflowing_sub(self, rhs);
                let (b, c2) = <$t>::overflowing_sub(a, borrow as $t);
                (b, c1 | c2)
            }
        }
        }
    )*};
    (signed $($t:ty)*) => {$(
        c0nst::c0nst! {
        c0nst impl BorrowingSub for $t {
            type Output = $t;
            #[inline]
            fn borrowing_sub(self, rhs: Self, borrow: bool) -> ($t, bool) {
                let (a, b) = <$t>::overflowing_sub(self, rhs);
                let (c, d) = <$t>::overflowing_sub(a, borrow as $t);
                (c, b != d)
            }
        }
        }
    )*};
}

borrowing_sub_impl!(unsigned usize u8 u16 u32 u64 u128);
borrowing_sub_impl!(signed isize i8 i16 i32 i64 i128);

c0nst::c0nst! {
/// Performs full double-width multiplication with carry and addend inputs,
/// the workhorse for multi-word ("bigint") multiplication.
pub c0nst trait CarryingMul: Sized {
    /// The type of the low half of the result: `Self` for unsigned types,
    /// the unsigned counterpart for signed types (matching std's
    /// `carrying_mul` signatures).
    type Unsigned;

    /// The type of the high half of the result (`Self` for the primitive impls).
    type Output;

    /// Calculates the "full multiplication" `self * rhs + carry` without the
    /// possibility to overflow, returning `(low, high)` words of the result.
    ///
    /// ```
    /// use const_num_traits::CarryingMul;
    ///
    /// assert_eq!(CarryingMul::carrying_mul(u8::MAX, u8::MAX, u8::MAX), (0, u8::MAX));
    /// assert_eq!(CarryingMul::carrying_mul(5u8, 7, 3), (38, 0));
    /// ```
    fn carrying_mul(self, rhs: Self, carry: Self) -> (Self::Unsigned, Self::Output);

    /// Calculates the "full multiplication" `self * rhs + carry + add`
    /// without the possibility to overflow, returning `(low, high)` words.
    ///
    /// This cannot overflow because `MAX * MAX + MAX + MAX` still fits in
    /// twice the bit width.
    fn carrying_mul_add(self, rhs: Self, carry: Self, add: Self) -> (Self::Unsigned, Self::Output);
}
}

macro_rules! carrying_mul_impl {
    // widening through a double-width type; for signed types the low half
    // comes back as the unsigned counterpart, like std
    ($($t:ty, $u:ty, $w:ty;)*) => {$(
        c0nst::c0nst! {
        c0nst impl CarryingMul for $t {
            type Unsigned = $u;
            type Output = $t;

            #[inline]
            fn carrying_mul(self, rhs: Self, carry: Self) -> ($u, $t) {
                let wide = (self as $w) * (rhs as $w) + (carry as $w);
                (wide as $u, (wide >> <$t>::BITS) as $t)
            }

            #[inline]
            fn carrying_mul_add(self, rhs: Self, carry: Self, add: Self) -> ($u, $t) {
                let wide = (self as $w) * (rhs as $w) + (carry as $w) + (add as $w);
                (wide as $u, (wide >> <$t>::BITS) as $t)
            }
        }
        }
    )*};
}

carrying_mul_impl! {
    u8, u8, u16;
    u16, u16, u32;
    u32, u32, u64;
    u64, u64, u128;
    usize, usize, UDoubleSize;
    i8, u8, i16;
    i16, u16, i32;
    i32, u32, i64;
    i64, u64, i128;
    isize, usize, IDoubleSize;
}

// 128-bit types have no wider type to widen through; multiply in 64-bit
// limbs instead. Ported from core's portable intrinsic fallback.
pub(crate) const fn wide_mul_u128(a: u128, b: u128) -> (u128, u128) {
    const MASK: u128 = u64::MAX as u128;
    let (a_lo, a_hi) = (a & MASK, a >> 64);
    let (b_lo, b_hi) = (b & MASK, b >> 64);

    // low = a * b_lo, expressed in three 64-bit limbs
    let t = a_lo * b_lo;
    let (low0, c) = (t & MASK, t >> 64);
    let t = a_hi * b_lo + c;
    let (low1, low2) = (t & MASK, t >> 64);

    // high = a * b_hi, shifted up by one limb
    let t = a_lo * b_hi;
    let (high0, c) = (t & MASK, t >> 64);
    let t = a_hi * b_hi + c;
    let (high1, high2) = (t & MASK, t >> 64);

    let t = low1 + high0;
    let (r1, c) = (t & MASK, t >> 64);
    let t = low2 + high1 + c;
    let (r2, c) = (t & MASK, t >> 64);
    let r3 = high2 + c;
    (low0 | (r1 << 64), r2 | (r3 << 64))
}

c0nst::c0nst! {
c0nst impl CarryingMul for u128 {
    type Unsigned = u128;
    type Output = u128;

    #[inline]
    fn carrying_mul(self, rhs: Self, carry: Self) -> (u128, u128) {
        let (low, mut high) = wide_mul_u128(self, rhs);
        let (low, c) = u128::overflowing_add(low, carry);
        high += c as u128;
        (low, high)
    }

    #[inline]
    fn carrying_mul_add(self, rhs: Self, carry: Self, add: Self) -> (u128, u128) {
        let (low, mut high) = wide_mul_u128(self, rhs);
        let (low, c) = u128::overflowing_add(low, carry);
        high += c as u128;
        let (low, c) = u128::overflowing_add(low, add);
        high += c as u128;
        (low, high)
    }
}
}

c0nst::c0nst! {
c0nst impl CarryingMul for i128 {
    type Unsigned = u128;
    type Output = i128;

    #[inline]
    fn carrying_mul(self, rhs: Self, carry: Self) -> (u128, i128) {
        let zero = 0i128;
        CarryingMul::carrying_mul_add(self, rhs, carry, zero)
    }

    // Unsigned 128x128 multiply, then correct the high word for the signs
    // of the inputs (`x >> 127` is 0 or -1) and sign-extend the addends.
    #[inline]
    fn carrying_mul_add(self, rhs: Self, carry: Self, add: Self) -> (u128, i128) {
        let (low, high) = wide_mul_u128(self as u128, rhs as u128);
        let mut high = high as i128;
        high = high.wrapping_add(i128::wrapping_mul(self >> 127, rhs));
        high = high.wrapping_add(i128::wrapping_mul(self, rhs >> 127));
        let (low, c) = u128::overflowing_add(low, carry as u128);
        high = high.wrapping_add((c as i128) + (carry >> 127));
        let (low, c) = u128::overflowing_add(low, add as u128);
        high = high.wrapping_add((c as i128) + (add >> 127));
        (low, high)
    }
}
}

c0nst::c0nst! {
/// Performs multiplication that widens into the next-larger integer type, so
/// it cannot overflow.
pub c0nst trait WideningMul: Sized {
    /// The double-width result type.
    type Wide;

    /// Widening multiplication. Computes the complete product `self * rhs`
    /// in the next-larger primitive, mirroring core's `widening_mul`
    /// (`fn widening_mul(self, rhs) -> $WideT` — the single-wide form, as of
    /// rust-lang/rust#156644).
    ///
    /// Because it promotes to a wider *primitive*, it is only provided for
    /// types that have one (`u8`–`u64`, `i8`–`i64`). `u128`/`usize` and
    /// arbitrary-precision types — which have no wider primitive — should
    /// use [`CarryingMul::carrying_mul`] instead, which returns the product
    /// as `(low, high)` words and works for every width.
    ///
    /// ```
    /// use const_num_traits::WideningMul;
    ///
    /// assert_eq!(WideningMul::widening_mul(u8::MAX, u8::MAX), 65025u16);
    /// assert_eq!(WideningMul::widening_mul(-128i8, -128), 16384i16);
    /// ```
    fn widening_mul(self, rhs: Self) -> Self::Wide;
}
}

macro_rules! widening_mul_impl {
    ($($t:ty => $w:ty;)*) => {$(
        c0nst::c0nst! {
        c0nst impl WideningMul for $t {
            type Wide = $w;

            #[inline]
            fn widening_mul(self, rhs: Self) -> $w {
                (self as $w) * (rhs as $w)
            }
        }
        }
    )*};
}

widening_mul_impl! {
    u8 => u16;
    u16 => u32;
    u32 => u64;
    u64 => u128;
    i8 => i16;
    i16 => i32;
    i32 => i64;
    i64 => i128;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn carrying_add_borrowing_sub() {
        assert_eq!(
            CarryingAdd::carrying_add(u8::MAX, u8::MAX, true),
            (255, true)
        );
        assert_eq!(
            CarryingAdd::carrying_add(0xFFFF_FFFF_FFFF_FFFFu64, 0, true),
            (0, true)
        );
        assert_eq!(CarryingAdd::carrying_add(i8::MAX, 0, true), (i8::MIN, true));
        assert_eq!(CarryingAdd::carrying_add(-1i8, 1, false), (0, false));
        assert_eq!(BorrowingSub::borrowing_sub(0u8, u8::MAX, true), (0, true));
        assert_eq!(BorrowingSub::borrowing_sub(10u8, 3, true), (6, false));
        assert_eq!(
            BorrowingSub::borrowing_sub(i8::MIN, 0, true),
            (i8::MAX, true)
        );
    }

    #[test]
    fn carrying_mul_small() {
        // chain two words of 255*255
        assert_eq!(CarryingMul::carrying_mul(u8::MAX, u8::MAX, 0), (1, 254));
        assert_eq!(
            CarryingMul::carrying_mul_add(u8::MAX, u8::MAX, u8::MAX, u8::MAX),
            (255, 255)
        );
        assert_eq!(CarryingMul::carrying_mul(-1i8, -1, 0), (1, 0));
        assert_eq!(CarryingMul::carrying_mul(i8::MIN, i8::MIN, 0), (0, 64));
        assert_eq!(CarryingMul::carrying_mul(-1i8, 1, 0), (255, -1));
    }

    #[test]
    fn carrying_mul_128() {
        // (2^127) * 2 = 2^128 -> low 0, high 1
        assert_eq!(CarryingMul::carrying_mul(1u128 << 127, 2, 0), (0, 1));
        assert_eq!(
            CarryingMul::carrying_mul(u128::MAX, u128::MAX, u128::MAX),
            (0, u128::MAX)
        );
        // cross-check a "random" value against 64-bit limb math
        let a = 0x0123_4567_89ab_cdef_fedc_ba98_7654_3210u128;
        let b = 0x0fed_cba9_8765_4321_1234_5678_9abc_def0u128;
        let (low, high) = CarryingMul::carrying_mul(a, b, 0);
        // verify via the identity (a*b) mod 2^128 == low
        assert_eq!(a.wrapping_mul(b), low);
        // and high via floor(a*b / 2^128) computed with 64-bit limbs
        let (l2, h2) = super::wide_mul_u128(a, b);
        assert_eq!((low, high), (l2, h2));

        // signed: -1 * -1 = 1
        assert_eq!(CarryingMul::carrying_mul(-1i128, -1, 0), (1, 0));
        // signed: MIN * -1 = 2^126 * 2 = +2^127 -> (1<<127, 0)
        assert_eq!(
            CarryingMul::carrying_mul(i128::MIN, -1, 0),
            (1u128 << 127, 0)
        );
        assert_eq!(CarryingMul::carrying_mul(-1i128, 1, 0), (u128::MAX, -1));
    }

    #[test]
    fn widening_mul() {
        // single wide type, matching core's `widening_mul -> $WideT`
        assert_eq!(WideningMul::widening_mul(u8::MAX, u8::MAX), 65025u16);
        assert_eq!(
            WideningMul::widening_mul(u64::MAX, u64::MAX),
            u64::MAX as u128 * u64::MAX as u128
        );
        assert_eq!(WideningMul::widening_mul(i8::MIN, i8::MIN), 16384i16);
        assert_eq!(
            WideningMul::widening_mul(-1i32, i32::MAX),
            -(i32::MAX as i64)
        );
    }
}
