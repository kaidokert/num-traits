# const-num-traits

A fork of [`num-traits`](https://github.com/rust-num/num-traits) with two goals:

1. **Const traits**: the integer trait surface is `const trait` / `impl const`
   on nightly (via the [`c0nst`](https://crates.io/crates/c0nst) macro crate),
   while compiling to byte-identical plain traits on stable.
2. **Complete coverage of modern Rust's integer operations**: every inherent
   numeric method on the primitive integers in current nightly std — stable or
   unstable — has a trait here, including the full
   checked/wrapping/saturating/overflowing/strict variant matrix (`pow`,
   `abs`, division, shifts, Euclidean division), bigint helpers
   (`carrying_add`, `borrowing_sub`, `carrying_mul[_add]`, `widening_mul`,
   carry-less multiplication), rounding (`div_ceil`/`div_floor`/`div_exact`,
   `next_multiple_of`, `midpoint`), `ilog*`/`isqrt`/power-of-two ops,
   bit manipulation (unbounded/funnel/exact shifts, bit isolation,
   `bit_width`, PDEP/PEXT), mixed-signedness arithmetic, and conversions
   (`cast_signed`/`cast_unsigned`, `widen`/`truncate`, generic
   checked/wrapping/saturating/strict casts, `abs_diff`, `unsigned_abs`,
   `clamp_magnitude`).

Operations whose std counterparts are newer than the MSRV (1.86) or still
unstable are hand-rolled with the same algorithms and panic messages as core,
so they work on stable Rust and inside `const` on nightly. The `unsafe`
`unchecked_*` family is deliberately not exposed.

Enable const traits with the `nightly` feature on a nightly compiler:

```toml
[dependencies]
const-num-traits = { version = "0.1", features = ["nightly"] }
```

Numeric traits for generic mathematics in Rust.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
const-num-traits = "0.1"
```

## Features

This crate can be used without the standard library (`#![no_std]`) by disabling
the default `std` feature. Use this in `Cargo.toml`:

```toml
[dependencies.const-num-traits]
version = "0.1"
default-features = false
# features = ["libm"]    # <--- Uncomment if you wish to use `Float` and `Real` without `std`
```

The `Float` and `Real` traits are only available when either `std` or `libm` is enabled.

The `FloatCore` trait is always available.  `MulAdd` and `MulAddAssign` for `f32`
and `f64` also require `std` or `libm`, as do implementations of signed and floating-
point exponents in `Pow`.

## Releases

Release notes are available in [RELEASES.md](RELEASES.md).

## Compatibility

The `const-num-traits` crate is tested for rustc 1.86 and greater (upstream
`num-traits` supports 1.60+; this fork raised the floor to delegate to
inherent std methods stabilized through 1.86).

## License

Licensed under either of

 * [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
 * [MIT license](http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
