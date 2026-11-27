//! Builds the library to assembly for embedded targets and checks that the
//! typestate accessors let the optimiser remove what each function's name
//! says:
//!   `nopanic_*`  must not reference a panic,
//!   `nomask_*`   must not mask the shift amount to 5 or 6 bits.
//!
//! Run with `cargo run --bin check`. Requires the targets in [`TARGETS`].

use std::path::Path;
use std::process::{exit, Command};

/// ARM register shifts read 8 bits of the amount, so a shift by an unproven
/// amount gets masked; RISC-V's read 5, so only multi-word shifts (`i64` on
/// rv32) do.
const TARGETS: [&str; 3] = [
    "thumbv6m-none-eabi",
    "thumbv7em-none-eabihf",
    "riscv32imac-unknown-none-elf",
];

fn main() {
    // Taken from the library source, so every function added there is
    // checked.
    let fns: Vec<&str> = include_str!("../lib.rs")
        .lines()
        .filter_map(|line| line.trim().strip_prefix("pub fn "))
        .map(|rest| rest.split('(').next().unwrap_or(rest))
        .filter(|name| name.starts_with("nopanic_") || name.starts_with("nomask_"))
        .collect();
    if fns.is_empty() {
        eprintln!("FAIL no nopanic_*/nomask_* functions in src/lib.rs");
        exit(1);
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut failed = false;
    for target in TARGETS {
        let asm = assembly(manifest_dir, target);
        for name in &fns {
            let Some(body) = function_body(&asm, name) else {
                println!("FAIL {target} {name}: not in the assembly");
                failed = true;
                continue;
            };
            let bad = if name.starts_with("nopanic_") {
                body.iter()
                    .filter(|line| line.contains("panic"))
                    .copied()
                    .collect()
            } else {
                shift_masks(&body)
            };
            if bad.is_empty() {
                println!("ok   {target} {name}");
            } else {
                println!("FAIL {target} {name}");
                for line in bad {
                    println!("    {line}");
                }
                failed = true;
            }
        }
    }
    exit(i32::from(failed));
}

/// Builds the library for `target` and returns its assembly.
fn assembly(manifest_dir: &Path, target: &str) -> String {
    // An explicit path: Cargo's layout of intermediate files varies by version.
    let asm = manifest_dir.join(format!("target/typestate_codegen-{target}.s"));
    // Cleaning the crate makes rustc run, and write the file, every time.
    cargo(
        &[
            "clean",
            "--quiet",
            "--release",
            "--target",
            target,
            "-p",
            "typestate-codegen",
        ],
        manifest_dir,
    );
    let _ = std::fs::remove_file(&asm);
    // Absent when `CARGO_TARGET_DIR` points elsewhere, and rustc won't create it.
    if let Some(dir) = asm.parent() {
        std::fs::create_dir_all(dir).unwrap_or_else(|e| {
            eprintln!("FAIL creating {}: {e}", dir.display());
            exit(1);
        });
    }
    let emit = format!("asm={}", asm.display());
    cargo(
        &[
            "rustc",
            "--quiet",
            "--lib",
            "--release",
            "--target",
            target,
            "--",
            "--emit",
            &emit,
        ],
        manifest_dir,
    );
    std::fs::read_to_string(&asm).unwrap_or_else(|e| {
        eprintln!("FAIL {target}: no assembly at {}: {e}", asm.display());
        exit(1);
    })
}

fn cargo(args: &[&str], dir: &Path) {
    // `CARGO` is the cargo running this binary, so the build uses the same
    // toolchain.
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let status = Command::new(cargo)
        .args(args)
        .current_dir(dir)
        .status()
        .unwrap_or_else(|e| {
            eprintln!("FAIL running cargo {}: {e}", args[0]);
            exit(1);
        });
    if !status.success() {
        eprintln!("FAIL cargo {} exited with {status}", args[0]);
        exit(1);
    }
}

/// The lines between `name:` and the function's `.Lfunc_end` label.
fn function_body<'a>(asm: &'a str, name: &str) -> Option<Vec<&'a str>> {
    let label = format!("{name}:");
    let mut lines = asm.lines().skip_while(|line| *line != label);
    lines.next()?;
    Some(
        lines
            .take_while(|line| !line.starts_with(".Lfunc_end"))
            .collect(),
    )
}

/// Instructions that mask a shift amount to 5 or 6 bits. thumbv7em and RISC-V
/// mask with an immediate (`and r1, r1, #31`, `andi a3, a2, 63`); thumbv6m has
/// none, so it loads the constant (`movs r2, #31`) and then `ands` with it.
fn shift_masks<'a>(body: &[&'a str]) -> Vec<&'a str> {
    let mnemonic = |line: &str| line.split_whitespace().next().unwrap_or("").to_owned();
    let mask_imm = |line: &str| {
        let imm = line.rsplit([' ', '\t', ',']).next().unwrap_or("");
        matches!(imm.trim_start_matches('#'), "31" | "63")
    };

    let mut bad: Vec<&str> = body
        .iter()
        .filter(|line| {
            matches!(mnemonic(line).as_str(), "and" | "ands" | "and.w" | "andi") && mask_imm(line)
        })
        .copied()
        .collect();
    let loads_mask = body.iter().any(|line| {
        let mut parts = line.split_whitespace();
        parts.next() == Some("movs")
            && parts.next().is_some_and(|reg| reg.starts_with('r'))
            && mask_imm(line)
    });
    if loads_mask && body.iter().any(|line| mnemonic(line) == "ands") {
        bad.extend(
            body.iter()
                .filter(|line| mnemonic(line) == "ands" || mask_imm(line))
                .copied(),
        );
    }
    bad
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_v7em_immediate_mask() {
        assert_eq!(
            shift_masks(&["\tand\tr1, r1, #31", "\tlsls\tr0, r1"]),
            ["\tand\tr1, r1, #31"]
        );
        assert_eq!(shift_masks(&["\tand.w\tr1, r1, #63"]).len(), 1);
    }

    #[test]
    fn finds_the_riscv_immediate_mask() {
        assert_eq!(
            shift_masks(&["\tandi\ta3, a2, 63", "\tsra\ta0, a1, a3"]),
            ["\tandi\ta3, a2, 63"]
        );
        assert!(shift_masks(&["\tandi\ta3, a2, 15"]).is_empty());
        assert!(shift_masks(&["\txori\ta1, a1, 31"]).is_empty());
    }

    #[test]
    fn finds_the_v6m_loaded_mask() {
        let body = ["\tmovs\tr2, #31", "\tands\tr2, r1", "\tlsls\tr0, r2"];
        assert_eq!(shift_masks(&body), ["\tmovs\tr2, #31", "\tands\tr2, r1"]);
    }

    #[test]
    fn ignores_other_ands_and_constants() {
        assert!(shift_masks(&["\tands\tr0, r1", "\tlsls\tr0, r1"]).is_empty());
        assert!(shift_masks(&["\tmovs\tr2, #31", "\tlsls\tr0, r2"]).is_empty());
        assert!(shift_masks(&["\tand\tr1, r1, #15"]).is_empty());
    }

    #[test]
    fn body_stops_at_the_function_end() {
        let asm = "a:\n\tx\n.Lfunc_end0:\nb:\n\ty\n.Lfunc_end1:\n";
        assert_eq!(function_body(asm, "a"), Some(vec!["\tx"]));
        assert_eq!(function_body(asm, "b"), Some(vec!["\ty"]));
        assert_eq!(function_body(asm, "c"), None);
    }
}
