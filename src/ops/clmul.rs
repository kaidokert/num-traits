//! Carry-less ("polynomial" / GF(2)) multiplications — the clmul /
//! PCLMULQDQ family used by GHASH, CRC, and similar bit-polynomial codes.
//! Each partial product is XORed (rather than added) into the accumulator,
//! so there is no carry to propagate: a distinct domain from the integer
//! arithmetic in [`crate::ops::carrying`].
//!
//! All bodies are hand-rolled (ported from core's portable intrinsic
//! fallbacks) so they work on the crate's MSRV and inside `const` on nightly.
//!
//! **CT tier A (CT-implementable)**: all bodies are branchless on the data.

#[cfg(target_pointer_width = "16")]
type UDoubleSize = u32;
#[cfg(target_pointer_width = "32")]
type UDoubleSize = u64;
#[cfg(target_pointer_width = "64")]
type UDoubleSize = u128;

c0nst::c0nst! {
/// Performs carry-less ("polynomial" / XOR) multiplication.
pub c0nst trait CarrylessMul: Sized {
    /// Carry-less multiplication of `self` and `rhs`: a multiplication where
    /// the partial products are XORed instead of added, i.e. multiplication
    /// of polynomials over GF(2). Returns the low `BITS` bits of the result.
    ///
    /// ```
    /// use const_num_traits::CarrylessMul;
    ///
    /// // (x + 1)² = x² + 1 over GF(2)
    /// assert_eq!(CarrylessMul::carryless_mul(0b11u8, 0b11), 0b101);
    /// ```
    type Output;
    fn carryless_mul(self, rhs: Self) -> Self::Output;
}
}

macro_rules! carryless_mul_impl {
    ($($t:ty)*) => {$(
        c0nst::c0nst! {
        c0nst impl CarrylessMul for $t {
            type Output = $t;
            #[inline]
            fn carryless_mul(self, rhs: Self) -> $t {
                let mut result: $t = 0;
                let mut i = 0u32;
                while i < <$t>::BITS {
                    if (rhs >> i) & 1 == 1 {
                        result ^= self << i;
                    }
                    i += 1;
                }
                result
            }
        }
        }
    )*};
}

carryless_mul_impl!(usize u8 u16 u32 u64 u128);

c0nst::c0nst! {
/// Performs carry-less multiplication that widens into the next-larger
/// integer type, keeping all result bits.
pub c0nst trait WideningCarrylessMul: Sized {
    /// The double-width result type.
    type Wide;

    /// Widening carry-less multiplication: like
    /// [`CarrylessMul::carryless_mul`] but returning the full product in the
    /// double-width type. Like std, only provided for `u8`–`u64`.
    ///
    /// ```
    /// use const_num_traits::WideningCarrylessMul;
    ///
    /// assert_eq!(WideningCarrylessMul::widening_carryless_mul(0x80u8, 0x80), 0x4000u16);
    /// ```
    fn widening_carryless_mul(self, rhs: Self) -> Self::Wide;
}
}

macro_rules! widening_carryless_mul_impl {
    ($($t:ty => $w:ty;)*) => {$(
        c0nst::c0nst! {
        c0nst impl WideningCarrylessMul for $t {
            type Wide = $w;

            #[inline]
            fn widening_carryless_mul(self, rhs: Self) -> $w {
                let wide = self as $w;
                let mut result: $w = 0;
                let mut i = 0u32;
                while i < <$t>::BITS {
                    if (rhs >> i) & 1 == 1 {
                        result ^= wide << i;
                    }
                    i += 1;
                }
                result
            }
        }
        }
    )*};
}

widening_carryless_mul_impl! {
    u8 => u16;
    u16 => u32;
    u32 => u64;
    u64 => u128;
}

c0nst::c0nst! {
/// Performs full double-width carry-less multiplication with a carry-in,
/// for chaining multi-word carry-less ("polynomial") multiplications.
pub c0nst trait CarryingCarrylessMul: Sized {
    /// Calculates the full carry-less product `self ⊗ rhs`, XORs `carry`
    /// into the low word (in GF(2), "adding" the carry is XOR and can never
    /// propagate), and returns `(low, high)`.
    ///
    /// Unlike [`WideningCarrylessMul`] this is available for all unsigned
    /// types including `u128` and `usize`, mirroring std.
    ///
    /// ```
    /// use const_num_traits::CarryingCarrylessMul;
    ///
    /// assert_eq!(
    ///     CarryingCarrylessMul::carrying_carryless_mul(0x80u8, 0x80, 0b11),
    ///     (0b11, 0x40),
    /// );
    /// ```
    type Output;
    fn carrying_carryless_mul(self, rhs: Self, carry: Self) -> (Self::Output, Self::Output);
}
}

macro_rules! carrying_carryless_mul_impl {
    // clmul in the double-width type, then split
    ($($t:ty => $w:ty;)*) => {$(
        c0nst::c0nst! {
        c0nst impl CarryingCarrylessMul for $t {
            type Output = $t;
            #[inline]
            fn carrying_carryless_mul(self, rhs: Self, carry: Self) -> ($t, $t) {
                let a = self as $w;
                let mut p: $w = 0;
                let mut i = 0u32;
                while i < <$t>::BITS {
                    if (rhs >> i) & 1 == 1 {
                        p ^= a << i;
                    }
                    i += 1;
                }
                ((p as $t) ^ carry, (p >> <$t>::BITS) as $t)
            }
        }
        }
    )*};
}

carrying_carryless_mul_impl! {
    u8 => u16;
    u16 => u32;
    u32 => u64;
    u64 => u128;
    usize => UDoubleSize;
}

// u128 has no wider type; Karatsuba over three 64x64 carry-less products,
// same as core (carry-less multiplication distributes over XOR).
const fn wide_clmul_u64(a: u64, b: u64) -> u128 {
    let a = a as u128;
    let mut p: u128 = 0;
    let mut i = 0u32;
    while i < 64 {
        if (b >> i) & 1 == 1 {
            p ^= a << i;
        }
        i += 1;
    }
    p
}

c0nst::c0nst! {
c0nst impl CarryingCarrylessMul for u128 {
    type Output = u128;
    #[inline]
    fn carrying_carryless_mul(self, rhs: Self, carry: Self) -> (u128, u128) {
        let x0 = self as u64;
        let x1 = (self >> 64) as u64;
        let y0 = rhs as u64;
        let y1 = (rhs >> 64) as u64;

        let z0 = wide_clmul_u64(x0, y0);
        let z2 = wide_clmul_u64(x1, y1);
        // Karatsuba: z3 = (x0^x1)(y0^y1) = z0 ^ z1 ^ z2, so
        let z3 = wide_clmul_u64(x0 ^ x1, y0 ^ y1);
        let z1 = z3 ^ z0 ^ z2;

        let lo = z0 ^ (z1 << 64);
        let hi = z2 ^ (z1 >> 64);
        (lo ^ carry, hi)
    }
}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn carrying_carryless() {
        // small types: agree with widening clmul split + XOR'd carry
        for a in [0u8, 1, 3, 0x80, 0xAB, 0xFF] {
            for b in [0u8, 1, 3, 0x80, 0xCD, 0xFF] {
                let wide = WideningCarrylessMul::widening_carryless_mul(a, b);
                let (lo, hi) = CarryingCarrylessMul::carrying_carryless_mul(a, b, 0x5A);
                assert_eq!(lo, (wide as u8) ^ 0x5A);
                assert_eq!(hi, (wide >> 8) as u8);
            }
        }
        // u128 Karatsuba: agree with a direct 256-bit shift-xor reference
        fn reference(a: u128, b: u128) -> (u128, u128) {
            let (mut lo, mut hi) = (0u128, 0u128);
            for i in 0..128u32 {
                if (b >> i) & 1 == 1 {
                    lo ^= a.wrapping_shl(i);
                    if i > 0 {
                        hi ^= a >> (128 - i);
                    }
                }
            }
            (lo, hi)
        }
        let vals = [
            0u128,
            1,
            u128::MAX,
            0x0123_4567_89ab_cdef_fedc_ba98_7654_3210,
            1 << 127,
            0xdead_beef,
        ];
        for &a in &vals {
            for &b in &vals {
                let (lo, hi) = CarryingCarrylessMul::carrying_carryless_mul(a, b, 0);
                assert_eq!((lo, hi), reference(a, b), "{a:#x} {b:#x}");
            }
        }
        // carry is XOR'd into the low word only
        let (lo0, hi0) = CarryingCarrylessMul::carrying_carryless_mul(u128::MAX, u128::MAX, 0);
        let (lo1, hi1) =
            CarryingCarrylessMul::carrying_carryless_mul(u128::MAX, u128::MAX, 0xFF);
        assert_eq!((lo0 ^ 0xFF, hi0), (lo1, hi1));
    }

    #[test]
    fn carryless() {
        assert_eq!(CarrylessMul::carryless_mul(0b11u8, 0b11), 0b101);
        assert_eq!(CarrylessMul::carryless_mul(0x80u8, 0x80), 0); // truncated
        assert_eq!(
            WideningCarrylessMul::widening_carryless_mul(0x80u8, 0x80),
            0x4000u16
        );
        // clmul by a power of two is a plain shift
        assert_eq!(CarrylessMul::carryless_mul(0b1011u32, 0b100), 0b101100);
    }
}
