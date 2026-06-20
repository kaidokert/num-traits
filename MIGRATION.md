# Migration guide

How `const-num-traits` got to its current design, and exactly what changed
for code that depends on it (whether you're coming from upstream
`num-traits` or from an earlier state of this fork).

> For a concise contract of *what* breaks and *why we chose each break*, see
> [API_BREAKS.md](API_BREAKS.md). This guide is the *how-to-fix* companion.

---

## Part 1 — The journey (why the design is what it is)

This started as a straight const-traits port of `num-traits` (the `c0nst!`
macro makes the source read as plain traits on stable and as
`const trait`/`impl const` on nightly). From there it grew through a series
of deliberate decisions, each driven by a concrete goal:

1. **Complete the surface.** We audited every inherent numeric method on the
   integer and float primitives in current nightly `std` (via the
   `rust-docs-json` component) and added a trait for each one that was
   missing — the full checked/wrapping/saturating/overflowing/**strict**
   variant matrix, bigint helpers (`carrying_*`, `borrowing_sub`,
   `widening_mul`, carry-less mul), rounding, `ilog*`/`isqrt`/power-of-two,
   bit manipulation, mixed-signedness, conversions, byte-string parsing
   (`FromAscii`) and emission (`FormatInto`), `Parity`, and a first slice of
   float ops. `COVERAGE.md` is the audit: **zero uncategorized gaps**; only
   the `unsafe unchecked_*` family is deliberately excluded.

2. **Split the aggregates.** num-traits (and our additions) bundled several
   capabilities per trait. That's ergonomic until it isn't: a
   constant-time integer type can implement bit operations but not division
   or comparison, so a bundle that welds them together is unimplementable.
   We split along capability lines — `PrimInt` → `PrimBits` + `PrimInt`,
   `Signed` → `Signum` + `Signed`, `Ilog` → `Ilog2`/`Ilog10`/`Ilog`, and so
   on (full list in Part 2) — extracting the cheap/constant-time-friendly
   core as a supertrait so the bundle still provides everything to generic
   code.

3. **Layer it.** A `RingOps` aggregation (`Add + Sub + Mul`, division-free)
   for code that can't assume division; a `FromStrRadix` atom; a `prelude`;
   helper macros to regenerate the 14-method `ToPrimitive`/`FromPrimitive`
   impls from a 2-method core; CT (constant-time) tier tags (A/B/C) on every
   op; and a `ct` feature exposing `subtle`-backed masked-return
   counterparts of the leaky predicates/checked ops.

4. **Match core's calling convention.** The biggest change. num-traits uses
   `fn checked_add(&self, v: &Self)` — by-reference receivers and arguments —
   because it was built bignum-first (`num-bigint` is heap-allocated and
   non-`Copy`, so by-ref avoids moves/clones). core's inherent methods are
   by-value because primitives are `Copy`. We chose **by-value, tracking
   core exactly**, because (a) this crate's implementors are predominantly
   `Copy`, (b) it's the convention a reader already knows from std, and (c)
   the original ref signature is what made the downstream `widening_mul`
   integration awkward.

   The catch: pure by-value `fn checked_add(self, v: Self) -> Option<Self>`
   *forecloses* non-`Copy` implementations — a non-`Copy` type can only
   `impl for &T`, and `Option<&T>` can't be constructed from a computed
   result. We weren't willing to make the crate non-viable for non-`Copy`
   types (we can't know who will implement these). The resolution: every
   owned-`Self`-returning method carries an associated **`Output`** — reused
   from the operator supertrait where one exists (`Add<Self>` →
   `<Self as Add>::Output`), fresh otherwise. A `Copy` type's per-impl
   signature is byte-identical to core's; a non-`Copy` type implements
   `Trait for &T` with `Output = OwnedType`. Both work; nothing is assumed
   about the implementor.

The result: a single, uniform, core-matching surface that serves `Copy` and
non-`Copy` implementors alike, with capability splits that let
constant-time and division-free types implement exactly the subset they
support. The cost is num-traits *signature* compatibility — addressed
precisely below.

`DESIGN.md` has the full architecture and the decision log; `CLAUDE.md` has
the per-file mechanics.

---

## Part 2 — What changed, precisely

### 2.1 Calling convention: by value (the headline)

Every **value-producing** trait method now takes its receiver and
`Self`-typed arguments **by value** instead of by reference.

```rust
// before (num-traits / earlier fork)        // now
fn checked_add(&self, v: &Self) -> ...        fn checked_add(self, v: Self) -> ...
fn wrapping_shl(&self, rhs: u32) -> ...        fn wrapping_shl(self, rhs: u32) -> ...   // scalar args unchanged
fn signum(&self) -> Self                       fn signum(self) -> Self::Output
```

**At call sites, drop the `&`:**

```rust
x.checked_add(&y)                  →  x.checked_add(y)
CheckedAdd::checked_add(&x, &y)    →  CheckedAdd::checked_add(x, y)
a.widening_mul(&b)                 →  a.widening_mul(b)
```

Receivers are `Copy` for primitives, so this is a pure ergonomic
simplification there. For a non-`Copy` value you keep what you have after
the call by implementing the trait for the reference type (see 2.3) or by
`.clone()`-ing.

**Unchanged:** in-place methods keep `&mut self` (`Zero::set_zero`,
`One::set_one`, `MulAddAssign`); no-receiver methods are unchanged
(`Num::from_str_radix`, `Bounded::{min,max}_value`, `NumCast::from`,
`FromPrimitive::from_*`, `AsPrimitive::as_`); the `ct` feature's
masked-return traits stay `&self` (they follow `subtle`'s API style).

### 2.2 Generic bounds may need to restate the result type

Operator-backed traits no longer pin their supertrait's `Output` to `Self`
(that pinning is what blocked non-`Copy` impls). Generic code that relied on
"the result is `Self`" must say so:

```rust
// before
fn f<T: CheckedMul>(...) -> T { ... a.checked_mul(b)? ... }
// now — restate the result type
fn f<T: CheckedMul + Mul<Output = T>>(...) -> T { ... a.checked_mul(b)? ... }
// (or, equivalently, bind through the trait:)
fn f<T: CheckedMul<Output = T>>(...) -> T { ... }
```

`T: PrimInt` is unaffected — it already bounds `CheckedAdd<Output = Self>`
etc. internally.

### 2.3 Implementing the traits on your own type

- **`Copy` type** (e.g. `FixedUInt`): implement as before; for the
  owned-returning traits your method still returns the concrete owned type
  (`-> Option<Self>`), which matches the trait's `Option<Self::Output>`
  because `Output == Self`. **Operator-backed traits need nothing extra**
  (the `Output` comes from your existing `Add`/`Sub`/… impl). **Fresh-`Output`
  traits require one line** — `type Output = Self;` — in your impl. The
  fresh-`Output` traits are:

  `CheckedNeg`, `WrappingNeg`, `OverflowingNeg`, `StrictNeg`, `Signum`,
  `CarrylessMul`, `CarryingCarrylessMul`, `Isqrt`, `CheckedIsqrt`,
  `NextPowerOfTwo`, `Midpoint`, `IsolateHighestOne`, `IsolateLowestOne`,
  `FunnelShl`, `FunnelShr`, `DepositBits`, `ExtractBits`, `ClampMagnitude`,
  the 20 mixed-sign flavor traits, and the `float_ops` owned-returning
  traits (`NextUp`, `NextDown`, `Maximum`, `Minimum`, `RoundTiesEven`,
  `Algebraic`, `Erf`, `Gamma`).

- **non-`Copy` type** (e.g. a heap bignum): implement the trait **for the
  reference type** so you don't consume operands:
  ```rust
  impl CheckedAdd for &BigInt {           // Self = &BigInt
      fn checked_add(self, v: &BigInt) -> Option<<&BigInt as Add>::Output> { … }
  }                                        // Output is the owned BigInt
  ```
  This is the capability that the associated `Output` exists to preserve.

### 2.4 Implementing the multiply family: `Widening*` (primitive-only) vs `Carrying*` (any width)

The multiply-helper traits come in two shapes, mirroring core exactly, and
which one you implement depends on whether your type has a wider primitive:

- **`Widening*` traits return a single double-width value** — `WideningMul`
  (`-> Self::Wide`) and `WideningCarrylessMul` (`-> Self::Wide`), mirroring
  core's `widening_mul` / `widening_carryless_mul` (`-> $WideT`, single-wide
  as of rust-lang/rust#156644). They exist only for types that *have* a
  next-larger primitive (`u8`–`u64`, `i8`–`i64`). **Arbitrary-precision or
  fixed-width-generic types (e.g. `FixedUInt<N>`) cannot implement them** —
  there is no `FixedUInt<2N>` without `generic_const_exprs` — and should
  **skip them.**

- **`Carrying*` traits return `(low, high)`** — `CarryingMul`
  (`carrying_mul` / `carrying_mul_add -> (Self::Unsigned, Self)`) and
  `CarryingCarrylessMul` (`-> (Self, Self)`), mirroring core's
  `carrying_mul` / `carrying_carryless_mul`. They return two `Self`-wide
  words and need **no** double-width type, so they are the form an
  arbitrary-precision type implements (compute the full product in N-bit
  space and split into low/high — exactly how this crate's own `u128` impls
  work, with 64-bit limbs and Karatsuba). `CarryingMul` carries a
  `type Unsigned` because core's *signed* `carrying_mul` returns the low
  word as the unsigned counterpart (`i32 -> (u32, i32)`).

- **`CarrylessMul` (`-> Self`)** is the truncated low-half product; it needs
  no wider type either (XOR `self << i` into an N-bit accumulator for each
  set bit of `rhs`; the bits that shift out are the discarded high half).

None of these is an obligation — the traits are independent. A complete
bignum surface implements `CarryingMul` (plus `CarryingCarrylessMul` /
`CarrylessMul` if wanted) and **skips `WideningMul` / `WideningCarrylessMul`**.
Generic code that must work over arbitrary-precision types should bound on
the `Carrying*` traits, not the `Widening*` ones.

### 2.5 Traits that were split or extracted

If you **implement** any of these, you now implement the pieces; if you only
**call** them generically, the supertrait relationship means existing
`T: Bundle` bounds keep working.

| Was | Now |
|---|---|
| `PrimInt` (all bit ops + `pow`) | **`PrimBits`** (count/leading/trailing ones+zeros, rotates, `swap_bytes`, `reverse_bits`, `to/from_be/le`, signed/unsigned shifts) **+ `PrimInt`** (`pow`, adds `PrimBits` + arithmetic). Implementors: split into a `PrimBits` impl and a `PrimInt` impl. |
| `Signed` (incl. `signum`) | **`Signum`** (`signum`) **+ `Signed`** (`abs`, `abs_sub`, `is_positive`, `is_negative`; requires `Signum`). |
| `Ilog` | `Ilog2` / `Ilog10` / `Ilog` |
| `IsolateOne` | `HighestOne` / `LowestOne` / `IsolateHighestOne` / `IsolateLowestOne` |
| `DepositExtractBits` | `DepositBits` / `ExtractBits` |
| `PowerOfTwo` | `IsPowerOfTwo` / `NextPowerOfTwo` |
| `Isqrt` (had `checked_isqrt`) | `Isqrt` (`isqrt`) + `CheckedIsqrt` (signed-only) |
| `CheckedCast` + `BoundedCast` | `CheckedCast<T>` / `StrictCast<T>` / `WrappingCast<T>` / `SaturatingCast<T>` (one method each) |
| `AddSigned`/`SubSigned`/`AddUnsigned`/`SubUnsigned` (5 flavors each) | 20 one-method traits: `{Checked,Wrapping,Overflowing,Saturating,Strict}{Add,Sub}{Signed,Unsigned}` |

**Note on concrete-type calls after extraction:** for a non-primitive type,
`bigint.count_ones()` with only `PrimInt` in scope no longer resolves
(`count_ones` moved to `PrimBits`). Import `PrimBits`, or
`use const_num_traits::prelude::*;`. Generic code (`fn f<T: PrimInt>`) and
the primitive types themselves are unaffected.

### 2.6 Methods that were removed (use the generic casts instead)

`CastSigned`/`CastUnsigned` are now **one-method** (the bit reinterpretation
only). Their checked/saturating/strict variants were removed; likewise
`Truncate`'s checked/saturating variants. They were exact duplicates of the
generic cast traits applied to the counterpart type:

```rust
x.checked_cast_signed()        →  CheckedCast::<iN>::checked_cast(x)
x.saturating_truncate::<i8>()  →  SaturatingCast::<i8>::saturating_cast(x)
```

### 2.7 New things you can rely on (additive — no migration needed)

- **`RingOps`** — `Add + Sub + Mul` aggregation (division-free `NumOps`);
  bound on it for code that may run on types without division.
- **`FromStrRadix`** — standalone parsing atom (`Num` keeps its own method
  too; with both in scope, disambiguate as `<T as Num>::from_str_radix`).
- **`FromAscii` / `FormatInto`** — byte-string integer parse/emit without
  `core::fmt`/UTF-8 machinery. `FormatInto` requires the `nightly-std`
  feature (it mirrors core's unstable `NumBuffer` API exactly).
- **`Parity`** (`is_odd`/`is_even`), the `float_ops` slice, the `ct`
  masked-return traits, and `impl_to_primitive_minimal!` /
  `impl_from_primitive_minimal!` helper macros.

### 2.8 Features and MSRV

| | |
|---|---|
| **MSRV** | **1.86** (was 1.60) |
| `nightly` | const traits (const-ness only; never changes which items exist) |
| `nightly-std` | `nightly` + unstable-std mirrors (`FormatInto`) |
| `ct` | `subtle`-backed masked-return traits (`ops::ct`) |
| `libm` | `Erf`/`Gamma` float impls |

### 2.9 What did *not* survive

- **num-traits drop-in signature compatibility.** Trait *names* and
  *supertrait bounds* are preserved (so `use`s and `T: CheckedAdd` bounds
  resolve), but the by-value signatures mean call sites change (2.1) and
  generic bounds may need a result-type restatement (2.2). The migration is
  mechanical, not structural.
- The internal `compat_canary` test was retired — it asserted
  signature-level num-traits compatibility, which is the thing we
  deliberately ended.

---

## Part 3 — Migration checklist

1. Bump your toolchain to **≥ 1.86**.
2. Drop the `&` from call sites: `x.op(&y)` → `x.op(y)`.
3. For generic functions bounded on the arithmetic traits that return `T`,
   add the result-type restatement: `T: CheckedMul + Mul<Output = T>`.
4. If you **implement** the traits:
   - add `type Output = Self;` to your impls of the **fresh-`Output`** traits
     in 2.3 (operator-backed ones need nothing);
   - split `PrimInt` impls into `PrimBits` + `PrimInt`, and add a `Signum`
     impl alongside `Signed`;
   - non-`Copy` types: implement for `&YourType` to stay non-destructive.
5. Replace removed cast/truncate variants with `CheckedCast<T>` /
   `SaturatingCast<T>` / `StrictCast<T>` / `WrappingCast<T>` (2.6).
6. For an arbitrary-precision / fixed-width-generic type, implement the
   `Carrying*` multiply traits (`(low, high)`) and **skip** the `Widening*`
   ones (single double-width type) — see 2.4.
7. If concrete-type method calls stop resolving after an extraction, add the
   atom trait to scope or `use const_num_traits::prelude::*;`.
8. Enable `nightly-std` if you use `FormatInto`; `ct`/`libm` as needed.

---

## Part 4 — Field guide: real migrations, by compiler error

Parts 1–3 are the design. This part is the **cookbook**, written from actually
migrating a fleet of `> 1M`-download downstream crates against the fork — the
`rust-num` shared deps (`num-integer`, `num-complex`, `num-bigint`) plus ten
leaf consumers (`bigdecimal`, `ndarray`, `arbitrary-int`, `float8`, `uom`,
`half`, `rust_decimal`, `rug`, `deranged`, and `fixed`, which implements the
*entire* trait surface). All thirteen compile against the fork. Every snippet
below is a real before/after. The migration is **entirely compiler-driven** —
fix the error rustc names, recompile, repeat. You will not have to
reverse-engineer intent.

### The one habit that matters: patch shared deps first

Most leaf crates (`bigdecimal`, `rug`, `ndarray`) don't break against
`num-traits` *directly* — they break **through** the `rust-num` family
(`num-integer`, `num-complex`, `num-bigint`). Patch those once and the
transitive breakage disappears from every leaf. A failing shared dep also
*short-circuits* the build, hiding the leaf's real (often smaller) error count
behind a misleadingly large or small number. Order of operations:

1. `num-integer` → `num-complex` → `num-bigint` (foundation up).
2. Then each leaf's own direct impls.

`num-integer`'s entire patch was **one line** (see E0271 below), and it alone
greened `rug`'s transitive breakage and most of `ndarray`'s.

### E0053 — "incompatible type for trait": drop the `&`

The headline change. Receivers and value args are by value now.

```rust
// num-bigint, before → after
fn checked_add(&self, v: &BigInt) -> Option<BigInt> { Some(self.add(v)) }
fn checked_add(self, v: BigInt)   -> Option<BigInt> { Some(self.add(v)) }
```

Applies to every `checked_*`, `wrapping_*`, `Euclid::*`, `Signed::abs/abs_sub/
is_positive/is_negative`, and `ToBytes::to_be_bytes`. The body usually needs no
change. (`FromBytes` is the exception — it stays borrowed; see its entry below.)

### E0046 — "missing trait items": supply a stripped default or new `Output`

A few defaults can't be const, so they're required now. Add the trivial bodies:

```rust
// bigdecimal — Zero / One
fn set_zero(&mut self) { *self = Zero::zero(); }
fn set_one(&mut self)  { *self = One::one(); }

// num-bigint — CheckedEuclid / Euclid (div_rem_euclid lost its default)
fn checked_div_rem_euclid(self, v: Self) -> Option<(Self, Self)> {
    if v.is_zero() { return None; }
    Some((self.clone().div_euclid(v.clone()), self.rem_euclid(v)))
}
fn div_rem_euclid(self, v: Self) -> (Self, Self) {
    (self.clone().div_euclid(v.clone()), self.rem_euclid(v))
}
```

If the trait gained a fresh `type Output` (Part 2.3), add `type Output = Self;`.

### E0407 — "method `signum` is not a member of trait `Signed`": move it to `Signum`

`signum` was extracted into its own atom. Lift it out of your `Signed` impl:

```rust
// num-bigint / bigdecimal
impl Signum for BigInt {
    type Output = BigInt;
    fn signum(self) -> BigInt { /* unchanged body */ }
}
impl Signed for BigInt { /* abs, abs_sub, is_positive, is_negative only */ }
```

This *also* clears the wave of **E0277 `T: Signum is not satisfied`** errors —
they were all just "`Signed` now requires a `Signum` impl that doesn't exist
yet." Add the impl and they vanish together. (`use num_traits::Signum;` too.)

### E0271 / E0308 / E0369 — Output is now a projection, not always `Self`

Two flavors, two fixes.

**(a) A bare literal can't infer its type backward through `abs`.** `Signed::abs`
returns `<Self as Neg>::Output`, which — unlike a concrete `-> Self` — can't
drive an untyped literal. Pin the literal:

```rust
// num-integer gcd(), the famous one-liner
return (1 << shift).abs();            // 1 defaults to i32 → mismatch
return ((1 as Self) << shift).abs();  // pinned to Self → resolves
```

**(b) Generic code assumed `Op<Output = Self>`.** Restate the result type on the
bound. This ripples one level up to callers, so fix the whole chain:

```rust
// num-complex l1_norm — abs() + abs() must land back on T
impl<T: Clone + Signed + Neg<Output = T>> Complex<T> { ... }

// bigdecimal checked_diff — checked_sub now yields Option<<T as Sub>::Output>
T: ToPrimitive + CheckedSub + Sub<Output = T> + Ord,
```

### E0507 / E0382 — "cannot move out of borrow" / "use of moved value"

A by-value method consuming a value you only borrow. Two idioms:

```rust
// 1. You own a clonable field — clone before the consuming call (bigdecimal)
int_val: self.int_val.clone().abs(),   // was self.int_val.abs()

// 2. You only need a cheap predicate — use a borrow-friendly inherent accessor
//    instead of the by-value trait method (num-bigint, bigdecimal)
if i.sign() == Sign::Minus { ... }     // was i.is_negative()
```

Idiom 2 is the better fix for hot predicate checks (`is_negative`/`is_positive`
on a `&self`): it avoids cloning a bignum just to test its sign.

### `ToBytes`/`FromBytes` — only `ToBytes` changed

`ToBytes::to_be_bytes` is by value (`self`); `FromBytes` is **unchanged from
upstream** — borrowed buffer, `type Bytes: ?Sized`. So a `to_be_bytes(&self)`
impl needs the usual E0053 `&`-drop, while `FromBytes` impls compile untouched.
Variable-length types keep the zero-alloc `[u8]` slice:

```rust
// num-bigint — FromBytes is upstream-identical (no change needed)
impl FromBytes for BigUint {
    type Bytes = [u8];                                       // unsized, borrowed
    fn from_be_bytes(bytes: &Self::Bytes) -> Self { Self::from_bytes_be(bytes) }
}
// ToBytes returns owned, so it's by value; add `impl ToBytes for &BigUint`
// if you need to read bytes without consuming the value.
impl ToBytes for BigUint {
    type Bytes = Vec<u8>;
    fn to_be_bytes(self) -> Self::Bytes { self.to_bytes_be() }   // self by value
}
```

(An earlier revision made `FromBytes` by-value/`Sized` too, which forced an
owned `Vec<u8>` + an allocation on variable-length types for no gain — since
`from_*_bytes` *reads* the buffer rather than reusing it. That was reverted; the
by-value convention applies to numeric operands and receivers, not to a
read-only input buffer.)

### E0034 — "multiple applicable items in scope": by-value made a same-named method ambiguous

The subtle one, and the dominant error in the worst-hit crate (`fixed`: **50 of
421**). If a downstream type defines its *own* `core`-shaped, by-value method of
the same name **on one of its own traits**, the fork's now-by-value trait method
has the **same signature shape** — so when both traits are in scope (typically
in generic code bounded by both), a bare `x.saturating_add(y)` call is ambiguous:

```text
2105 | op! { saturating_add, Add add, AddAssign add_assign }
     |       ^^^^^^^^^^^^^^ multiple `saturating_add` found
note: candidate #1 is defined in the trait `Fixed`         fn saturating_add(self, rhs: Self) -> Self;
note: candidate #2 is defined in the trait `SaturatingAdd`  fn saturating_add(self, v: Self) -> <Self as Add>::Output;
```

Under upstream's `(&self, &Self)` the two had *different* shapes and never
collided — **by value is precisely what creates the ambiguity.** Disambiguate
with **fully-qualified** syntax (or don't glob-import both traits):

```rust
<F as Fixed>::saturating_add(a, b)          // your own trait's method
<F as SaturatingAdd>::saturating_add(a, b)  // the num-traits one
```

Use the `<Type as Trait>::method` form, **not** the bare `Trait::method(a, b)`
form that rustc's own suggestion prints: in a generic context the bare form
fails with **E0782** ("expected a type, found a trait"). And if one shared macro
emits the call for a *mix* of methods — most on your trait, but one (e.g. `rem`)
that is really the plain operator living on `core::ops::Rem`, not your trait
(`<F as Fixed>::rem` → **E0576**) — pass the qualifying trait in as a macro
parameter so each call names the right one (`<F as Fixed>::wrapping_add` vs
`<F as Rem>::rem`).

An *inherent* method of the same name does **not** collide — it wins method-call
resolution outright, so concrete-type calls are usually fine (`half` hit zero
E0034 for exactly this reason). The ambiguity is specifically two **traits** both
in scope.

This bites any crate with std-shaped `saturating_*`/`wrapping_*`/etc. methods on
its own trait; in `fixed` all 50 are `{saturating,wrapping}_{add,sub,mul}` plus
`wrapping_neg` (no `saturating_neg`). (A *second*, unrelated E0034 source exists
but did not occur in any crate tested: `from_str_radix` lives on both `Num` and
`FromStrRadix`, so `T::from_str_radix` needs `<T as Num>::from_str_radix` when
both are in scope.)

### Effort calibration (measured, not guessed)

| Crate | Edits | Shape of the work |
|---|--:|---|
| `num-integer` | 1 | one literal pin (E0271) |
| `num-complex` | 2 | one bound + one clone |
| `ndarray` | 1 | one `set_zero` (the rest came free via the shared patches) |
| `arbitrary-int` | 4 | by-value drops in one macro (signed + unsigned) |
| `float8` | 4 | `Zero`/`One` defaults, two types |
| `uom` | 5 | Output-relaxation bounds (`Neg`/`Signum<Output = V>`) + `set_zero` |
| `half` | 7 | `Zero`/`One`/`ToBytes`/`Signed` on `f16`+`bf16`, one shared `impl_signed!` |
| `rust_decimal` | ~12 | full leaf migration on a `Copy` type |
| `bigdecimal` | ~12 | full leaf migration: Signum split, by-value drops, one rippled bound |
| `rug` | ~16 | the non-`Copy` GMP bignum; bodies borrow internally |
| `num-bigint` | ~20 | the canonical bignum migration; every pattern above |
| `fixed` | ~45 | **the whole surface**: 340 errors via one `impl_traits!` macro body, then 50 E0034 via the `op!` macros (`<F as Fixed>::` + a macro-param qualifier) |

Cost tracks how much `num_traits` surface the crate touches, nothing else —
`fixed` implements *every* trait and still came down to two macro bodies (390
errors → 0). None of it required a design decision: at no point was the fix
anything other than "type what the compiler told you to" (the one exception
being `fixed`'s `op!` macro, where the *kind* of qualifier — own trait vs the
`Rem` operator — had to be passed in). It is a deliberate **breaking** migration
(call sites change), so it belongs in a major version bump — but per-edit it is
mechanical and fast.
