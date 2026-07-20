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

if [ ! -s src/kani_proofs.rs ]; then
    fail_or_skip "src/kani_proofs.rs is not present"
fi

kani_toolchain="${HASHAVATAR_KANI_TOOLCHAIN:-1.90.0-x86_64-unknown-linux-gnu}"

if ! rustup toolchain list | grep -q "^$kani_toolchain"; then
    fail_or_skip "Rust toolchain $kani_toolchain is not installed"
fi

cargo_kani() {
    rustup run "$kani_toolchain" cargo kani "$@"
}

if ! cargo_kani --version >/dev/null 2>&1; then
    fail_or_skip "cargo kani is not installed"
fi

log="$(mktemp)"
trap 'rm -f "$log"' EXIT

echo "Kani checks: using Rust toolchain $kani_toolchain"

harnesses="
avatar_spec_new_preserves_supported_dimension_contract
render_resource_budget_uses_saturating_memory_math
resource_budget_memory_division_never_divides_by_zero
rect_of_size_and_edges_remain_non_zero_and_saturating
rect_intersection_when_present_is_inside_both_inputs
"

status=0
for harness in $harnesses; do
    echo "Kani checks: harness $harness" >>"$log"
    if cargo_kani --harness "$harness" --no-default-features >>"$log" 2>&1; then
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
