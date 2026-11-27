#!/usr/bin/env bash
# Builds src/lib.rs to assembly for Cortex-M targets and checks that the
# typestate accessors let the optimiser remove what each function's name says:
#   nopanic_*  must not reference a panic,
#   nomask_*   must not mask the shift amount (`and ..., #31` / `#63`).
# Requires the thumbv6m-none-eabi and thumbv7em-none-eabihf targets.
set -euo pipefail
cd "$(dirname "$0")"

# Every checked function must appear in the assembly, so a renamed or
# inlined-away function fails instead of passing unchecked.
fns=$(grep -oE 'pub fn (nopanic|nomask)_[a-z0-9_]+' src/lib.rs | sed 's/^pub fn //')
if [ -z "$fns" ]; then
    echo "FAIL no nopanic_*/nomask_* functions in src/lib.rs"
    exit 1
fi

status=0
for target in thumbv6m-none-eabi thumbv7em-none-eabihf; do
    # An explicit path: Cargo's layout of intermediate files varies by version.
    asm="target/typestate_codegen-$target.s"
    # Cleaning this crate makes rustc run, and write the file, every time.
    cargo clean --quiet --release --target "$target" -p typestate-codegen
    rm -f "$asm"
    cargo rustc --quiet --release --target "$target" -- --emit "asm=$asm"
    if [ ! -f "$asm" ]; then
        echo "FAIL $target: rustc wrote no $asm"
        exit 1
    fi
    for fn in $fns; do
        if ! grep -qx "$fn:" "$asm"; then
            echo "FAIL $target $fn: not in the assembly"
            status=1
            continue
        fi
        body=$(awk -v f="$fn:" '$0 == f {on = 1; next} on && /^\.Lfunc_end/ {exit} on' "$asm")
        bad=""
        case "$fn" in
            nopanic_*)
                bad=$(grep -E 'panic' <<<"$body" || true) ;;
            nomask_*)
                # thumbv7em masks with an immediate (`and r1, r1, #31`);
                # thumbv6m has none, so it loads the constant (`movs r2, #31`)
                # and then `ands` with it.
                bad=$(grep -E '^[[:space:]]*and(s|\.w)?[[:space:]].*#(31|63)$' <<<"$body" || true)
                if grep -qE '^[[:space:]]*movs[[:space:]]+r[0-9]+, #(31|63)$' <<<"$body" &&
                    grep -qE '^[[:space:]]*ands[[:space:]]' <<<"$body"; then
                    bad+=$(grep -E '#(31|63)$|^[[:space:]]*ands' <<<"$body")
                fi ;;
        esac
        if [ -n "$bad" ]; then
            echo "FAIL $target $fn"
            sed 's/^/    /' <<<"$bad"
            status=1
        else
            echo "ok   $target $fn"
        fi
    done
done
exit "$status"
