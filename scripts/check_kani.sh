#!/usr/bin/env sh
set -eu

required=false
case "${1:-}" in
    "")
        ;;
    --required)
        required=true
        ;;
    *)
        echo "usage: scripts/check_kani.sh [--required]" >&2
        exit 2
        ;;
esac

fail_or_skip() {
    message="$1"
    if [ "$required" = true ]; then
        echo "Kani checks: required; $message" >&2
        exit 1
    fi
    echo "Kani checks: skipping; $message" >&2
    exit 0
}

if [ ! -s crates/hashavatar-core/src/kani_proofs.rs ]; then
    fail_or_skip "crates/hashavatar-core/src/kani_proofs.rs is not present"
fi

kani_toolchain="${HASHAVATAR_KANI_TOOLCHAIN:-1.90.0-x86_64-unknown-linux-gnu}"

if ! rustup toolchain list | grep -q "^$kani_toolchain"; then
    fail_or_skip "Rust toolchain $kani_toolchain is not installed"
fi

cargo_kani() {
    rustup run "$kani_toolchain" cargo kani "$@"
}

if ! actual_kani="$(cargo_kani --version 2>/dev/null)"; then
    fail_or_skip "cargo kani is not installed"
fi

expected_kani="cargo-kani 0.67.0"
if [ "$actual_kani" != "$expected_kani" ]; then
    fail_or_skip "expected $expected_kani, found $actual_kani"
fi

log="$(mktemp)"
trap 'rm -f "$log"' EXIT

echo "Kani checks: using Rust toolchain $kani_toolchain"

harnesses="
accessory_stack_capacity_is_fail_closed
catalog_byte_selection_stays_in_frozen_bounds
request_dimension_admission_is_exact
unit_fixed_conversion_stays_in_closed_interval
fixed_lerp_stays_between_small_positive_bounds
pixel_center_is_inside_its_pixel
validated_scene_report_has_exact_rgba_size
source_over_channels_remain_canonical_bytes
alpha_multiplication_stays_bounded
"

status=0
for harness in $harnesses; do
    echo "Kani checks: harness $harness" >>"$log"
    if cargo_kani -p hashavatar-core --harness "$harness" --no-default-features >>"$log" 2>&1; then
        :
    else
        status="$?"
        break
    fi
done

if [ "$status" -eq 0 ]; then
    cat "$log"
    exit 0
fi

if grep -q "Kani Rust Verifier" "$log" && grep -q "requires rustc" "$log"; then
    cat "$log"
    fail_or_skip "installed Kani compiler is incompatible with this crate's Rust version"
fi

cat "$log"
exit "$status"
