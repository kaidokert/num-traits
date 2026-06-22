//! Bit-manipulation operations beyond `PrimInt`: unbounded and funnel
//! shifts, exact (lossless) shifts, bit isolation/indexing, bit width and
//! PDEP/PEXT-style bit deposit/extract, mirroring the corresponding inherent
//! methods on the primitive integer types.
//!
//! Stability in std (as of nightly 2026): `unbounded_shl`/`unbounded_shr`
//! are stable since 1.87; `highest_one`/`lowest_one`/`isolate_highest_one`/
//! `isolate_lowest_one`/`bit_width` since 1.98; funnel shifts, exact shifts
//! and `deposit_bits`/`extract_bits` are still nightly-only. Everything
//! newer than the crate's MSRV is hand-rolled with the same semantics.
//!
//! **CT tiers**: [`IsolateHighestOne`]/[`IsolateLowestOne`], [`BitWidth`],
//! [`DepositBits`]/[`ExtractBits`] and the funnel/unbounded shifts (under
//! the public-parameter convention for shift amounts) are Tier A;
//! [`HighestOne`]/[`LowestOne`] and [`ShlExact`]/[`ShrExact`] are Tier B
//! (`Option` returns).

use core::ops::{Shl, Shr};

c0nst::c0nst! {
/// Performs a left shift that never panics, returning 0 for large shifts.
pub c0nst trait UnboundedShl: Sized + [c0nst] Shl<u32> {
    /// Unbounded shift left. Computes `self << rhs`, without bounding the
    /// value of `rhs`: if `rhs >= BITS` the entire value is shifted out and
    /// 0 is returned.
    ///
    /// ```
    /// use const_num_traits::UnboundedShl;
    ///
    /// assert_eq!(UnboundedShl::unbounded_shl(1u8, 4), 16);
    /// assert_eq!(UnboundedShl::unbounded_shl(1u8, 200), 0);
    /// ```
    fn unbounded_shl(self, rhs: u32) -> <Self as Shl<u32>>::Output;
}
}

c0nst::c0nst! {
/// Performs a right shift that never panics, shifting in zero or sign bits
/// for large shift amounts.
pub c0nst trait UnboundedShr: Sized + [c0nst] Shr<u32> {
    /// Unbounded shift right. Computes `self >> rhs`, without bounding the
    /// value of `rhs`: if `rhs >= BITS`, unsigned values become 0 and signed
    /// values become 0 or -1 depending on the sign (the sign bit fills every
    /// position).
    ///
    /// ```
    /// use const_num_traits::UnboundedShr;
    ///
    /// assert_eq!(UnboundedShr::unbounded_shr(16u8, 4), 1);
    /// assert_eq!(UnboundedShr::unbounded_shr(16u8, 200), 0);
    /// assert_eq!(UnboundedShr::unbounded_shr(-16i8, 200), -1);
    /// ```
    fn unbounded_shr(self, rhs: u32) -> <Self as Shr<u32>>::Output;
}
}

macro_rules! unbounded_shift_impl {
    (unsigned $($t:ty)*) => {$(
        c0nst::c0nst! {
        c0nst impl UnboundedShl for $t {
            #[inline]
            fn unbounded_shl(self, rhs: u32) -> Self {
                if rhs < <$t>::BITS { self << rhs } else { 0 }
            }
        }
        }
        c0nst::c0nst! {
        c0nst impl UnboundedShr for $t {
            #[inline]
            fn unbounded_shr(self, rhs: u32) -> Self {
                if rhs < <$t>::BITS { self >> rhs } else { 0 }
            }
        }
        }
    )*};
    (signed $($t:ty)*) => {$(
        c0nst::c0nst! {
        c0nst impl UnboundedShl for $t {
            #[inline]
            fn unbounded_shl(self, rhs: u32) -> Self {
                if rhs < <$t>::BITS { self << rhs } else { 0 }
            }
        }
        }
        c0nst::c0nst! {
        c0nst impl UnboundedShr for $t {
            #[inline]
            fn unbounded_shr(self, rhs: u32) -> Self {
                if rhs < <$t>::BITS {
                    self >> rhs
                } else {
                    // shifting by BITS-1 copies the sign bit everywhere
                    self >> (<$t>::BITS - 1)
                }
            }
        }
        }
    )*};
}

unbounded_shift_impl!(unsigned usize u8 u16 u32 u64 u128);
unbounded_shift_impl!(signed isize i8 i16 i32 i64 i128);

c0nst::c0nst! {
/// Performs a funnel left shift on a double-width value formed from two words.
pub c0nst trait FunnelShl: Sized {
    /// Funnel shift left: concatenates `self` (high word) with `rhs` (low
    /// word), shifts the combination left by `n`, and returns the high word
    /// — i.e. `(self << n) | (rhs >> (BITS - n))`. Like std, this is only
    /// provided for unsigned types.
    ///
    /// # Panics
    ///
    /// Panics if `n >= BITS`.
    ///
    /// ```
    /// use const_num_traits::FunnelShl;
    ///
    /// assert_eq!(FunnelShl::funnel_shl(0x01u8, 0x80, 1), 0x03);
    /// ```
    type Output;
    fn funnel_shl(self, rhs: Self, n: u32) -> Self::Output;
}
}

c0nst::c0nst! {
/// Performs a funnel right shift on a double-width value formed from two words.
pub c0nst trait FunnelShr: Sized {
    /// Funnel shift right: concatenates `self` (high word) with `rhs` (low
    /// word), shifts the combination right by `n`, and returns the low word
    /// — i.e. `(rhs >> n) | (self << (BITS - n))`. Like std, this is only
    /// provided for unsigned types.
    ///
    /// # Panics
    ///
    /// Panics if `n >= BITS`.
    ///
    /// ```
    /// use const_num_traits::FunnelShr;
    ///
    /// assert_eq!(FunnelShr::funnel_shr(0x01u8, 0x80, 1), 0xC0);
    /// ```
    type Output;
    fn funnel_shr(self, rhs: Self, n: u32) -> Self::Output;
}
}

macro_rules! funnel_shift_impl {
    ($($t:ty)*) => {$(
        c0nst::c0nst! {
        c0nst impl FunnelShl for $t {
            type Output = $t;
            #[inline]
            #[track_caller]
            fn funnel_shl(self, rhs: Self, n: u32) -> Self {
                assert!(n < <$t>::BITS, "attempt to funnel shift left with overflow");
                if n == 0 {
                    self
                } else {
                    (self << n) | (rhs >> (<$t>::BITS - n))
                }
            }
        }
        }
        c0nst::c0nst! {
        c0nst impl FunnelShr for $t {
            type Output = $t;
            #[inline]
            #[track_caller]
            fn funnel_shr(self, rhs: Self, n: u32) -> Self {
                assert!(n < <$t>::BITS, "attempt to funnel shift right with overflow");
                if n == 0 {
                    rhs
                } else {
                    (rhs >> n) | (self << (<$t>::BITS - n))
                }
            }
        }
        }
    )*};
}

funnel_shift_impl!(usize u8 u16 u32 u64 u128);

c0nst::c0nst! {
/// Performs a lossless (exactly reversible) left shift.
pub c0nst trait ShlExact: Sized + [c0nst] Shl<u32> {
    /// Exact shift left. Computes `self << rhs` if no bits would be shifted
    /// out (so the operation can be losslessly reversed), `None` otherwise.
    ///
    /// ```
    /// use const_num_traits::ShlExact;
    ///
    /// assert_eq!(ShlExact::shl_exact(0x11u8, 3), Some(0x88));
    /// assert_eq!(ShlExact::shl_exact(0x11u8, 4), None);
    /// ```
    fn shl_exact(self, rhs: u32) -> Option<<Self as Shl<u32>>::Output>;
}
}

c0nst::c0nst! {
/// Performs a lossless (exactly reversible) right shift.
pub c0nst trait ShrExact: Sized + [c0nst] Shr<u32> {
    /// Exact shift right. Computes `self >> rhs` if no one-bits would be
    /// shifted out (so the operation can be losslessly reversed), `None`
    /// otherwise.
    ///
    /// ```
    /// use const_num_traits::ShrExact;
    ///
    /// assert_eq!(ShrExact::shr_exact(0x88u8, 3), Some(0x11));
    /// assert_eq!(ShrExact::shr_exact(0x88u8, 4), None);
    /// ```
    fn shr_exact(self, rhs: u32) -> Option<<Self as Shr<u32>>::Output>;
}
}

macro_rules! exact_shift_impl {
    (unsigned $($t:ty)*) => {$(
        c0nst::c0nst! {
        c0nst impl ShlExact for $t {
            #[inline]
            fn shl_exact(self, rhs: u32) -> Option<Self> {
                if rhs <= <$t>::leading_zeros(self) && rhs < <$t>::BITS {
                    Some(self << rhs)
                } else {
                    None
                }
            }
        }
        }
        exact_shift_impl!(@shr $t);
    )*};
    (signed $($t:ty)*) => {$(
        c0nst::c0nst! {
        c0nst impl ShlExact for $t {
            #[inline]
            fn shl_exact(self, rhs: u32) -> Option<Self> {
                // for negative values the sign-extension bits are the
                // recoverable ones, hence leading_ones
                if rhs < <$t>::leading_zeros(self) || rhs < <$t>::leading_ones(self) {
                    Some(self << rhs)
                } else {
                    None
                }
            }
        }
        }
        exact_shift_impl!(@shr $t);
    )*};
    (@shr $t:ty) => {
        c0nst::c0nst! {
        c0nst impl ShrExact for $t {
            #[inline]
            fn shr_exact(self, rhs: u32) -> Option<Self> {
                if rhs <= <$t>::trailing_zeros(self) && rhs < <$t>::BITS {
                    Some(self >> rhs)
                } else {
                    None
                }
            }
        }
        }
    };
}

exact_shift_impl!(unsigned usize u8 u16 u32 u64 u128);
exact_shift_impl!(signed isize i8 i16 i32 i64 i128);

c0nst::c0nst! {
/// Finds the index of the highest one-bit.
pub c0nst trait HighestOne: Sized {
    /// Returns the index of the highest bit set to one, or `None` if the
    /// value is zero.
    ///
    /// ```
    /// use const_num_traits::HighestOne;
    ///
    /// assert_eq!(HighestOne::highest_one(0b0101_0000u8), Some(6));
    /// assert_eq!(HighestOne::highest_one(0u8), None);
    /// ```
    fn highest_one(self) -> Option<u32>;
}
}

c0nst::c0nst! {
/// Finds the index of the lowest one-bit.
pub c0nst trait LowestOne: Sized {
    /// Returns the index of the lowest bit set to one, or `None` if the
    /// value is zero.
    ///
    /// ```
    /// use const_num_traits::LowestOne;
    ///
    /// assert_eq!(LowestOne::lowest_one(0b0101_0000u8), Some(4));
    /// assert_eq!(LowestOne::lowest_one(0u8), None);
    /// ```
    fn lowest_one(self) -> Option<u32>;
}
}

c0nst::c0nst! {
/// Isolates the highest one-bit, branchlessly.
pub c0nst trait IsolateHighestOne: Sized {
    /// Returns `self` with only its highest one-bit kept, or 0 if the value
    /// is zero.
    ///
    /// ```
    /// use const_num_traits::IsolateHighestOne;
    ///
    /// assert_eq!(IsolateHighestOne::isolate_highest_one(0b0101_0000u8), 0b0100_0000);
    /// ```
    type Output;
    fn isolate_highest_one(self) -> Self::Output;
}
}

c0nst::c0nst! {
/// Isolates the lowest one-bit, branchlessly.
pub c0nst trait IsolateLowestOne: Sized {
    /// Returns `self` with only its lowest one-bit kept, or 0 if the value
    /// is zero.
    ///
    /// ```
    /// use const_num_traits::IsolateLowestOne;
    ///
    /// assert_eq!(IsolateLowestOne::isolate_lowest_one(0b0101_0000u8), 0b0001_0000);
    /// ```
    type Output;
    fn isolate_lowest_one(self) -> Self::Output;
}
}

macro_rules! isolate_one_impl {
    // operate on the unsigned bit pattern; `$u` is `$t` itself for the
    // unsigned instantiations
    ($($t:ty => $u:ty;)*) => {$(
        c0nst::c0nst! {
        c0nst impl HighestOne for $t {
            #[inline]
            fn highest_one(self) -> Option<u32> {
                if self == 0 {
                    None
                } else {
                    Some(<$t>::BITS - 1 - <$t>::leading_zeros(self))
                }
            }
        }
        }

        c0nst::c0nst! {
        c0nst impl LowestOne for $t {
            #[inline]
            fn lowest_one(self) -> Option<u32> {
                if self == 0 {
                    None
                } else {
                    Some(<$t>::trailing_zeros(self))
                }
            }
        }
        }

        c0nst::c0nst! {
        c0nst impl IsolateHighestOne for $t {
            type Output = $t;
            #[inline]
            fn isolate_highest_one(self) -> Self {
                let bits = self as $u;
                (bits & ((1 as $u) << (<$u>::BITS - 1)).wrapping_shr(<$u>::leading_zeros(bits))) as $t
            }
        }
        }

        c0nst::c0nst! {
        c0nst impl IsolateLowestOne for $t {
            type Output = $t;
            #[inline]
            fn isolate_lowest_one(self) -> Self {
                self & <$t>::wrapping_neg(self)
            }
        }
        }
    )*};
}

isolate_one_impl! {
    u8 => u8; u16 => u16; u32 => u32; u64 => u64; usize => usize; u128 => u128;
    i8 => u8; i16 => u16; i32 => u32; i64 => u64; isize => usize; i128 => u128;
}

c0nst::c0nst! {
/// Computes the minimal number of bits required to represent an unsigned value.
pub c0nst trait BitWidth: Sized {
    /// Returns the minimum number of bits required to represent `self`,
    /// i.e. `BITS - leading_zeros`. Returns 0 for 0. Like std, this is only
    /// provided for unsigned types.
    ///
    /// ```
    /// use const_num_traits::BitWidth;
    ///
    /// assert_eq!(BitWidth::bit_width(0u8), 0);
    /// assert_eq!(BitWidth::bit_width(0b0101_0000u8), 7);
    /// ```
    fn bit_width(self) -> u32;
}
}

macro_rules! bit_width_impl {
    ($($t:ty)*) => {$(
        c0nst::c0nst! {
        c0nst impl BitWidth for $t {
            #[inline]
            fn bit_width(self) -> u32 {
                <$t>::BITS - <$t>::leading_zeros(self)
            }
        }
        }
    )*};
}

bit_width_impl!(usize u8 u16 u32 u64 u128);

c0nst::c0nst! {
/// Scatters bits through a mask (the PDEP operation).
pub c0nst trait DepositBits: Sized {
    /// Scatters the contiguous low-order bits of `self` into the positions
    /// of the one-bits of `mask` (the PDEP operation). All other bits of the
    /// result are zero. Like std, this is only provided for unsigned types.
    ///
    /// ```
    /// use const_num_traits::DepositBits;
    ///
    /// assert_eq!(DepositBits::deposit_bits(0b101u8, 0b1111_0000), 0b0101_0000);
    /// ```
    type Output;
    fn deposit_bits(self, mask: Self) -> Self::Output;
}
}

c0nst::c0nst! {
/// Gathers bits through a mask (the PEXT operation).
pub c0nst trait ExtractBits: Sized {
    /// Gathers the bits of `self` selected by the one-bits of `mask` into
    /// the contiguous low-order bits of the result (the PEXT operation).
    /// Like std, this is only provided for unsigned types.
    ///
    /// ```
    /// use const_num_traits::ExtractBits;
    ///
    /// assert_eq!(ExtractBits::extract_bits(0b0101_0011u8, 0b1111_0000), 0b101);
    /// ```
    type Output;
    fn extract_bits(self, mask: Self) -> Self::Output;
}
}

macro_rules! deposit_extract_impl {
    ($($t:ty)*) => {$(
        c0nst::c0nst! {
        c0nst impl DepositBits for $t {
            type Output = $t;
            #[inline]
            fn deposit_bits(self, mask: Self) -> Self {
                let mut result: $t = 0;
                let mut remaining = mask;
                let mut bb: $t = 1;
                while remaining != 0 {
                    let lowest = remaining & <$t>::wrapping_neg(remaining);
                    if self & bb != 0 {
                        result |= lowest;
                    }
                    remaining &= remaining - 1;
                    bb = <$t>::wrapping_shl(bb, 1);
                }
                result
            }
        }
        }

        c0nst::c0nst! {
        c0nst impl ExtractBits for $t {
            type Output = $t;
            #[inline]
            fn extract_bits(self, mask: Self) -> Self {
                let mut result: $t = 0;
                let mut remaining = mask;
                let mut bb: $t = 1;
                while remaining != 0 {
                    let lowest = remaining & <$t>::wrapping_neg(remaining);
                    if self & lowest != 0 {
                        result |= bb;
                    }
                    remaining &= remaining - 1;
                    bb = <$t>::wrapping_shl(bb, 1);
                }
                result
            }
        }
        }
    )*};
}

deposit_extract_impl!(usize u8 u16 u32 u64 u128);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unbounded_shifts() {
        assert_eq!(UnboundedShl::unbounded_shl(1u8, 7), 0x80);
        assert_eq!(UnboundedShl::unbounded_shl(1u8, 8), 0);
        assert_eq!(UnboundedShr::unbounded_shr(0x80u8, 8), 0);
        assert_eq!(UnboundedShr::unbounded_shr(-1i8, 100), -1);
        assert_eq!(UnboundedShr::unbounded_shr(i8::MAX, 100), 0);
    }

    #[test]
    fn funnel_shifts() {
        // 0x0180 << 1 = 0x0300 -> high byte 0x03
        assert_eq!(FunnelShl::funnel_shl(0x01u8, 0x80, 1), 0x03);
        assert_eq!(FunnelShl::funnel_shl(0xABu8, 0xCD, 0), 0xAB);
        // 0x0180 >> 1 = 0x00C0 -> low byte 0xC0
        assert_eq!(FunnelShr::funnel_shr(0x01u8, 0x80, 1), 0xC0);
        assert_eq!(FunnelShr::funnel_shr(0xABu8, 0xCD, 0), 0xCD);
        // rotation is a funnel shift with both words equal
        assert_eq!(FunnelShl::funnel_shl(0x81u8, 0x81, 1), 0x81u8.rotate_left(1));
    }

    #[test]
    #[should_panic(expected = "attempt to funnel shift left with overflow")]
    fn funnel_shl_panics() {
        let _ = FunnelShl::funnel_shl(1u8, 1, 8);
    }

    #[test]
    fn exact_shifts() {
        assert_eq!(ShlExact::shl_exact(0x11u8, 3), Some(0x88));
        assert_eq!(ShlExact::shl_exact(0x11u8, 4), None);
        assert_eq!(ShrExact::shr_exact(0x88u8, 3), Some(0x11));
        assert_eq!(ShrExact::shr_exact(0x88u8, 4), None);
        assert_eq!(ShrExact::shr_exact(0u8, 7), Some(0));
        assert_eq!(ShrExact::shr_exact(0u8, 8), None);
        // negative values: sign bits are recoverable
        assert_eq!(ShlExact::shl_exact(-1i8, 7), Some(i8::MIN));
        assert_eq!(ShlExact::shl_exact(-2i8, 6), Some(i8::MIN));
        assert_eq!(ShlExact::shl_exact(-2i8, 7), None);
        assert_eq!(ShlExact::shl_exact(1i8, 6), Some(64));
        assert_eq!(ShlExact::shl_exact(1i8, 7), None);
    }

    #[test]
    fn isolate() {
        assert_eq!(HighestOne::highest_one(0b0101_0000u8), Some(6));
        assert_eq!(HighestOne::highest_one(0u8), None);
        assert_eq!(LowestOne::lowest_one(0b0101_0000u8), Some(4));
        assert_eq!(LowestOne::lowest_one(0i64), None);
        assert_eq!(IsolateHighestOne::isolate_highest_one(0b0101_0000u8), 0b0100_0000);
        assert_eq!(IsolateLowestOne::isolate_lowest_one(0b0101_0000u8), 0b0001_0000);
        assert_eq!(IsolateHighestOne::isolate_highest_one(0u8), 0);
        assert_eq!(IsolateLowestOne::isolate_lowest_one(0u8), 0);
        // signed: operates on the bit pattern
        assert_eq!(HighestOne::highest_one(-1i8), Some(7));
        assert_eq!(IsolateHighestOne::isolate_highest_one(-1i8), i8::MIN);
        assert_eq!(IsolateLowestOne::isolate_lowest_one(-2i8), 2);
    }

    #[test]
    fn bit_width() {
        assert_eq!(BitWidth::bit_width(0u8), 0);
        assert_eq!(BitWidth::bit_width(1u8), 1);
        assert_eq!(BitWidth::bit_width(255u8), 8);
        assert_eq!(BitWidth::bit_width(0x0101u16), 9);
    }

    #[test]
    fn deposit_extract() {
        assert_eq!(DepositBits::deposit_bits(0b101u8, 0b1111_0000), 0b0101_0000);
        assert_eq!(DepositBits::deposit_bits(0xFFu8, 0b1010_1010), 0b1010_1010);
        assert_eq!(ExtractBits::extract_bits(0b0101_0011u8, 0b1111_0000), 0b101);
        assert_eq!(ExtractBits::extract_bits(0xFFu8, 0b1010_1010), 0b1111);
        // extract is the inverse of deposit on the masked bits
        let mask = 0b0110_1001u8;
        for x in 0u8..16 {
            let deposited = DepositBits::deposit_bits(x, mask);
            assert_eq!(deposited & !mask, 0);
            assert_eq!(ExtractBits::extract_bits(deposited, mask), x);
        }
    }
}
