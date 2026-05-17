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

echo "checks: docs"
cargo doc --no-deps

echo "checks: dependency and license policy"
cargo deny check
cargo audit

echo "checks: fuzz harnesses"
scripts/check_fuzz.sh

echo "checks: ok"
