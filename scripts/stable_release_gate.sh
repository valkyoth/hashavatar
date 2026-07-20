#!/usr/bin/env sh
set -eu

mode="${1:-check}"

case "$mode" in
    check | release)
        ;;
    *)
        echo "usage: scripts/stable_release_gate.sh [check|release]" >&2
        exit 2
        ;;
esac

cargo_version="$(
    sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | sed -n '1p'
)"

if [ "$mode" = "release" ]; then
    case "$cargo_version" in
        *-*)
            echo "stable release gate: release mode requires a stable Cargo.toml version, got $cargo_version" >&2
            exit 1
            ;;
    esac
fi

echo "stable release gate: standard checks"
scripts/checks.sh

echo "stable release gate: docs"
cargo doc --no-deps

echo "stable release gate: fuzz harnesses"
scripts/check_fuzz.sh

echo "stable release gate: Kani proofs"
if [ "$mode" = "release" ]; then
    scripts/check_kani.sh --required
else
    scripts/check_kani.sh
fi

echo "stable release gate: reproducible .crate package archive"
scripts/reproducible_build_check.sh

echo "stable release gate: SBOM"
if [ "$mode" = "release" ]; then
    scripts/generate-sbom.sh --required
else
    scripts/generate-sbom.sh
fi

echo "stable release gate: publish dry run"
cargo publish --dry-run --allow-dirty

echo "stable release gate: ok ($mode)"
