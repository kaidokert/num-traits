#!/usr/bin/env bash
# Builds src/lib.rs to assembly for Cortex-M targets and checks that the
# typestate accessors let the optimiser remove what each function's name says:
#   nopanic_*  must not reference a panic,
#   nomask_*   must not mask the shift amount (`and ..., #31` / `#63`).
# Requires the thumbv6m-none-eabi and thumbv7em-none-eabihf targets.
set -euo pipefail
cd "$(dirname "$0")"

status=0
for target in thumbv6m-none-eabi thumbv7em-none-eabihf; do
    cargo rustc --quiet --release --target "$target" -- --emit asm
    asm=$(ls -t "target/$target/release/deps/"typestate_codegen-*.s | head -n 1)
    for fn in $(grep -oE '^(nopanic|nomask)_[a-z0-9_]+:' "$asm" | tr -d ':'); do
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
