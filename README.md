# const-num-traits

[![crate](https://img.shields.io/crates/v/const-num-traits.svg)](https://crates.io/crates/const-num-traits)
[![documentation](https://docs.rs/const-num-traits/badge.svg)](https://docs.rs/const-num-traits)
[![minimum rustc 1.86](https://img.shields.io/badge/rustc-1.86+-red.svg)](https://rust-lang.github.io/rfcs/2495-min-rust-version.html)
[![build status](https://github.com/kaidokert/num-traits/actions/workflows/master.yaml/badge.svg)](https://github.com/kaidokert/num-traits/actions)

Numeric traits for generic mathematics in Rust.

A fork of [`num-traits`](https://github.com/rust-num/num-traits). It:

- compiles to ordinary traits on stable Rust — no nightly required — and on a
  nightly toolchain the same surface becomes `const trait`, usable in `const`
  (via the [`c0nst`](https://crates.io/crates/c0nst) macro);
- exposes newer `core` integer and float operations that `num-traits` yet does not,
  such as `wide_mul`;
- is intentionally API-breaking with `num-traits`, to keep the `const` and
  non-`const` views of the API as close as possible.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
const-num-traits = "0.1"
```

Enable the `const trait` surface with the `nightly` feature on a nightly compiler:

```toml
[dependencies]
const-num-traits = { version = "0.1", features = ["nightly"] }
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

## Compatibility

The `const-num-traits` crate is tested for rustc 1.86 and greater.

## License

Licensed under either of

 * [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
 * [MIT license](http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
