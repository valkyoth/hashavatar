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
        echo "usage: scripts/generate-sbom.sh [--required]" >&2
        exit 2
        ;;
esac

fail_or_skip() {
    message="$1"
    if [ "$required" = true ]; then
        echo "generate sbom: required; $message" >&2
        exit 1
    fi
    echo "generate sbom: skipping; $message" >&2
    exit 0
}

output_dir="${HASHAVATAR_SBOM_DIR:-target/release-evidence}"
mkdir -p "$output_dir"

if ! cargo sbom --version >/dev/null 2>&1; then
    fail_or_skip "install with: cargo install --locked cargo-sbom --version 0.10.0"
fi

spdx_output="$output_dir/hashavatar.spdx.json"
cyclonedx_output="$output_dir/hashavatar.cyclonedx.json"
manifest="$output_dir/sbom-MANIFEST.txt"

cargo sbom --output-format spdx_json_2_3 > "$spdx_output"
cargo sbom --output-format cyclone_dx_json_1_4 > "$cyclonedx_output"

test -s "$spdx_output"
test -s "$cyclonedx_output"
grep -q '"spdxVersion"[[:space:]]*:[[:space:]]*"SPDX-2.3"' "$spdx_output"
grep -q '"bomFormat"[[:space:]]*:[[:space:]]*"CycloneDX"' "$cyclonedx_output"

{
    echo "hashavatar SBOM evidence"
    echo
    rustc -Vv
    cargo -V
    cargo sbom --version
    echo
    sha256sum "$spdx_output" "$cyclonedx_output"
} > "$manifest"

cat "$manifest"
