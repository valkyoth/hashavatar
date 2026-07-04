#!/usr/bin/env sh
set -eu

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

echo "checks: formatting"
cargo fmt --all --check

echo "checks: release metadata"
scripts/validate-release-metadata.sh

echo "checks: minimal dependency graph"
scripts/validate-dependencies.sh

echo "checks: unsafe boundary"
scripts/validate-unsafe-boundary.sh

echo "checks: panic policy"
scripts/validate-panic-policy.sh

echo "checks: build"
cargo check

echo "checks: lint"
cargo clippy --all-targets -- -D warnings

echo "checks: tests"
cargo test

echo "checks: MSRV compatibility"
if command -v rustup >/dev/null 2>&1; then
    if ! rustup toolchain list | grep -q '^1\.90\.0'; then
        rustup toolchain install 1.90.0 --profile minimal
    fi
    cargo +1.90.0 check
    cargo +1.90.0 check --features all-formats
    cargo +1.90.0 check --features "blake3 all-formats"
    cargo +1.90.0 check --features "xxh3 all-formats"
else
    echo "checks: rustup is required for MSRV compatibility checks" >&2
    exit 1
fi

echo "checks: docs"
cargo doc --no-deps

echo "checks: dependency and license policy"
cargo deny check
cargo audit

echo "checks: fuzz harnesses"
scripts/check_fuzz.sh

echo "checks: ok"
