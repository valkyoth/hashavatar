#!/usr/bin/env sh
set -eu

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

echo "checks: formatting"
cargo fmt --all --check

echo "checks: release metadata"
scripts/validate-release-metadata.sh

echo "checks: documentation links"
scripts/check_doc_links.sh

echo "checks: workspace release policy"
scripts/release_crates.py --check
python3 scripts/test_release_crates.py

echo "checks: minimal dependency graph"
scripts/validate-dependencies.sh

echo "checks: format feature isolation"
scripts/check_format_features.sh

echo "checks: source size"
scripts/validate-source-size.sh

echo "checks: unsafe boundary"
scripts/validate-unsafe-boundary.sh

echo "checks: panic policy"
scripts/validate-panic-policy.sh

echo "checks: build"
cargo check --workspace --all-targets

echo "checks: lint"
cargo clippy --workspace --all-targets --all-features -- -D warnings

echo "checks: tests"
cargo test --workspace
cargo test --workspace --all-features --release

echo "checks: MSRV compatibility"
if command -v rustup >/dev/null 2>&1; then
    if ! rustup toolchain list | grep -q '^1\.90\.0'; then
        rustup toolchain install 1.90.0 --profile minimal
    fi
    cargo +1.90.0 check --workspace --all-targets --all-features
    cargo +1.90.0 check --workspace --all-targets --no-default-features
else
    echo "checks: rustup is required for MSRV compatibility checks" >&2
    exit 1
fi

echo "checks: docs"
cargo doc --workspace --all-features --no-deps

echo "checks: dependency and license policy"
cargo deny check
cargo audit

echo "checks: fuzz harnesses"
scripts/check_fuzz.sh

echo "checks: ok"
