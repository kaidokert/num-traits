# API breaks vs. `num-traits` 0.2

This fork is **not a drop-in replacement** for `num-traits`. Trait *names* and
*supertrait bounds* are preserved, but signatures changed. This document is the
contract: each breaking change, what it breaks, and the reason we chose it.
For *how* to migrate, see [MIGRATION.md](MIGRATION.md); for a per-crate damage
survey, [../dependents/BREAKAGE.md](../dependents/BREAKAGE.md).

The whole fork exists to make the trait surface **const-callable on nightly**
while staying byte-identical to upstream on stable. Almost every break below is
downstream of one root requirement: **be a faithful `const` mirror of `core`'s
own integer/float methods.** Where a break wasn't forced by that, we say so.

## What does NOT break (so you can scope it)

- **Trait names** — `use num_traits::{CheckedAdd, Signed, …}` still resolves.
- **Supertrait bounds** — `T: CheckedAdd`, `T: Num`, `T: PrimInt` still mean
  what they did; generic *bounds* don't change.
- **The const-stable contract** — on stable, behavior is byte-identical to
  upstream; `nightly` only adds const-callability, never changes which items
  exist or their signatures.

## The breaks

| # | Change | What breaks downstream | Why we chose it |
|---|---|---|---|
| 1 | **By-value receivers/args.** `checked_add(self, v: Self)`, not `(&self, &Self)`. Same for `wrapping_*`, `Euclid::*`, `Signed::abs/abs_sub/is_*`, byte ops. | Call sites drop the `&`: `x.checked_add(&y)` → `x.checked_add(y)` (E0053 in impls, E0308 at calls). **Second-order:** if a downstream type has its own `core`-shaped `saturating_add`/`wrapping_*`/etc., the now-identical signatures make a bare call ambiguous (E0034) where `&self`/`&Self` kept them distinct — qualify it (`SaturatingAdd::saturating_add(a, b)`). This is the bulk of why `fixed` is the worst-hit crate. | `core`'s inherent methods are all by value. To be a faithful const mirror the trait signatures must match `core` exactly — keeping the old `&self`/`&Self` signatures would diverge from the thing we mirror and couldn't be the same item on both toolchains. |
| 2 | **Associated `Output` on owned-returning traits**; operator supertraits relaxed `Add<Self, Output=Self>` → `Add<Self>`. | Generic code that assumed "result is `Self`" must restate it: `T: CheckedMul + Mul<Output = T>` (E0271/E0308/E0369). | This is the price of supporting **non-`Copy`** implementors. By-value + non-`Copy` is only viable if a type can `impl Trait for &T` and return the owned result — which requires the return type to be an associated `Output`, not a hard `Self`. We explicitly prioritized "usable by both `Copy` and non-`Copy` types" over signature minimalism. |
| 3 | **`signum` extracted** from `Signed` into a `Signum` atom (same for `PrimBits`⊂`PrimInt`, `RingOps`, `FromStrRadix`). | `impl Signed` with a `signum` method fails (E0407); `Signed` now requires a separate `impl Signum` (E0277). | Capability layering for constant-time work: an atom (`Signum`, `PrimBits`) carries no CT-hostile supertraits (`PartialEq`/`Ord`/`Div`). A non-`Copy`, no-`PartialEq` newtype — or a bignum answering from its low limb — can implement the atom without dragging in bounds it can't satisfy const-ly. |
| 4 | **A few defaults are now required** — `set_zero`, `set_one`, `is_one`, `Euclid::div_rem_euclid`, `CheckedEuclid::checked_div_rem_euclid`. | Minimal impls that relied on these defaults fail (E0046). | Their upstream bodies use ops that aren't const: `*self = …` needs `const Destruct`; `is_one` needs `==`; the euclid pair used `?`/`self` twice. They couldn't be kept as const defaults. |
| 5 | **`ToBytes::to_be_bytes` takes `self` by value** (`FromBytes` is *unchanged* — still borrowed `&Self::Bytes`, `?Sized`). | Downstream `to_be_bytes(&self)` impls drop the `&` (E0053); a type that must stay readable after the call adds `impl ToBytes for &T`. `FromBytes` impls are untouched. | `ToBytes` must *return* `Bytes` by value (you can't return an unsized value), so a by-value receiver is the consistent shape — and its receiver is rescued by `impl … for &T`. `FromBytes` only *reads* its buffer (it never reuses it as storage), so it stays borrowed + `?Sized`: variable-length types keep the zero-alloc `type Bytes = [u8]` impl exactly as upstream. Applying by-value there would have forced an owned `Vec<u8>` and an allocation for no benefit — so we didn't. |
| 6 | **Redundant cast variants removed** — the checked/saturating/strict `cast_signed`/`cast_unsigned` family. | Calls to those names fail; use the generic `CheckedCast<T>`/`SaturatingCast<T>`/`StrictCast<T>`/`WrappingCast<T>` instead. | They were *exactly* the generic casts at the counterpart type — pure redundancy. One generic cast table (all 144 int pairs) replaces them. |
| 7 | **Method atomization → ambiguity.** `from_str_radix` lives on both `Num` and `FromStrRadix`. | With both in scope, `T::from_str_radix(…)` is ambiguous (E0034); qualify as `<T as Num>::from_str_radix`. (A minor, unobserved E0034 source — the dominant one is the by-value convergence in #1.) | The standalone parsing atom is needed so parsing can be required without `Num`'s full arithmetic surface. Associated types can't re-export through supertraits, so the redundancy is deliberate, not accidental. |
| 8 | **MSRV raised to 1.86** (was 1.60). | Older toolchains won't build. | Impls delegate to `core` inherent methods stabilized ≤ 1.86; anything newer/unstable is hand-rolled, so 1.86 is the real floor. |

## Was it worth it? (the one-paragraph defense)

Every break traces to one of two non-negotiables: **(a)** be a faithful const
mirror of `core` (→ by-value signatures, #1/#5/#8), or **(b)** stay usable by
non-`Copy` types and constant-time code (→ associated `Output` and atom
splits, #2/#3). #4/#6/#7 are minor cleanups forced by const-ness or pure
redundancy. The blast radius is deliberately small — trait names and bounds are
preserved — so that adoption is a **mechanical, compiler-guided** edit (drop a
`&`, add a bound, split one impl), not a structural rewrite. Measured cost on
real crates ranged from **1 edit** (`num-integer`) to **~20** (`num-bigint`),
scaling only with how much of the surface a crate touches.
</content>
