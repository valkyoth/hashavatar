#!/usr/bin/env sh
set -eu

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

echo "checks: formatting"
cargo fmt --all --check

echo "checks: metadata"
package_files="$(cargo package --allow-dirty --list)"
case "$package_files" in
    *"LICENSE-APACHE"* ) ;;
    *)
        echo "missing LICENSE-APACHE from package file list" >&2
        exit 1
        ;;
esac
case "$package_files" in
    *"LICENSE-MIT"* ) ;;
    *)
        echo "missing LICENSE-MIT from package file list" >&2
        exit 1
        ;;
esac
case "$package_files" in
    *"LICENSE-EUPL-1.2.txt"* | *"
LICENSE
"* )
        echo "old EUPL license files unexpectedly appear in package file list" >&2
        exit 1
        ;;
esac

echo "checks: build"
cargo check

echo "checks: lint"
cargo clippy --all-targets -- -D warnings

echo "checks: tests"
cargo test

echo "checks: cli smoke"
cargo run --quiet --bin hashavatar-cli -- \
    --id robot@hashavatar.app \
    --kind robot \
    --background transparent \
    --format svg \
    --output "$tmp_dir/robot.svg" >/dev/null
test -s "$tmp_dir/robot.svg"

cargo run --quiet --bin hashavatar-cli -- \
    --id cat@hashavatar.app \
    --kind cat \
    --background themed \
    --format png \
    --output "$tmp_dir/cat.png" >/dev/null
test -s "$tmp_dir/cat.png"

echo "checks: dependency and license policy"
cargo deny check
cargo audit

echo "checks: ok"
