# HOW TO FIX: `impl c0nst Trait` doesn't build on nightly ≥ 2026-06-19

Symptom (any feature that enables `c0nst`'s nightly path — typically `--features nightly`):

```text
error: expected a trait, found type
  --> src/bounds.rs:31:1
   | impl<T: [c0nst] Bounded> c0nst LowerBounded for T {
```

…repeated dozens to thousands of times, one per `c0nst::c0nst!` block.

## Why

rustc moved the const-impl keyword from *after* `impl` to *before* it,
joining `unsafe` / `default` as a pre-`impl` modifier, then removed the
old position:

- **PR:** <https://github.com/rust-lang/rust/pull/158009> — *"Reject `impl
  const Trait` since the right syntax is `const impl Trait` now"* (oli-obk,
  merged 2026-06-18, in `nightly-2026-06-19` and later).
- **Commit:** `a88521a6f6e`.

The `c0nst` macro crate (0.2.1, latest published) emits `impl const Trait`
tokens, which the new parser rejects. No migration diagnostic — the rustc
PR author argued rustfmt rewrites the old form to the new form
automatically; macro output never sees rustfmt, so we get the bare
"expected a trait, found type" instead.

See `NIGHTLY_PIN.md` for the full bisect / why-this-pinned story.

## The fix — mechanical source migration

Move `c0nst` from after `impl` to before it. The `c0nst` 0.2.1 macro
accepts **both** positions; the new position expands to `const impl …`
on nightly (parser-correct) and to plain `impl …` on stable.

|              | OLD (broken on nightly ≥ 2026-06-19) | NEW (works everywhere) |
|---           |---                                   |---                     |
| Generic      | `impl<T> c0nst Trait for Type { … }` | `c0nst impl<T> Trait for Type { … }` |
| Non-generic  | `impl c0nst Trait for Type { … }`    | `c0nst impl Trait for Type { … }` |

The `pub c0nst trait …` *declaration* syntax and `[c0nst]` bounds are
**unchanged** — #158009 only moved the keyword on `impl`. Don't touch
the trait declarations.

### One-shot from the repo root

Both forms are single-line in this codebase. No nested `<>` inside the
generics list. So two perl substitutions cover everything:

```bash
# Generic form
grep -rlE '^[[:space:]]*impl<[^>]*>[[:space:]]+c0nst[[:space:]]+[A-Z]' src/ \
  | xargs perl -i -pe 's/^(\s*)impl<([^>]*)>\s+c0nst\s+([A-Z]\S*)/\1c0nst impl<\2> \3/'

# Non-generic form
grep -rlE '^[[:space:]]*impl[[:space:]]+c0nst[[:space:]]+[A-Z]' src/ \
  | xargs perl -i -pe 's/^(\s*)impl\s+c0nst\s+([A-Z]\S*)/\1c0nst impl \2/'
```

### Verify

```bash
# Expect empty:
grep -rE '^[[:space:]]*impl[[:space:]]*<[^>]*>?\s*c0nst\s+[A-Z]' src/

# Expect a non-empty count (one per migrated site):
grep -rcE '^[[:space:]]*c0nst impl' src/ | grep -v ':0$'
```

### Validate

```bash
cargo build                                       # stable — unchanged behavior
cargo +nightly build --features nightly           # nightly — should now succeed
```

If nightly still fails, look for **multi-line** `impl` heads (a long
generic list wrapped across lines) — the perl one-liner only handles the
single-line case. Hand-edit any survivors flagged by the verify-grep.

## After fixing

Delete `rust-toolchain.toml` and bump the documented minimum nightly to
`nightly-2026-06-19`. The source now requires the new parser, so older
nightlies will fail just as nightly ≥ 2026-06-19 used to.

## Reference fix in a downstream consumer

`fixed-bigint`'s `experiment/external-const-num-traits` branch, commit
`9fd4a5d` ("Sync with v0.3.1: nightly const impl syntax + version +
pre-commit hook"), applied this exact migration across 104 sites in 21
files. Use it as a worked example if anything here is ambiguous.
