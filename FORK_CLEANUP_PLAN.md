# Fork cleanup plan — `const-num-traits` clean break

Goal: publish this fork with the smallest, most defensible long-lived footprint.
No leftover upstream scaffolding, no untested feature surface, no API we aren't
prepared to keep stable. Items are ordered so that anything affecting the
published package or the public API lands **before** the first crates.io release.

---

## 1. CI actually tests what the fork ships — DONE (pre-commit-centric)

**Design change from the original plan:** the hand-rolled `ci/test_full.sh`
was dropped entirely rather than fixed. A bespoke shell script as the canonical
"what passes" is the upstream smell we're shedding. Instead:

- **`.pre-commit-config.yaml` is the single source of truth** for the enforced
  gate (fmt, clippy on `std libm ct` and `no_std libm ct` with `-D warnings`,
  `cargo test` on `std libm ct`). Local `cargo` hooks (not the stale pinned
  `doublify/pre-commit-rust`) so they target the fork's real feature set.
  Runs on `git commit` locally and again in CI — same checks both places,
  platform-independent, auto-triggered.
- **`.github/workflows/ci.yaml` is now a thin backstop**, consolidated from the
  old `pr.yaml` / `main.yaml` / `ci.yaml` (all three deleted → one file, one
  trigger set: `pull_request` + `push:main` + `merge_group` + weekly `schedule`).
  Jobs:
  - `gate` — runs `pre-commit run --all-files` (the enforced gate, in CI too)
  - `test` — MSRV / stable / beta matrix, feature combos as inline `cargo` steps
  - `nightly` — the const surface (`--features nightly`, runs
    `tests/const_nightly.rs`) gates the run (see §4: `nightly-std` removed)
  - `cross` — i586 (explicit stable features, **not** `--all-features`) +
    thumbv6m no_std incl. `ct`
  - `success` — branch-protection summary gate
- `ci/test_full.sh` and `ci/rustup.sh` deleted; `ci/` dir removed.
- README build badge repointed `main.yaml` → `ci.yaml`.

## 2. Delete `build.rs` + autocfg — DONE

Both probes were unconditionally true at MSRV 1.86 (`total_cmp` stable 1.62,
`core::num::Saturating` stable 1.74).

- `build.rs` deleted; `build = "build.rs"` and `[build-dependencies] autocfg`
  removed from `Cargo.toml`.
- Inlined all cfg branches (kept the "has" path, deleted fallbacks):
  `src/float.rs` `total_cmp` macro (also simplified its now-unused
  `$I/$U/$bits` params), and **12** `has_num_saturating` sites across
  `src/lib.rs` (4) and `src/identities.rs` (8) — the original plan only listed
  the lib.rs ones; identities.rs was found during the sweep.
- Verified `grep -rn "has_" src` is clean. `cargo build` + `cargo test --lib`
  green; dependents no longer run a build script at all.

## 3. `Cargo.toml` metadata (blocker — bakes into the published package)

- [ ] `authors = ["The Rust Project Developers"]` → the fork's author.
      Upstream attribution stays in README + the license headers (already
      present)
- [ ] `[package.metadata.docs.rs]`: `features = ["std"]` hides `ops::ct` from
      docs.rs entirely. Use `features = ["std", "ct"]`, and consider
      `--cfg docsrs` + `doc(cfg(...))` annotations so gated items are labeled
- [ ] Add `.pre-commit-config.yaml` to `exclude` (currently ships in the
      `.crate` package)
- [ ] Sanity-check the final package contents: `cargo package --list`

## 4. Drop the `nightly-std` feature / `format_into` module (decision)

The original instinct that started this review. `src/ops/format_into.rs` is the
**only** module that names unstable std types directly
(`core::fmt::NumBuffer` / `NumBufferTrait`), so it is the only part of the
crate a nightly bump can outright *break* rather than merely churn. Everything
else hand-rolls bodies and mirrors API shape only.

> **Live evidence (found during the §1 CI work, nightly 1.98.0 / 2026-06-19):**
> `nightly-std` **failed to compile** — `NumBuffer` / `NumBufferTrait` had moved
> behind the `fmt_internals` unstable feature, which the compiler itself flags
> as *"internal to the compiler or standard library"* (i.e. never intended for
> external crates), while `int_format_into` had meanwhile **stabilized**. So
> this module now tracks a permanently-internal API.

**RESOLVED — removed.** Two decisive problems, both structural:

1. The `nightly-std` feature name promised a *category* ("nightly features
   needing std") but gated exactly one trait (`FormatInto`) in one module —
   a name decoupled from its functionality.
2. The module's stated design ("mirror std exactly rather than invent a
   crate-owned buffer") is only expressible by naming `NumBuffer` /
   `NumBufferTrait`, which require the **compiler-internal** `fmt_internals`
   feature. You cannot keep the module's purpose and drop that dependency —
   they are the same thing. (`int_format_into` meanwhile stabilized, making
   the gate half-redundant and half-internal.)

Depending on a `#[unstable]`-flagged *compiler-internal* feature is
categorically wrong for a stable-oriented fork, so the module was deleted
rather than patched:

- Removed `src/ops/format_into.rs`, the `nightly-std` feature (Cargo.toml),
  the `feature(int_format_into, fmt_internals)` attr, the `pub mod`
  (`src/ops/mod.rs`), both re-exports (`src/lib.rs`), and the CI best-effort
  step. `grep` confirms zero references remain.
- Bonus: plain `nightly` is now a *pure* const-ness upgrade — its
  `#![feature(...)]` list is only `const_trait_impl` and friends, no library
  features at all. That restores the crate's clean invariant ("`nightly` never
  changes which items exist, only their const-ness").
- If decimal-to-buffer formatting is ever wanted, it should be a **stable**,
  crate-owned buffer API (no unstable anything) or wait for the std trait to
  stabilize — not a mirror of internal types.

## 5. Scope decision: typestate / personality / clmul (decision, pre-0.1)

These are the largest additions to the semver surface and the least "numeric
traits"-shaped. 0.1 on crates.io starts the clock on keeping them.

- [ ] **`src/personality.rs`** — its own docs concede "nothing about the marker
      itself is numeric" and already sketch the exit ("lifting it into its own
      crate is mechanical"). Decide: keep, or extract to a companion crate and
      re-export
- [ ] **`src/ops/typestate.rs`** (1,308 lines) + `examples/typestates.rs` +
      `tests/typestate_generic_carrier.rs` — `Odd`/`Even` docs reference a
      "modmath layer" that lives *downstream*, i.e. this exists to serve the
      bigint project. Decide: keep here, or move to the companion crate with
      personality
- [ ] **`src/ops/clmul.rs`** — GF(2)/polynomial multiplication is
      crypto-domain, not generic math. Weaker case for removal (it's small and
      self-contained); decide and record the rationale either way
- [ ] Whatever stays: one pass to confirm every `pub` item is intentional API
      (`cargo doc` review, or `cargo public-api` diff against intent)

## 6. README / docs accuracy

- [ ] README says the fork exposes ops "such as `wide_mul`" — the public
      method is `widening_mul` (`wide_mul_u128` is a `pub(crate)` helper)
- [ ] Verify the stabilization-version claims that justify hand-rolled bodies
      (they're load-bearing for the MSRV story):
  - `src/ops/strict.rs` — "strict_* … stable in std since Rust 1.91"
  - `src/ops/carrying.rs` — "for unsigned types these stabilized in Rust 1.91"
- [ ] Drop `#![doc(html_root_url = ".../0.1")]` from `src/lib.rs` — per-release
      maintenance chore the ecosystem has abandoned; rustdoc resolves links
      fine without it
- [ ] Start a fresh `RELEASES.md`/`CHANGELOG.md` for the fork (upstream's was
      deleted — good; a one-entry 0.1.0 changelog documenting the break from
      `num-traits` makes the fork's intent legible)

## 7. Dependency posture (note, not action)

- `c0nst` 0.2.1 is a hard (non-optional) pre-1.0 proc-macro dependency in the
  critical path of every build, stable included, and it drags `proc-macro2`.
  Acceptable (it's the mechanism of the fork), but: pin expectations for its
  own stability before 1.0 of this crate, and consider documenting in the
  README why the dep is required even for stable-only users
- `subtle` is optional and correctly `default-features = false` — fine
- `libm` — inherited, fine

## 8. Pre-publish checklist

- [ ] §3 and §6 done; §5 decided (§1, §2, §4 already merged)
- [ ] `cargo test` on 1.86.0, stable; `cargo +nightly test --features nightly`;
      `--features ct` variants
- [ ] `cargo build --target thumbv6m-none-eabi --no-default-features`
- [ ] `cargo package --list` reviewed; `cargo publish --dry-run` clean
- [ ] docs.rs metadata verified by `cargo +nightly doc` with the same feature
      set locally
