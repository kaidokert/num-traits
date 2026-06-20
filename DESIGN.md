# Trait architecture plan: compat + modern + constant-time

Status: **P0–P3 + `ct` feature implemented 2026-06-12.** Remaining: P4
(fixed-bigint migration — delete its `const_numtraits.rs` /
`patch_num_traits.rs`, implement atoms per personality, re-export
`Parity`/`ct` traits from here). Deviations from the letter of the plan:
per-trait CT tier tags were rolled up into module-header tier notes plus the
crate-root tier definition (same information, less doc churn); `CtParity`
was added to the `ct` v1 set (modmath's Montgomery-ladder bit).

## Resolved decisions (review round 1)

1. Bundle/atom relation: **forwarding defaults**, not blanket impls.
2. `Num`/`FromStrRadix` associated-type redundancy: **accepted**.
3. Mixed-sign ops: **full split** into one-method flavor traits (macros
   absorb the count).
4. CT layer: **take the `subtle` dependency**, and gate the *entire*
   CT-tailored surface behind a `ct` cargo feature (see *The `ct` feature*
   below). Eases fixed-bigint migration (it already depends on `subtle 2.6`).
5. Division-free aggregation (`RingOps`): **approved**.
6. Redundant sign-cast/truncate variants: **delete** in favor of the generic
   cast traits.

Editor's calls on the remaining open questions: `Ilog2`/`Ilog10`/`Ilog` keep
the `{op, checked_op}` pairing (same capability, panicking derives from
checked); forwarding defaults land eagerly in P2; single `prelude` module
re-exporting modern + compat (names are collision-free).

## The `ct` feature

- `Cargo.toml`: `ct = ["dep:subtle"]`, with
  `subtle = { version = "2.6", default-features = false, optional = true }`.
- New module `src/ops/ct.rs` (re-exported at root behind the feature):
  Tier-B atoms get masked-return counterparts using `subtle::Choice` /
  `subtle::CtOption` — v1 set, driven by what fixed-bigint's `Ct`
  personality needs:
  - `CtIsZero` (`-> Choice`), `CtIsPowerOfTwo` (`-> Choice`)
  - `CtCheckedAdd/Sub/Mul/Neg` (`-> CtOption<Self>`)
  - `CtCheckedSignedDiff` (`-> CtOption<Self::Signed>`)
  - selection/equality themselves come from `subtle`
    (`ConditionallySelectable`, `ConstantTimeEq`) — we do not duplicate them.
- **Const caveat**: `subtle`'s constructors are not `const fn`, so the `ct`
  traits are *plain* traits (no `c0nst!` wrap) until subtle grows const
  support. Document this asymmetry; it does not affect the c0nst story of
  the rest of the crate.
- Primitive impls for the unsigned + signed types `subtle` supports;
  fixed-bigint implements them for `FixedUInt<_, _, Ct>` only, preserving
  its "wrong-personality calls are compile errors" property.

## Goals

1. **Drop-in migration**: `sed s/num_traits/const_num_traits/` on a downstream
   crate should compile. Every num-traits trait keeps its name, methods,
   signatures (`&self` receivers, associated types) and root re-export.
2. **Modern surface**: every inherent integer op in current nightly std has a
   trait, in a cleaned-up, predominantly one-method form.
3. **Constant-time (CT) viability**: a CT integer type (e.g.
   `FixedUInt<_, _, Ct>`) must be able to implement a useful subset without
   being forced to expose comparison, division, data-dependent panics, or
   data-dependent branching. No CT-safe trait may have a CT-hostile
   supertrait.
4. Everything stays `c0nst!`-portable: plain traits on stable, `const trait`
   on nightly.

Non-goals: CT *enforcement* (a trait can't prove an impl is branchless — this
is a documentation/contract tier, not a type-system property); float traits;
a `subtle::Choice`-style masked-boolean API (see Tension 5).

## Architecture: three layers

```
L2  compat bundles      Num, PrimInt, Signed, ToPrimitive, FromPrimitive,
    (num-traits API,    NumCast, Zero/One, Euclid…  — names & signatures
     verbatim)          frozen; re-expressed over L0 where possible
         ▲  supertrait + forwarding defaults (PrimInt pattern)
L1  std-mirror          one trait per inherent-method family; what the
    (this fork's new    "complete nightly coverage" pass added, after the
     surface)           splits below
         ▲  (mostly the same traits — L1 traits ARE atoms after splitting)
L0  capability atoms    one-method (or one-capability) traits, each graded
                        for CT implementability
```

Key observation: most of num-traits' variant traits (`CheckedAdd`,
`WrappingShl`, `SaturatingMul`, …) are *already* atom-shaped. The compat
problem is confined to about eight bundles: `PrimInt`, `Num` (+ the
`NumOps`/`NumRef`/`NumAssign*` aggregations), `Zero`/`One` (predicates),
`Signed`, `ToPrimitive`/`FromPrimitive`/`NumCast`, and the `Float*` family
(out of CT scope, stays verbatim). Everything added by this fork is
unpublished and freely refactorable.

### Verified mechanics

The forwarding pattern works under `c0nst!` on stable *and* nightly-const
(probe: `/tmp/c0nst_fwd_test`, const-asserted):

```rust
c0nst::c0nst! {
pub c0nst trait Bundle: [c0nst] Atom {
    fn bundle_op(&self) -> u32 { self.atom_op() + 1 }   // forwarding default
}
}
c0nst::c0nst! { impl c0nst Bundle for u32 {} }           // empty impl, default kept
```

Also load-bearing: for a generic parameter `T: PrimInt`, methods of
*supertraits* resolve without importing them — so generic callers are immune
to method extraction. Only **concrete-type method-syntax callers** of
non-primitive impls (`my_bigint.count_ones()` with just `use …::PrimInt`)
need the supertrait import (or a prelude). For the primitive types themselves,
inherent methods shadow trait methods anyway.

## CT classification (documentation tiers)

Every L0/L1 trait gets a rustdoc tier tag:

- **Tier A — CT-implementable**: branchless on data; data-dependent inputs
  flow only through arithmetic/bitwise ops. (wrapping/overflowing/carrying/
  borrowing/widening ops, carry-less mul, bit isolation, byte/bit
  reversal/rotation, `cast_signed`/`cast_unsigned`, lossy `truncate`,
  `widen`, `midpoint`, `abs_diff`, `bit_width`, funnel/unbounded shifts*)
- **Tier B — caller-leaky**: implementable branchlessly, but the *return
  type* (`bool`, `Option`) invites leaking at the call site.
  (`checked_*`, `is_power_of_two`, `is_multiple_of`, `highest_one`/`lowest_one`,
  `checked_signed_diff`)
- **Tier C — CT-hostile**: data-dependent control flow, division, or
  data-dependent panics. (everything `Div`/`Rem`-based incl. euclid/div_ceil/
  ilog10/ilog/next_multiple_of/isqrt, the whole `strict_*` family,
  `from_str_radix`, `pow` with secret exponent)

(*) Convention: **shift amounts, exponents, and log bases are public
parameters** — branching on them does not demote a trait from Tier A/B.
Document this convention once, in the crate root.

Rule enforced by review: a Tier-A trait never has a Tier-C supertrait, and
never has `PartialEq`/`PartialOrd`/`Div`/`Rem` as supertraits.

## Disposition table

### Fork-new traits (unpublished — change freely now)

| Today | Disposition | Why |
|---|---|---|
| `Ilog` (6 methods) | split → `Ilog2`, `Ilog10`, `Ilog` (base), each `{op, checked_op}` | `ilog2` is Tier A/B (leading_zeros); `ilog10`/`ilog` are Tier C (division loops). Checked+panicking stay paired: same capability, panicking derives from checked. |
| `IsolateOne` | split → `IsolateHighestOne`, `IsolateLowestOne` (Tier A, `Self→Self`) + `HighestOne`, `LowestOne` (Tier B, `→Option<u32>`) | branchless isolators vs zero-branching index finders |
| `CastSigned`/`CastUnsigned` (4 methods) | shrink → one-method (plain bit cast, Tier A). **Delete** checked/saturating/strict variants | the variants are exactly `CheckedCast::<Signed>`/`BoundedCast::<Signed>` on the counterpart type — pure redundancy with the generic casts |
| `Truncate<T>` (3 methods) | shrink → one-method lossy `truncate` (Tier A) | `checked_truncate`/`saturating_truncate` ≡ generic casts restricted to the same-sign pair table; `Truncate`/`Widen` survive as *intent* traits only |
| `CheckedCast<T>` {checked, strict} + `BoundedCast<T>` {wrapping, saturating} | split → `CheckedCast<T>`, `StrictCast<T>`, `WrappingCast<T>`, `SaturatingCast<T>`, all one-method | maps 1:1 to std's generic `*_cast` methods; `wrapping_cast` is the Tier-A one and shouldn't be welded to compare-based `saturating_cast` |
| `PowerOfTwo` | split → `IsPowerOfTwo` (Tier B predicate) + `NextPowerOfTwo` {next, checked_next, wrapping_next} | predicate vs constructors; the three constructors share one capability (the `one_less` trick) |
| `DepositExtractBits` | split → `DepositBits`, `ExtractBits` | independent ops (PDEP without PEXT is coherent) |
| `Isqrt` | keep `isqrt`; move `checked_isqrt` to signed-only (drop the artificial unsigned always-`Some`) | restores exact std mirroring |
| `AddSigned`/`SubSigned`/`AddUnsigned`/`SubUnsigned` (5 flavors each) | **RESOLVED: full split** per flavor (20 one-method traits with `type Signed`/`type Unsigned`) | house convention everywhere else is flavor-per-trait; `wrapping_*` are the Tier-A members |
| `CarryingAdd`, `BorrowingSub`, `CarryingMul` {mul, mul_add}, `WideningMul`, `CarrylessMul`, `WideningCarrylessMul`, `Midpoint`, `AbsDiff`, `UnsignedAbs`, `ClampMagnitude`, `BitWidth`, `FunnelShl/Shr`, `UnboundedShl/Shr`, `ShlExact/ShrExact`, `DivCeil`, `DivFloor`, `DivExact`, `MultipleOf`, `NextMultipleOf`, `CheckedSignedDiff`, all `Strict*` | keep as-is | already atom-shaped or derivable-pair bundles within one capability (`carrying_mul ≡ carrying_mul_add(…, 0)`; `StrictEuclid` mirrors the `Euclid` div+rem pairing) |
| `Parity` {is_odd, is_even} (added 2026-06-12 at review) | new atom, `ops/parity.rs` | no std counterpart (num-integer `Integer` bundles it); modmath's blanket-impl shape requires `PartialEq + Clone`, which excludes CT types — a per-type atom doesn't. Tier B (the returned `bool` *is* the operand's low bit). modmath migration: replace its local `Parity` with a re-export of this one; its no-feature blanket fallback can't coexist (orphan rule), so the per-type impls here + `FixedUInt`'s own impl cover its ecosystem. |

### Legacy num-traits bundles (frozen API — extract underneath)

| Bundle | Extraction | Mechanism |
|---|---|---|
| `PrimInt` | extract `PrimBits` supertrait: count/leading/trailing ones+zeros, rotates, swaps, `reverse_bits`, endian conversions, unsigned/signed shifts — the proven fixed-bigint `ConstBitPrimInt` cut. `PrimInt: PrimBits + NumOps + Ord + …` keeps `pow` + arithmetic-dependent rest | methods move *up* (live only in `PrimBits`); `PrimInt` bound still provides them to generic code |
| `Num` | extract `FromStrRadix` as standalone atom (own `type Err`). `Num` keeps its own `FromStrRadixErr` + required method (NOT forwarded) | `<T as Num>::FromStrRadixErr` appears in downstream signatures; associated types can't be re-exported through supertraits on stable, so `Num` stays self-contained and primitives implement both. Documented redundancy. |
| `Zero`/`One` | no change. CT atoms already exist: `ConstZero`/`ConstOne` (associated consts, no predicate). Do **not** make `Zero: ConstZero` | would break `Wrapping<T>: Zero` for `T: Zero` without `ConstZero`; predicates (`is_zero`/`is_one`) stay compat-only (Tier B/C) |
| `Signed` | no structural change; modern layer already covers it (`UnsignedAbs`, `Checked/Wrapping/Saturating/Strict/Overflowing-Abs`); add one-method `Signum` atom | `abs` (panics on MIN), `signum`, deprecated `abs_sub` stay bundled in compat |
| `ToPrimitive`/`FromPrimitive` (14 methods each) | no structural change; modern answer is `CheckedCast<T>`. Add helper macros `impl_to_primitive_via_checked_cast!` / `impl_from_primitive_via_checked_cast!` so third-party implementors write one line instead of 14 methods | per-target atom traits (`ToI64`, …) would be 28 traits of noise; the generic cast traits already are the atoms |
| `NumCast`/`AsPrimitive<T>` | no change; document `AsPrimitive<T>` ≡ `WrappingCast<T>` with `Copy + 'static` ergonomics | redundancy is the price of compat |
| `Euclid`/`CheckedEuclid`/`WrappingEuclid`/`OverflowingEuclid`/`StrictEuclid` | keep div+rem pairing everywhere | co-implementable, upstream precedent, uniformly Tier C |
| `Bounded` | already split upstream (`LowerBounded`/`UpperBounded`) — the in-crate precedent for this whole plan, alongside deprecated `Saturating` → `SaturatingAdd/Sub/Mul` |
| `Float`/`FloatCore`/`Real`/`Pow`/`MulAdd` | out of scope, verbatim |

## Tension points (decide at review)

1. **Method resolution vs extraction.** Moving a method from bundle to
   supertrait breaks *concrete-type, method-syntax* callers who import only
   the bundle (`use …::PrimInt; bigint.count_ones()`). Generic code and
   primitive types are unaffected. Mitigation: `prelude` module; accept the
   narrow break. Alternative — keeping the method in *both* traits — is
   worse: both in scope → ambiguity error requiring UFCS.
2. **Forwarding defaults vs blanket impls.** Blanket
   (`impl<T: Atoms> Bundle for T`) guarantees bundle/atom consistency but
   forecloses manual bundle impls (coherence) — a hard break for downstream
   implementors. Forwarding defaults (verified above) keep manual impls
   possible but allow a type to override a bundle method inconsistently with
   its atom. **Recommendation: forwarding defaults**, consistency by review.
3. **Associated types block clean `Num` extraction** (see table). Accept the
   `Num`/`FromStrRadix` redundancy.
4. **Mixed-sign flavor split: 20 tiny traits vs 4 bundles.**
   - (a) Full split — consistent with `CheckedAdd`-style convention, CT-clean,
     but 20 traits each repeating `type Signed`.
   - (b) Keep 4 bundles — compact, but the only place where flavors are
     welded together; a CT type wanting only `wrapping_add_signed` must
     implement panicking `strict_add_signed`.
   - (c) Bundles + carve out just the `Wrapping*` atoms — splits the
     difference, two homes for one method.
   **Recommendation: (a)**; the trait count is mechanical macro output.
5. **CT predicates return `bool`/`Option` — inherently caller-leaky.** An
   honest CT API needs a masked-boolean (`subtle::Choice`) and `CtOption`,
   which drags in a dependency and a parallel method set. **Recommendation:
   out of scope here**; tiers document the hazard, fixed-bigint's `Ct`
   personality handles masking internally. Revisit as an optional
   `subtle`-gated feature later.
6. **Operator supertraits on atoms.** `CheckedAdd: Add` (legacy, frozen)
   forces CT types to expose a plain `+`. In practice CT integer types do
   implement `Add` (wrapping or panic-on-overflow-in-debug), so this is
   acceptable; for *new* Tier-A atoms keep supertraits minimal (at most the
   matching `core::ops` trait; never `Ord`/`PartialEq`/`Div`/`Rem`).
7. **`NumOps` welds `Div + Rem` into the basic-arithmetic aggregation**, so
   a division-less CT type can never satisfy `Num`/`NumOps`-bounded generic
   code. Modern fix: an additive aggregation alias (e.g.
   `RingOps = Add + Sub + Mul` analog of `NumOps`) so generic CT code has a
   bound to use. Naming TBD at review. (`Num`'s `PartialEq` supertrait is the
   second, unfixable-in-compat exclusion.)
8. **Receiver style. RESOLVED (2026-06) → BY VALUE, tracking core exactly.**
   The original `&self` recommendation was reversed after review: we can't
   assume an implementor's `Copy`-ness, and *correctness/utility* (not
   implementation cost) drive the choice. Every value-producing method takes
   `self`/`Self`/scalars by value, matching the core inherent signature.
   Non-`Copy` viability is preserved via an associated `Output` (reused from
   the operator supertrait where one exists, fresh otherwise) so a non-`Copy`
   type implements `Trait for &T` returning the owned result — proven to
   compile. This **breaks num-traits drop-in signature compat** (accepted;
   names + supertraits preserved, `compat_canary` retired). See CLAUDE.md
   "By-value + associated Output convention" for the full recipe.
9. **Where the sign-cast/truncate variant deletions hurt std-mirroring.**
   After the shrink, `x.checked_cast_signed()` has no same-name trait; users
   write `CheckedCast::<i32>::checked_cast(&x)`. Accepted cost of removing
   duplication; revisit if std stabilizes the variants under these names.

## Phases

- **P0 — split the unpublished traits** (table §fork-new): pure refactor of
  this fork's new code + tests; no compat surface touched. Do before anything
  depends on the current shapes.
- **P1 — `PrimInt` → `PrimBits` extraction**, porting the proven
  fixed-bigint split; includes the forwarding-default machinery and the
  `prelude` module. Validate by building fixed-bigint against it.
- **P2 — remaining extractions** (`FromStrRadix`, `Signum`, the
  `to/from_primitive_via_checked_cast` helper macros, `RingOps`-style
  aggregation), one bundle per PR, each with a "compat canary" test that
  `use`s only num-traits names and compiles unchanged call sites.
- **P3 — CT tier documentation pass**: tier tag on every trait rustdoc, the
  public-parameter convention in the crate root, CLAUDE.md rule (no Tier-C
  supertrait under Tier-A trait).
- **P4 — fixed-bigint migration**: delete `const_numtraits.rs` /
  `patch_num_traits.rs`, implement atoms per personality (`Ct` implements
  Tier A + chosen Tier B; var-time implements through Tier C), keep only
  `ConstMachineWord` locally. This is the acceptance test for the whole
  design.

## Open questions — RESOLVED (see *Resolved decisions* at top)

1. Mixed-sign split: ~~(a)/(b)/(c)~~ → **(a) full split**.
2. `Ilog*` pairing: **keep `{op, checked_op}` paired** (editor's call).
3. Division-free aggregation: **approved**; name `RingOps` pending P2.
4. Forwarding defaults: **eagerly in P2**.
5. Tier-B masking: **yes — `ct` feature with `subtle`** (see section above).
6. Prelude: **single `prelude`** re-exporting modern + compat.
