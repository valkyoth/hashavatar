#!/usr/bin/env sh
set -eu

if [ ! -s src/kani_proofs.rs ]; then
    echo "Kani checks: skipping; src/kani_proofs.rs is not present"
    exit 0
fi

kani_toolchain="${HASHAVATAR_KANI_TOOLCHAIN:-1.90.0-x86_64-unknown-linux-gnu}"

if ! rustup toolchain list | grep -q "^$kani_toolchain"; then
    echo "Kani checks: skipping; Rust toolchain $kani_toolchain is not installed"
    exit 0
fi

cargo_kani() {
    rustup run "$kani_toolchain" cargo kani "$@"
}

if ! cargo_kani --version >/dev/null 2>&1; then
    echo "Kani checks: skipping; cargo kani is not installed"
    exit 0
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
    echo "Kani checks: skipping; installed Kani compiler is incompatible with this crate's Rust version"
    exit 0
fi

cat "$log"
exit "$status"
