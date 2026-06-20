# std primitive method coverage

Generated from the nightly `rust-docs-json` component (rustdoc JSON format 57, toolchain 2026-05-23) by extracting every inherent method on the primitives and matching it against this crate's trait surface by method name.

Statuses: ✅ covered (trait shown) · 🔁 expressible via the generic cast traits (variant deliberately not mirrored, DESIGN.md decision 6) · 🚫 skipped by design (`unsafe`) · ❌ gap.


## Integer methods (`u32` ∪ `i32`, core)

| method | status | maps to |
|---|---|---|
| `abs` | ✅ | `FloatCore` |
| `abs_diff` | ✅ | `AbsDiff` |
| `bit_width` | ✅ | `BitWidth` |
| `borrowing_sub` | ✅ | `BorrowingSub` |
| `carrying_add` | ✅ | `CarryingAdd` |
| `carrying_carryless_mul` | ✅ | `CarryingCarrylessMul` |
| `carrying_mul` | ✅ | `CarryingMul` |
| `carrying_mul_add` | ✅ | `CarryingMul` |
| `carryless_mul` | ✅ | `CarrylessMul` |
| `cast_signed` | ✅ | `CastSigned` |
| `cast_unsigned` | ✅ | `CastUnsigned` |
| `checked_abs` | ✅ | `CheckedAbs` |
| `checked_add` | ✅ | `CheckedAdd` |
| `checked_add_signed` | ✅ | `?` |
| `checked_add_unsigned` | ✅ | `?` |
| `checked_cast_signed` | 🔁 | CheckedCast<iN> |
| `checked_cast_unsigned` | 🔁 | CheckedCast<uN> |
| `checked_div` | ✅ | `CheckedDiv` |
| `checked_div_euclid` | ✅ | `CheckedEuclid` |
| `checked_div_exact` | ✅ | `DivExact` |
| `checked_ilog` | ✅ | `Ilog` |
| `checked_ilog10` | ✅ | `Ilog10` |
| `checked_ilog2` | ✅ | `Ilog2` |
| `checked_isqrt` | ✅ | `CheckedIsqrt` |
| `checked_mul` | ✅ | `CheckedMul` |
| `checked_neg` | ✅ | `CheckedNeg` |
| `checked_next_multiple_of` | ✅ | `NextMultipleOf` |
| `checked_next_power_of_two` | ✅ | `NextPowerOfTwo` |
| `checked_pow` | ✅ | `CheckedPow` |
| `checked_rem` | ✅ | `CheckedRem` |
| `checked_rem_euclid` | ✅ | `CheckedEuclid` |
| `checked_shl` | ✅ | `CheckedShl` |
| `checked_shr` | ✅ | `CheckedShr` |
| `checked_signed_diff` | ✅ | `CheckedSignedDiff` |
| `checked_sub` | ✅ | `CheckedSub` |
| `checked_sub_signed` | ✅ | `?` |
| `checked_sub_unsigned` | ✅ | `?` |
| `checked_truncate` | 🔁 | CheckedCast<T> |
| `clamp_magnitude` | ✅ | `ClampMagnitude` |
| `count_ones` | ✅ | `PrimBits` |
| `count_zeros` | ✅ | `PrimBits` |
| `deposit_bits` | ✅ | `DepositBits` |
| `div_ceil` | ✅ | `DivCeil` |
| `div_euclid` | ✅ | `Euclid` |
| `div_exact` | ✅ | `DivExact` |
| `div_floor` | ✅ | `DivFloor` |
| `extract_bits` | ✅ | `ExtractBits` |
| `format_into` | ✅ | `FormatInto` |
| `from_ascii` | ✅ | `FromAscii` |
| `from_ascii_radix` | ✅ | `FromAscii` |
| `from_be` | ✅ | `PrimBits` |
| `from_be_bytes` | ✅ | `FromBytes` |
| `from_le` | ✅ | `PrimBits` |
| `from_le_bytes` | ✅ | `FromBytes` |
| `from_ne_bytes` | ✅ | `FromBytes` |
| `from_str_radix` | ✅ | `Num` |
| `funnel_shl` | ✅ | `FunnelShl` |
| `funnel_shr` | ✅ | `FunnelShr` |
| `highest_one` | ✅ | `HighestOne` |
| `ilog` | ✅ | `Ilog` |
| `ilog10` | ✅ | `Ilog10` |
| `ilog2` | ✅ | `Ilog2` |
| `is_multiple_of` | ✅ | `MultipleOf` |
| `is_negative` | ✅ | `Signed` |
| `is_positive` | ✅ | `Signed` |
| `is_power_of_two` | ✅ | `IsPowerOfTwo` |
| `isolate_highest_one` | ✅ | `IsolateHighestOne` |
| `isolate_lowest_one` | ✅ | `IsolateLowestOne` |
| `isqrt` | ✅ | `Isqrt` |
| `leading_ones` | ✅ | `PrimBits` |
| `leading_zeros` | ✅ | `PrimBits` |
| `lowest_one` | ✅ | `LowestOne` |
| `max_value` *(deprecated in std)* | ✅ | `Bounded` |
| `midpoint` | ✅ | `Midpoint` |
| `min_value` *(deprecated in std)* | ✅ | `Bounded` |
| `next_multiple_of` | ✅ | `NextMultipleOf` |
| `next_power_of_two` | ✅ | `NextPowerOfTwo` |
| `overflowing_abs` | ✅ | `OverflowingAbs` |
| `overflowing_add` | ✅ | `OverflowingAdd` |
| `overflowing_add_signed` | ✅ | `?` |
| `overflowing_add_unsigned` | ✅ | `?` |
| `overflowing_div` | ✅ | `OverflowingDiv` |
| `overflowing_div_euclid` | ✅ | `OverflowingEuclid` |
| `overflowing_mul` | ✅ | `OverflowingMul` |
| `overflowing_neg` | ✅ | `OverflowingNeg` |
| `overflowing_pow` | ✅ | `OverflowingPow` |
| `overflowing_rem` | ✅ | `OverflowingRem` |
| `overflowing_rem_euclid` | ✅ | `OverflowingEuclid` |
| `overflowing_shl` | ✅ | `OverflowingShl` |
| `overflowing_shr` | ✅ | `OverflowingShr` |
| `overflowing_sub` | ✅ | `OverflowingSub` |
| `overflowing_sub_signed` | ✅ | `?` |
| `overflowing_sub_unsigned` | ✅ | `?` |
| `pow` | ✅ | `PrimInt` |
| `rem_euclid` | ✅ | `Euclid` |
| `reverse_bits` | ✅ | `PrimBits` |
| `rotate_left` | ✅ | `PrimBits` |
| `rotate_right` | ✅ | `PrimBits` |
| `saturating_abs` | ✅ | `SaturatingAbs` |
| `saturating_add` | ✅ | `Saturating` |
| `saturating_add_signed` | ✅ | `?` |
| `saturating_add_unsigned` | ✅ | `?` |
| `saturating_cast_signed` | 🔁 | SaturatingCast<iN> |
| `saturating_cast_unsigned` | 🔁 | SaturatingCast<uN> |
| `saturating_div` | ✅ | `SaturatingDiv` |
| `saturating_mul` | ✅ | `SaturatingMul` |
| `saturating_neg` | ✅ | `SaturatingNeg` |
| `saturating_pow` | ✅ | `SaturatingPow` |
| `saturating_sub` | ✅ | `Saturating` |
| `saturating_sub_signed` | ✅ | `?` |
| `saturating_sub_unsigned` | ✅ | `?` |
| `saturating_truncate` | 🔁 | SaturatingCast<T> |
| `shl_exact` | ✅ | `ShlExact` |
| `shr_exact` | ✅ | `ShrExact` |
| `signum` | ✅ | `FloatCore` |
| `strict_abs` | ✅ | `StrictAbs` |
| `strict_add` | ✅ | `StrictAdd` |
| `strict_add_signed` | ✅ | `?` |
| `strict_add_unsigned` | ✅ | `?` |
| `strict_cast_signed` | 🔁 | StrictCast<iN> |
| `strict_cast_unsigned` | 🔁 | StrictCast<uN> |
| `strict_div` | ✅ | `StrictDiv` |
| `strict_div_euclid` | ✅ | `StrictEuclid` |
| `strict_mul` | ✅ | `StrictMul` |
| `strict_neg` | ✅ | `StrictNeg` |
| `strict_pow` | ✅ | `StrictPow` |
| `strict_rem` | ✅ | `StrictRem` |
| `strict_rem_euclid` | ✅ | `StrictEuclid` |
| `strict_shl` | ✅ | `StrictShl` |
| `strict_shr` | ✅ | `StrictShr` |
| `strict_sub` | ✅ | `StrictSub` |
| `strict_sub_signed` | ✅ | `?` |
| `strict_sub_unsigned` | ✅ | `?` |
| `swap_bytes` | ✅ | `PrimBits` |
| `to_be` | ✅ | `PrimBits` |
| `to_be_bytes` | ✅ | `ToBytes` |
| `to_le` | ✅ | `PrimBits` |
| `to_le_bytes` | ✅ | `ToBytes` |
| `to_ne_bytes` | ✅ | `ToBytes` |
| `trailing_ones` | ✅ | `PrimBits` |
| `trailing_zeros` | ✅ | `PrimBits` |
| `truncate` | ✅ | `Truncate` |
| `unbounded_shl` | ✅ | `UnboundedShl` |
| `unbounded_shr` | ✅ | `UnboundedShr` |
| `unchecked_add` | 🚫 |  |
| `unchecked_disjoint_bitor` | 🚫 |  |
| `unchecked_div_exact` | 🚫 |  |
| `unchecked_funnel_shl` | 🚫 |  |
| `unchecked_funnel_shr` | 🚫 |  |
| `unchecked_mul` | 🚫 |  |
| `unchecked_neg` | 🚫 |  |
| `unchecked_shl` | 🚫 |  |
| `unchecked_shl_exact` | 🚫 |  |
| `unchecked_shr` | 🚫 |  |
| `unchecked_shr_exact` | 🚫 |  |
| `unchecked_sub` | 🚫 |  |
| `unsigned_abs` | ✅ | `UnsignedAbs` |
| `widen` | ✅ | `Widen` |
| `widening_carryless_mul` | ✅ | `WideningCarrylessMul` |
| `widening_mul` | ✅ | `WideningMul` |
| `wrapping_abs` | ✅ | `WrappingAbs` |
| `wrapping_add` | ✅ | `WrappingAdd` |
| `wrapping_add_signed` | ✅ | `?` |
| `wrapping_add_unsigned` | ✅ | `?` |
| `wrapping_div` | ✅ | `WrappingDiv` |
| `wrapping_div_euclid` | ✅ | `WrappingEuclid` |
| `wrapping_mul` | ✅ | `WrappingMul` |
| `wrapping_neg` | ✅ | `WrappingNeg` |
| `wrapping_next_power_of_two` | ✅ | `NextPowerOfTwo` |
| `wrapping_pow` | ✅ | `WrappingPow` |
| `wrapping_rem` | ✅ | `WrappingRem` |
| `wrapping_rem_euclid` | ✅ | `WrappingEuclid` |
| `wrapping_shl` | ✅ | `WrappingShl` |
| `wrapping_shr` | ✅ | `WrappingShr` |
| `wrapping_sub` | ✅ | `WrappingSub` |
| `wrapping_sub_signed` | ✅ | `?` |
| `wrapping_sub_unsigned` | ✅ | `?` |

**Integer methods (`u32` ∪ `i32`, core) summary**: 177 methods — 157 covered, 12 skipped-unsafe, 8 via-generic


## Float methods (`f32`, core ∪ std)

| method | status | maps to |
|---|---|---|
| `abs` | ✅ | `FloatCore` |
| `abs_sub` *(deprecated in std)* | ✅ | `Float` |
| `acos` | ✅ | `Float` |
| `acosh` | ✅ | `Float` |
| `algebraic_add` | ✅ | `Algebraic` |
| `algebraic_div` | ✅ | `Algebraic` |
| `algebraic_mul` | ✅ | `Algebraic` |
| `algebraic_rem` | ✅ | `Algebraic` |
| `algebraic_sub` | ✅ | `Algebraic` |
| `asin` | ✅ | `Float` |
| `asinh` | ✅ | `Float` |
| `atan` | ✅ | `Float` |
| `atan2` | ✅ | `Float` |
| `atanh` | ✅ | `Float` |
| `cbrt` | ✅ | `Float` |
| `ceil` | ✅ | `FloatCore` |
| `clamp` | ✅ | `FloatCore` |
| `clamp_magnitude` | ✅ | `ClampMagnitude` |
| `classify` | ✅ | `FloatCore` |
| `copysign` | ✅ | `Float` |
| `cos` | ✅ | `Float` |
| `cosh` | ✅ | `Float` |
| `div_euclid` | ✅ | `Euclid` |
| `erf` | ✅ | `Erf` |
| `erfc` | ✅ | `Erf` |
| `exp` | ✅ | `Float` |
| `exp2` | ✅ | `Float` |
| `exp_m1` | ✅ | `Float` |
| `floor` | ✅ | `FloatCore` |
| `fract` | ✅ | `FloatCore` |
| `from_be_bytes` | ✅ | `FromBytes` |
| `from_bits` | ✅ | `FloatBits` |
| `from_le_bytes` | ✅ | `FromBytes` |
| `from_ne_bytes` | ✅ | `FromBytes` |
| `gamma` | ✅ | `Gamma` |
| `hypot` | ✅ | `Float` |
| `is_finite` | ✅ | `FloatCore` |
| `is_infinite` | ✅ | `FloatCore` |
| `is_nan` | ✅ | `FloatCore` |
| `is_normal` | ✅ | `FloatCore` |
| `is_sign_negative` | ✅ | `FloatCore` |
| `is_sign_positive` | ✅ | `FloatCore` |
| `is_subnormal` | ✅ | `FloatCore` |
| `ln` | ✅ | `Float` |
| `ln_1p` | ✅ | `Float` |
| `ln_gamma` | ✅ | `Gamma` |
| `log` | ✅ | `Float` |
| `log10` | ✅ | `Float` |
| `log2` | ✅ | `Float` |
| `max` | ✅ | `FloatCore` |
| `maximum` | ✅ | `Maximum` |
| `midpoint` | ✅ | `Midpoint` |
| `min` | ✅ | `FloatCore` |
| `minimum` | ✅ | `Minimum` |
| `mul_add` | ✅ | `Float` |
| `next_down` | ✅ | `NextDown` |
| `next_up` | ✅ | `NextUp` |
| `powf` | ✅ | `Float` |
| `powi` | ✅ | `FloatCore` |
| `recip` | ✅ | `FloatCore` |
| `rem_euclid` | ✅ | `Euclid` |
| `round` | ✅ | `FloatCore` |
| `round_ties_even` | ✅ | `RoundTiesEven` |
| `signum` | ✅ | `FloatCore` |
| `sin` | ✅ | `Float` |
| `sin_cos` | ✅ | `Float` |
| `sinh` | ✅ | `Float` |
| `sqrt` | ✅ | `Float` |
| `tan` | ✅ | `Float` |
| `tanh` | ✅ | `Float` |
| `to_be_bytes` | ✅ | `ToBytes` |
| `to_bits` | ✅ | `FloatBits` |
| `to_degrees` | ✅ | `FloatCore` |
| `to_int_unchecked` | 🚫 |  |
| `to_le_bytes` | ✅ | `ToBytes` |
| `to_ne_bytes` | ✅ | `ToBytes` |
| `to_radians` | ✅ | `FloatCore` |
| `total_cmp` | ✅ | `TotalOrder` |
| `trunc` | ✅ | `FloatCore` |

**Float methods (`f32`, core ∪ std) summary**: 79 methods — 78 covered, 1 skipped-unsafe


## Notes

- Integer list is the union of `u32` and `i32`; sign-restricted methods are covered by the corresponding sign-restricted trait impls.
- The newest upstream-HEAD generic casts (`checked_cast`, `wrapping_cast`, `saturating_cast`, `strict_cast`, `widen`, `truncate`) are not in this nightly JSON snapshot yet but are covered by the generic cast traits.
- `FromAscii` parses byte strings into a crate-owned `AsciiParseError` (std's `ParseIntError` is unconstructible), exists on stable and nightly identically, and is const-callable on nightly.
- `FormatInto` mirrors `core`'s unstable inherent `format_into` *exactly* — same `format_into(self, &mut NumBuffer<Self>) -> &str` signature, re-exporting the unstable `NumBuffer`/`NumBufferTrait`. It is gated behind the **`nightly-std`** feature (= `nightly` + `feature(int_format_into)`) because `NumBuffer` cannot be named on stable; plain `nightly` (const traits) does not pull it. It is a plain (non-const) trait, since the inherent method is not `const fn` even on nightly. Code written against std's `format_into` ports verbatim.
- `Erf`/`Gamma` impls require the `libm` cargo feature (std's versions are unstable). `Algebraic` impls perform the plain IEEE operation — a conforming result — until std's intrinsics stabilize.
