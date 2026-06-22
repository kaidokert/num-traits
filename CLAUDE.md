# const-num-traits

A fork of [`num-traits`](https://github.com/rust-num/num-traits) that ports the trait surface to const traits via the [`c0nst`](https://crates.io/crates/c0nst) macro crate. Compiles to byte-identical upstream behavior on stable Rust; gains `const trait` / `impl const Foo for T` on nightly with the `nightly` feature.

Upstream baseline: commit `fbdc29a` ("Merge PR #337"). Our work lives in a single commit `Fork to const-num-traits` on branch `pr1`.

## How `c0nst::c0nst!` works

The macro is a passthrough on stable. On nightly + `--features nightly` (which forwards to `c0nst/nightly`), it rewrites:

| Source token | Stable expansion | Nightly expansion |
|---|---|---|
| `pub c0nst trait X` | `pub trait X` | `pub const trait X` |
| `impl c0nst Foo for T` | `impl Foo for T` | `impl const Foo for T` |
| `[c0nst] Bar` (in a bound) | `Bar` | `[const] Bar` |
| `[c0nst] X + [c0nst] Destruct` | `X` *(the whole `+ [c0nst] Destruct` is dropped)* | `[const] X + [const] core::marker::Destruct` |
| `pub c0nst fn foo<...>(...) -> T` | `pub fn foo<...>(...) -> T` | `pub const fn foo<...>(...) -> T` |

So on stable the source reads as plain upstream Rust; on nightly the same source produces real const traits and `~const`-style bounds. The nightly features required are pinned in `src/lib.rs:#![cfg_attr(feature = "nightly", feature(const_trait_impl, const_ops, const_cmp, const_destruct))]`.

### Invariant: `nightly` never changes a *stable item's* signature

The hard rule: for any item that exists on **both** toolchains, the `nightly` feature changes only whether it is `const`-callable — never its signature or whether it exists. A caller's code against the stable surface compiles identically on nightly; nightly just additionally permits `const` use. Concretely, an item is either:

1. **Present on both, const-ness only differs** — the overwhelming majority. The `c0nst!` macro handles this (passthrough on stable, const on nightly). Such items are *never* `#[cfg(feature = "nightly")]`-gated.
2. **Mirror-only, no stable counterpart** — a deliberately additive item that names an unstable std type which cannot exist on stable. Allowed *because there is no stable signature to diverge from*. Gated behind the **`nightly-std`** cargo feature (= `nightly` + the `int_format_into` lang feature), kept separate from plain `nightly` so const-traits-only builds aren't exposed to `int_format_into` churn. Currently exactly one member: `ops::format_into::FormatInto` (mirrors `core`'s `format_into` exactly, re-exporting the unstable `NumBuffer`/`NumBufferTrait`). To add another such mirror: gate its module + re-exports on `feature = "nightly-std"` and add its lang feature to the `#![cfg_attr(feature = "nightly-std", feature(...))]` line in `lib.rs`.

What you must NOT do: take an item that *could* exist on stable and give it a different (or nightly-only) signature to match unstable std — that's the category-1 violation. Example: `from_ascii` could not return std's unconstructible `ParseIntError`, so it owns `AsciiParseError` and exists identically on both, rather than becoming nightly-only. Reach for category 2 only when the std type is genuinely unnameable on stable AND a crate-owned parallel would be strictly worse than the exact mirror.

## What is const, what isn't, and why

### Const traits

Upstream-inherited: `Zero`, `One`, `Bounded`, `LowerBounded`, `UpperBounded`, `ToBytes`, `FromBytes`, `NumBytes`, `NumOps`, `NumAssignOps`, `NumRef`, `RefNum`, `NumAssign`, `NumAssignRef`, `Num`, `PrimInt`, `MulAdd`, `MulAddAssign`, `ToPrimitive`, `FromPrimitive`, `NumCast`, `AsPrimitive`, `CheckedAdd/Sub/Mul/Div/Rem/Neg/Shl/Shr`, `WrappingAdd/Sub/Mul/Neg/Shl/Shr`, `Saturating` (deprecated), `SaturatingAdd/Sub/Mul`, `OverflowingAdd/Sub/Mul`, `Euclid`, `CheckedEuclid`, `Inv`, `Signed`, `Unsigned`.

Fork-added (the "complete nightly std coverage" pass, ~45 traits — see *New ops coverage policy* below):

- Variant-matrix completion: `CheckedAbs/Pow`, `WrappingDiv/Rem/Abs/Pow`, `SaturatingDiv/Neg/Abs/Pow`, `OverflowingDiv/Rem/Neg/Abs/Shl/Shr/Pow`, `WrappingEuclid`, `OverflowingEuclid` (in the existing `ops/{checked,wrapping,saturating,overflowing,euclid}.rs`).
- `ops/strict.rs`: `StrictAdd/Sub/Mul/Div/Rem/Neg/Abs/Shl/Shr/Pow`, `StrictEuclid` — panic-on-overflow with std's exact panic messages.
- `ops/bigint.rs`: `CarryingAdd`, `BorrowingSub`, `CarryingMul` (assoc `type Unsigned` low word, std's signed signature), `WideningMul` (assoc `type Wide`, single-wide `-> $WideT` matching core trunk per rust#156644 — `u8`–`u64`/`i8`–`i64` only; bignums/`u128` use `CarryingMul`), `CarrylessMul`, `WideningCarrylessMul`, `CarryingCarrylessMul` (full clmul + XOR carry, all unsigned incl. u128 via Karatsuba over three 64×64 clmuls — tested against a 256-bit shift-xor reference). `wide_mul_u128` is a plain `pub(crate) const fn` (128-bit limb multiply, ported from core's intrinsic fallback).
- `ops/rounding.rs`: `DivCeil`, `DivFloor`, `DivExact` (`div_exact` + `checked_div_exact`), `MultipleOf` (`is_multiple_of`, unsigned-only like std), `NextMultipleOf` (+checked), `Midpoint`.
- `ops/log.rs` / `ops/sqrt.rs` / `ops/pow2.rs`: `Ilog2`/`Ilog10`/`Ilog` (split per base — `ilog2` is cheap/CT-friendly, the others need division; each pairs `{op, checked_op}`), `Isqrt` (all 12) + `CheckedIsqrt` (signed-only, exact std mirror), `IsPowerOfTwo` + `NextPowerOfTwo` (next/checked_next/wrapping_next, unsigned-only).
- `ops/bits.rs`: `UnboundedShl/Shr`, `FunnelShl/Shr` (unsigned-only, LLVM fshl/fshr semantics, panics on `n >= BITS`), `ShlExact/ShrExact` (lossless-or-`None`), `HighestOne`/`LowestOne` (→ `Option<u32>`) + `IsolateHighestOne`/`IsolateLowestOne` (branchless, `Self→Self`), `BitWidth` (unsigned), `DepositBits`/`ExtractBits` (PDEP/PEXT, unsigned, portable loop not core's butterfly).
- `ops/mixed.rs`: 20 one-method flavor traits — `{Checked,Wrapping,Overflowing,Saturating,Strict}{AddSigned,SubSigned}` (on unsigned, assoc `type Signed`) and `…{AddUnsigned,SubUnsigned}` (on signed, assoc `type Unsigned`) — plus `CheckedSignedDiff`. Declared via the `declare_mixed_trait!` macro.
- `ops/format_into.rs`: `FormatInto` (`format_into(self, &mut NumBuffer<Self>) -> &str`) — exact mirror of `core`'s unstable inherent method, re-exporting `NumBuffer`/`NumBufferTrait`. Gated on the **`nightly-std`** feature; plain (non-const) trait since std's isn't const. This is the one category-2 item (see the `nightly` invariant above). A prior crate-owned stable `&mut [u8]` + `MAX_STR_LEN` design was rejected in favor of exact mirroring per maintainer call.
- `ops/from_ascii.rs`: `FromAscii` (`from_ascii`/`from_ascii_radix` from `&[u8]`, own `AsciiParseError` since std's `ParseIntError` is unconstructible; hand-rolled const parser — unlike unstable std, **const-callable on nightly**; avoids pulling UTF-8 machinery into embedded binaries).
- `ops/float_ops.rs`: the first `Float`-decomposition slice (P5) — `FloatBits` (`type Bits`, to/from_bits, const), `NextUp`/`NextDown` (const, delegate at 1.86), `Maximum`/`Minimum` (IEEE 754-2019 NaN-propagating, hand-rolled const — std's unstable), `RoundTiesEven` (std delegate / no_std FloatCore fallback, plain), `Algebraic` (plain-IEEE conforming fallback until std's fast-math intrinsics stabilize, const), `Erf`/`Gamma` (impls gated on `feature = "libm"`; std's are unstable so there is nothing else to delegate to).
- `ops/parity.rs`: `Parity` (`is_odd`/`is_even`) — no std counterpart (lives in num-integer's `Integer` bundle upstream); per-type const trait so implementors avoid the `BitAnd + One + PartialEq + Clone` blanket-impl bounds modmath uses today (PartialEq is the CT-hostile one) and bigints can answer from the lowest limb.
- `ops/convert.rs`: `AbsDiff` (assoc `type Output` — unsigned counterpart for signed types), `UnsignedAbs`, `ClampMagnitude`, `CastSigned`/`CastUnsigned` (one-method bit reinterpretation only — the checked/saturating/strict variants were deliberately dropped: they are exactly the generic casts at the counterpart type), `Widen<T>`/`Truncate<T>` (one-method each, generic target, exact std pair tables — same signedness, usize/isize conservative), and the four one-method generic casts `CheckedCast<T>`/`StrictCast<T>`/`WrappingCast<T>`/`SaturatingCast<T>` for all 144 int-type pairs.

### Typestate proofs (`src/ops/typestate.rs`)

Zero-runtime-cost newtypes that carry a structural invariant constructed once (checked `new`/`new_unchecked`/`from_ref`, const where the predicate allows) and *spent* by a consuming op that deletes a branch/`Option`/division. **Pure-`core`, no feature gate, always available** (the old `typestate` cargo feature was removed — it was a no-op once `ct` got its own gate). Each secret-derivable predicate also gets a `CtOption` `new_ct` under the `ct` feature; `BitIndex` (public shift amount) and `Finite` (floats are outside the CT model) deliberately have none. Wrapper *types* are root re-exports but stay OUT of `prelude::*` (generic names); the *ops traits* (`PowerOfTwoOps`/`BitIndexOps`/`HasNonZero`/`DivNonZero`, + ct `CtNonZero`) are in the prelude. Residents:
  - `PowerOfTwo<T>` (unsigned, **exponent rep** not value, `Copy`, no `Deref`) + `PowerOfTwoOps` (c0nst): `div_pow2`/`rem_pow2`/`is_multiple_of_pow2`/`next_multiple_of_pow2`(+checked) as shifts/masks.
  - `BitIndex<T>` (all 12 ints, **u32 rep**, `Copy`) + `BitIndexOps` (c0nst): `shl_index`/`shr_index` by an amount proven `< BITS` — no overflow-check branch, no `unbounded_*` masking.
  - `HasNonZero` (c0nst) bridges to `core::num::NonZero<Self>` (`nonzero_get` accessor, no `Into<Self>` bound — `From<NonZero>` isn't const) + `DivNonZero` (c0nst, **unsigned**): infallible `div_nonzero`/`rem_nonzero`.
  - `NonNegative<T>`/`Positive<T>` (signed, repr(transparent)): `to_unsigned`/`abs`/`isqrt` + const refinement narrowings (`into_nonnegative`/`into_nonzero`/`into_nonmin`).
  - `NonMin<T>` (signed, repr(transparent)): proof `!= MIN`. Total `neg`/`abs` and — as the dividend — total signed `div_nonzero`/`rem_nonzero` (the only signed-div overflow is `MIN / -1`). This is the co-proof unsigned `DivNonZero` doesn't need.
  - `Odd<T>`/`Even<T>` (repr(transparent), via `Parity`): bare proofs, consumer lives in modmath.
  - `Finite<T>` (f32/f64, repr(transparent)): const `new` via exponent-bit test (not the non-const `is_finite`); spent by a total `Ord`/`Eq` (NaN excluded ⇒ `partial_cmp().unwrap()` can't panic) so finite floats become sortable/`BTreeMap`-keyable.

### New ops coverage policy (decided 2026-06)

- **Scope**: every inherent integer numeric op in current nightly std, stable or unstable, EXCEPT the `unsafe unchecked_*` family (deliberately skipped — codegen-hint methods make no sense behind trait indirection).
- **MSRV 1.86** (matches fixed-bigint). Impls delegate to inherent std methods stabilized ≤ 1.86; anything newer (e.g. strict ops 1.91, bigint helpers 1.91, `is_multiple_of`/`unbounded_*`/`cast_signed` 1.87, isolate/bit_width 1.98) or still unstable (funnel/carryless/deposit-extract/exact-div/exact-shifts/widen/truncate/generic casts/`div_floor`/signed `div_ceil`/`wrapping_next_power_of_two`/`clamp_magnitude`/signed bigint helpers) is hand-rolled with core's own algorithm, taken from `library/core/src/num/*` or `library/core/src/intrinsics/fallback.rs` in the rust checkout at `/opt/m/rust/upstream/rust`.
- **Receivers and args are BY VALUE, mirroring core's inherent methods exactly** (`fn checked_add(self, v: Self)`, not `&self, &Self`). This was a deliberate sweep (2026-06, commit after `19d6a1f`) away from num-traits' `&self` house style — see *By-value + associated Output convention* below. It **breaks num-traits drop-in signature compat** (callers drop the `&`: `x.checked_add(&y)` → `x.checked_add(y)`); trait names and supertrait bounds are preserved. `compat_canary.rs` was retired (its premise was signature-level compat).
- **Cross-type ops use associated types** (`type Signed`/`type Unsigned`/`type Wide`/`type Output`); generic-target casts use a type parameter like `AsPrimitive<T>`.
- **Impl coverage is primitives-only** (the 12 integer types; sign-restricted where std restricts). New traits intentionally have NO `Wrapping<T>`/`Saturating<T>`/`NonZero` impls.
- **Unsigned-only like std**: `MultipleOf`, `IsPowerOfTwo`/`NextPowerOfTwo`, `BitWidth`, `FunnelShl/Shr`, `CarrylessMul`, `DepositBits`/`ExtractBits`, `CastSigned`, `CheckedSignedDiff`, `*AddSigned`/`*SubSigned`. Signed-only: abs/neg variants, `CheckedIsqrt`, `UnsignedAbs`, `ClampMagnitude`, `CastUnsigned`, `*AddUnsigned`/`*SubUnsigned`.
- **DESIGN.md** records the full layered-architecture plan (compat + modern + `ct` feature) and the review decisions behind these splits; read it before reshaping any trait.

### By-value + associated Output convention (2026-06 sweep)

Every value-producing trait method takes `self`/`Self`/scalar args by value, matching the core inherent method's signature exactly. To let **both `Copy` and non-`Copy`** types implement them (the latter via `impl Trait for &T` returning the owned result), owned-`Self`-returning methods carry an associated **Output**:

- **Operator-backed traits reuse the operator's `Output`** — relax the supertrait `Add<Self, Output = Self>` → `Add<Self>` and return `<Self as Add<Self>>::Output`. No new associated type; the impls are unchanged; `PrimInt: CheckedAdd<Output = Self>` still resolves. Covers all the `checked`/`wrapping`/`saturating`/`overflowing`/`strict`/`euclid` arith, the shifts, `CarryingAdd`/`BorrowingSub`/`CarryingMul`, `DivCeil`/`DivFloor`/`DivExact`/`NextMultipleOf`, `Signed::abs`/`abs_sub` (via `Neg`).
- **Traits with no usable operator supertrait add a fresh `type Output`** + `type Output = $t` in their impl macro: the neg-on-unsigned ops (`CheckedNeg`/`WrappingNeg`/`OverflowingNeg`/`StrictNeg`), `Signum`, `CarrylessMul`/`CarryingCarrylessMul`, `Isqrt`/`CheckedIsqrt`, `NextPowerOfTwo`, `Midpoint`, `IsolateHighestOne`/`IsolateLowestOne`, `FunnelShl/Shr`, `DepositBits`/`ExtractBits`, `ClampMagnitude`, the 20 mixed-sign flavor traits (`declare_mixed_trait!` adds it), and the `float_ops` owned-returning traits.
- **No Output needed** for methods returning `bool`/`u32`/`Option<u32>` or a *distinct* associated type (`WideningMul::Wide`, `AbsDiff::Output`, `CastSigned::Signed`, `ToBytes::Bytes`, `CheckedSignedDiff::Signed`, the generic `Cast<T>`/`Widen<T>`/`Truncate<T>`): `impl for &T` already works.
- **Ripples**: relaxing `Op<Self, Output = Self>` means generic callers that relied on "result is `Self`" must restate it (`pow.rs`: `T: CheckedMul + Mul<Output = T>`; `Wrapping<T>` blanket impls: `T: WrappingAdd<Output = T>`; the `Signed`/`Signum` free fns; per-file test generics). `div_rem_euclid`'s default (used `self` twice; not const-cloneable) was stripped and supplied in the impl macros + std/no_std float impls.
- **`ct.rs` traits stay `&self`** (plain non-const masked-return traits, matching `subtle`'s API style) — they were not part of the by-value sweep.
- **`FromBytes::from_*_bytes` stays borrowed `&Self::Bytes` with `type Bytes: NumBytes + ?Sized`** — deliberately NOT swept by-value. Rationale: the by-value convention is for numeric *operands/receivers* you might consume/reuse; `from_*_bytes` *reads* a buffer it never reuses as storage, so by-value bought nothing and only forced variable-length types (`type Bytes = [u8]`) into an owned `Vec<u8>` + an allocation. So `FromBytes` is left upstream-identical (zero-alloc slice impls keep working). `ToBytes::to_be_bytes` **is** by value (it must return `Bytes` by value, and its receiver is rescued by `impl ToBytes for &T`). Do not "consistency-fix" `FromBytes` to by-value. (Reverted 2026-06; see API_BREAKS.md row 5.)
- **COVERAGE.md** is the generated std-primitive-method coverage audit (from the `rust-docs-json` rustup component). Regenerate after adding traits. As of 2026-06-12 there are **zero uncategorized gaps and zero out-of-scope entries**: every safe inherent method on the integer and float primitives is covered or expressible via the generic casts; only the `unsafe` family is deliberately skipped.
- **MIGRATION.md** is the downstream-facing guide — the design narrative plus the precise, mechanical list of what changed (by-value calling convention, associated `Output`, trait splits, removed cast variants, features, MSRV). Update it whenever a change affects downstream call sites or impls.
- **Gotcha**: on nightly, unstable inherent methods (`u32::from_ascii`, …) shadow same-named trait methods in `Type::method(...)` call syntax and emit future-compat warnings — always call shadowed atoms as `<u32 as FromAscii>::from_ascii(...)` in this crate's tests/doctests.

### Layered architecture (P1–P3, implemented 2026-06-12)

- **`PrimBits`** (src/int.rs) is the capability-pure core extracted from `PrimInt` (the fixed-bigint `ConstBitPrimInt` cut): all bit methods moved UP into it; `PrimInt: [c0nst] PrimBits + Num + Ord + …` keeps only `pow`. `PrimBits: ConstZero + ConstOne` (associated consts — usable in const without callable constructors). `tests/prim_bits.rs` is the acceptance proof: a newtype with NO `PartialEq`/`Ord`/`Div` implements it.
- **`Signum`** (src/sign.rs) extracted the same way: `Signed: [c0nst] Signum`, `signum` lives only on the atom.
- **`RingOps`** (lib.rs) = `Add + Sub + Mul` aggregation (division-free `NumOps`); `NumOps: [c0nst] RingOps + Div + Rem`. Bound CT-compatible generic code on `RingOps`, not `NumOps`.
- **`FromStrRadix`** (lib.rs) is the standalone parsing atom (plain, never-const trait). `Num` keeps its own method + assoc type (associated types can't be re-exported through supertraits) — deliberate redundancy. **Watch out**: with both `Num` and `FromStrRadix` in scope, `T::from_str_radix(…)` is ambiguous; use `<T as Num>::from_str_radix` (in-crate tests already do).
- **`prelude`** module (lib.rs): glob re-export of every trait; the answer to "method moved to a supertrait and my concrete-type call broke".
- **Minimal-impl macros** (src/cast.rs): `impl_to_primitive_minimal!` / `impl_from_primitive_minimal!` regenerate the full 14-method `ToPrimitive`/`FromPrimitive` impls from just the 64-bit pair, restoring the upstream "minimal impl" pattern that const default-stripping broke. Generated impls are plain (non-const).
- **`ct` feature** (`ct = ["dep:subtle"]`, src/ops/ct.rs): masked-return counterparts of Tier-B atoms (`CtIsZero`, `CtParity`, `CtIsPowerOfTwo`, `CtCheckedAdd/Sub/Mul/Neg`, `CtCheckedSignedDiff`) using `subtle::Choice`/`CtOption`. Plain traits — subtle isn't const; do NOT wrap them in `c0nst!`.
- **CT tiers**: defined in the crate-root docs (Tier A/B/C + the public-parameter convention for shift amounts/exponents/bases); each ops module header carries its tier. Rule: a Tier-A trait never gets a Tier-C supertrait and never requires `PartialEq`/`Ord`/`Div`/`Rem`.
- **`tests/compat_canary.rs`**: num-traits caller patterns that must keep compiling (`use const_num_traits as num_traits`). Run it after ANY trait reshaping.
- Root re-exports: all new traits are re-exported from the crate root (the upstream Overflowing* traits stay module-path-only at `ops::overflowing`, preserving upstream behavior).
- `tests/const_nightly.rs` is the const-callability canary: every new family computed in `const` initializers; only built with `--features nightly`.

### Const free fns

`zero`, `one`, `cast`, `clamp`, `clamp_min`, `clamp_max`, `abs`, `abs_sub`, `signum`. Also the `int.rs` helpers `one_per_byte` and `reverse_bits_fallback`.

### Deliberately NON-const impls (blockers documented at site)

Each comes with an inline comment explaining why; don't "fix" these without addressing the upstream blocker first.

- **`Wrapping<T>: Num` / `Saturating<T>: Num`** — std's `PartialEq` for these wrappers isn't a const trait impl (orphan rule prevents us from adding it). Verified in `core/src/cmp.rs` of the rust source tree: `partial_eq_impl!` covers primitives + tuples but not `Wrapping`/`Saturating`. **Don't waste time trying** — needs an upstream change to `core::num::wrapping`/`saturating` to use `#[derive_const(PartialEq)]` or hand-written const impls.
- **`f32`/`f64: Num`** — the upstream `from_str_radix` parser uses iterators, `Result::map_err`, `?`, `str::parse`. None const. Kept verbatim.
- **`f32`/`f64: Signed`** — `FloatCore::signum` (and `f32::signum` / `f64::signum`) aren't const fns yet.
- **`f32`/`f64: Euclid`** — `f32::div_euclid` / `f64::div_euclid` aren't const fns. Split: `cfg(feature = "std")` macro forwards to the inherent methods, `cfg(not(feature = "std"))` impls use `crate::float::FloatCore::trunc`/`abs`.
- **`f32`/`f64: MulAdd` / `MulAddAssign`** — delegates to non-const `crate::Float::mul_add`.
- **`Wrapping<T>` blanket impls** of `WrappingAdd/Sub/Mul/Neg/Shl/Shr` — std's op impls for `Wrapping<T>` aren't const traits.
- **`Wrapping<T>: Signed` / `Unsigned`** — depend on the non-const `Wrapping<T>: Num`.

### Things that look like they could be const but can't yet

- **`Option::map`, `Option::and_then`, `Result::map`, `Result::map_err`** — not const fns. Anywhere we needed this pattern, we hand-roll a `match { Some(v) => ..., None => None }` block. See `impl_from_primitive_nonzero_one!`, `impl_num_cast_nonzero!`, `impl_from_primitive_wrapping!`, `CheckedEuclid::checked_div_rem_euclid` macro body.
- **`?` operator** — desugars through `Try`/`FromResidual`, not const. Same `match` rewrite.
- **`f32::to_int_unchecked` / `f64::to_int_unchecked`** — not const. `cast::float_to_int_unchecked!` switched to a plain `$float as $int` cast (saturating-cast since 1.45). Equivalent because the surrounding range check at the call site already guarantees an in-range value.

## Trait defaults: stripped vs restored

The const port couldn't keep defaults whose *original* bodies used non-const ops (`and_then`, `map`, `?`, `==`). But several were **rewritten const-friendly (match-based) and restored as defaults** (2026-06) to preserve the downstream "minimal impl" pattern and cut E0046 breakage — see `dependents/BREAKAGE.md`.

| Trait | Status | Detail |
|---|---|---|
| `ToPrimitive` | **12 of 14 RESTORED** | `to_i64`/`to_u64` stay required; the other 12 default through them (match-based, via `CheckedCast`). So a minimal impl (override only `to_i64`/`to_u64`) compiles again. `default_tests::minimal_impl_inherits_defaults` is the canary. |
| `FromPrimitive` | **12 of 14 RESTORED** | `from_i64`/`from_u64` required; other 12 default (range-check + `as`, no `try_from`/`and_then`). |
| `Zero` | `set_zero` still stripped | const default `*self = Self::zero()` needs `[const] Destruct` (the drop of the old value), which the `c0nst!` macro can't strip from a lone `where` clause and which ripples to callers — not worth it for one trivial method. |
| `One` | `set_one` stripped (same `Destruct` reason); `is_one` stripped (its `*self == Self::one()` needs non-const `PartialEq`) | |
| `Euclid`/`CheckedEuclid` | `div_rem_euclid`/`checked_div_rem_euclid` stripped | by-value default would use `self` twice (not const-cloneable); supplied in the forward-impl macros. |

So a downstream type now needs to supply only `to_i64`/`to_u64`/`from_i64`/`from_u64` (+ `set_zero`/`set_one`/`is_one`, three trivial lines) — not all 14. The `impl_to_primitive_minimal!`/`impl_from_primitive_minimal!` helper macros remain for types that prefer explicit non-const impls but are now optional.

## `Destruct` bounds

Generic-by-value functions (and a few impls) need to drop `T` at end of scope. In const context that requires `T: [const] core::marker::Destruct`. The `c0nst!` macro injects the fully-qualified path automatically, so just write `[c0nst] Destruct` in source — no `use` needed. On stable, the entire `+ [c0nst] Destruct` clause is removed; on nightly it becomes `+ [const] core::marker::Destruct`.

Affected fns: `cast`, `clamp`, `clamp_min`, `clamp_max`, `abs`, `abs_sub`, `signum`.

Affected impls: `Wrapping<T>: FromPrimitive`, `Wrapping<T>: NumCast`, and `NumCast::from<T>` itself.

## Build & test

```bash
# stable (default features = std)
cargo build
cargo test          # 101 unit + 20 integration + 240 doctests
                    # (+5 const-context tests on nightly via tests/const_nightly.rs)

# nightly, const traits enabled
cargo +nightly build --features nightly
cargo +nightly test --features nightly

# no_std
cargo build --no-default-features
```

`cargo +nightly rustc --lib --features nightly -- -Zunpretty=expanded` is invaluable for confirming the const expansion — search the output for `pub const trait` / `impl const X for Y`.

## Code layout

- `src/lib.rs` — root `Num` + marker traits (`NumOps`, `NumRef`, …) + `clamp` fns + the (non-const) float `from_str_radix` parser.
- `src/identities.rs` — `Zero`, `One`, plus the (independent, non-c0nst) `ConstZero` / `ConstOne` associated-constant traits.
- `src/bounds.rs` — `Bounded`, `LowerBounded`, `UpperBounded`.
- `src/cast.rs` — `ToPrimitive`, `FromPrimitive`, `NumCast`, `AsPrimitive`, `cast()`.
- `src/int.rs` — `PrimInt` + helper fns `one_per_byte` / `reverse_bits_fallback`.
- `src/sign.rs` — `Signed`, `Unsigned`, `abs`/`abs_sub`/`signum` fns.
- `src/ops/{checked,wrapping,saturating,overflowing,euclid,inv,mul_add,bytes}.rs` — operator traits (upstream families, extended with the missing pow/abs/div/rem/shift/euclid variants).
- `src/ops/{strict,bigint,rounding,log,sqrt,pow2,bits,mixed,convert}.rs` — fork-added trait families (see *New ops coverage policy*).
- `tests/const_nightly.rs` — const-context compile/run tests, `--features nightly` only.
- **Verbatim upstream** (only crate-name renames in doctests): `src/float.rs`, `src/real.rs`, `src/pow.rs`. **Do not** wrap these in `c0nst!` — `Float`/`Real`/`Pow` delegate to non-const `Float::*` methods and aren't const-portable in their current form.

## Common pitfalls

1. **Doc comments must be inside the `c0nst::c0nst! { ... }` block**, otherwise rustdoc warns "unused doc comment, rustdoc does not generate documentation for macro invocations". Pattern:

   ```rust
   c0nst::c0nst! {
   /// Trait doc.
   pub c0nst trait Foo { ... }
   }
   ```

   not

   ```rust
   /// Trait doc.        // wrong: rustdoc skips this
   c0nst::c0nst! {
   pub c0nst trait Foo { ... }
   }
   ```

2. **rustdoc currently doesn't render the `const` keyword** for `const trait` items, even on nightly. So `cargo +nightly doc --features=nightly` will show `pub trait Foo: Sized + Add<...>` — that's a rustdoc limitation, not a bug in our integration. Verify const-ness via `-Zunpretty=expanded`.

3. **Non-const impl of a const trait is allowed.** Don't add `[c0nst]` keywords on impls that you know can't be const (e.g., `Wrapping<T>: Num` is intentionally plain `impl<T: Num> Num for Wrapping<T>`). Nightly accepts this and just makes the impl non-callable from const contexts.

4. **Default method bodies in a const trait must be const-callable for all impls.** If you keep a default that calls `Option::and_then`, every impl breaks on nightly. Either strip the default or rewrite the body to const-friendly ops.

5. **When adding a new const trait**, the same `c0nst::c0nst! { … }` wrap-the-trait-and-every-impl-macro pattern applies. Use `[c0nst]` on operator supertraits (`Add`/`Sub`/`Mul`/…) and `[c0nst]` on other const-trait supertraits in this crate (`Num`, `Bounded`, etc.). Leave non-const supertraits (e.g., `'static`, `Copy`, `Sized`) bare.

6. **Test naming**: integer tests typically live next to the trait (`#[test]` blocks in the same file). The big integration test is `tests/cast.rs` — it exercises downstream usage patterns and is the canary for default-stripping issues.

## What's not const-portable (yet) and why we left it

`Float`, `FloatCore`, `Real` — the trait surface is huge (~50 transcendental methods) and the bodies route through `std`/`libm` for `sqrt`/`sin`/`cos`/`ln`/etc., none of which are const. A const port would either have to be a much narrower trait (bit-pattern and classification methods only) or wait for stdlib float fns to be marked const. We left these as plain upstream code.

If a future session decides to const-port a slice of `Float` (e.g., just `is_nan`, `is_infinite`, `to_bits`, `from_bits`, the IEEE constants), that's feasible — just wrap those methods + the trait declaration the same way.
