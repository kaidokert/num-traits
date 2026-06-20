# Nightly toolchain pin

`rust-toolchain.toml` pins this crate to **`nightly-2026-06-18`**. This is a
**temporary** workaround for a const-trait *syntax* change in rustc that the
[`c0nst`](https://crates.io/crates/c0nst) macro crate hasn't caught up to.

Plain stable builds (`default` features) are unaffected by the underlying bug —
the pin only matters for the `nightly` / `nightly-std` features (the
`const trait` / `impl const` surface). On nightly **≥ 2026-06-19** the crate
fails to build those features.

## Symptom

```text
$ cargo +nightly build --features nightly
error: expected a trait, found type
  --> src/bounds.rs:31:1
   | impl<T: [c0nst] Bounded> c0nst LowerBounded for T {
```

…on every `impl c0nst Trait` the `c0nst!` macro expands (no migration hint —
see below for why).

## Root cause: rust-lang/rust#158009

rustc **removed the `impl const Trait for T` syntax** in favor of
**`const impl Trait for T`** — the `const` qualifier moved *before* `impl`,
joining `unsafe`/`default` as a pre-`impl` modifier.

- PR: <https://github.com/rust-lang/rust/pull/158009>
- Commit: `a88521a6f6e` — *"Reject `impl const Trait` since the right syntax is
  `const impl Trait` now"*
- Author: Oli Scherer (`oli-obk`), **2026-06-17**
- Parser change: `compiler/rustc_parse/src/parser/item.rs` (the rest of the PR
  mass-migrates `library/core`/`alloc`/stdarch's own const-trait impls)

The `const trait` *declaration* syntax and `[const]` *bounds* were **not**
changed — only the impl keyword position.

The author deliberately shipped **no migration diagnostic**:

> "rustfmt immediately formats `impl const Trait` to `const impl Trait`. This is
> also the reason I expect that this change will break very little nightly
> users, so we don't even need to add some helpful diagnostic."

A proc-macro that *emits* `impl const` tokens (`c0nst`) is exactly the
unaccounted-for case: its output never passes through rustfmt, so it gets
neither the auto-migration nor a useful error — just the bare
`expected a trait, found type`.

## Bisect

| nightly | rustc | `--features nightly` |
|---|---|---|
| `nightly-2026-06-18` | `1.98.0-nightly (c1b22f44c 2026-06-17)` | ✅ builds |
| **`nightly-2026-06-19`** | `1.98.0-nightly (f428d123a)` | ❌ breaks |

The change landed in the rustc commit window `c1b22f44c..f428d123a`.

## Why `c0nst` breaks

`c0nst` ([`npmccallum/c0nst`](https://github.com/npmccallum/c0nst), `0.2.1` — the
latest published) does *positional* `c0nst` → `const` substitution. Our source
writes `impl c0nst Trait`, which expands to `impl const Trait` — precisely the
syntax #158009 removed. `c0nst` is not our crate, and no fixed version is
published.

## Unpinning — the forward path

Pick one, then delete `rust-toolchain.toml`:

1. **Wait for `c0nst`** to emit `const impl Trait`. File/track an issue on
   `npmccallum/c0nst` citing #158009; bump the dependency when fixed. Cleanest —
   our source stays `impl c0nst Trait`.

2. **Migrate our source** `impl c0nst Trait` → `c0nst impl Trait` (move the
   keyword *before* `impl`). **Verified** to work with the *current* `c0nst`
   0.2.1: on stable the keyword is dropped (`impl Trait`), on nightly it becomes
   `const impl Trait`. Mechanical (dozens of `impl c0nst` sites, including the
   generic `impl<…> c0nst Trait` → `c0nst impl<…> Trait` form). This ties the
   crate to nightly **≥ 2026-06-19**.

Either way, `const impl Trait` is now the canonical, rustfmt-enforced syntax.

## CI note

`rust-toolchain.toml` overrides the toolchain for any `cargo` invocation in this
directory **including CI**, so the stable/MSRV jobs (`pr.yaml`, `ci.yaml`,
`master.yaml`) will run on the pinned nightly unless they explicitly override it
(e.g. via `RUSTUP_TOOLCHAIN` per job). Account for this while the pin is in
place, or scope the pin out of the stable CI.
