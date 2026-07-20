#!/usr/bin/env sh
set -eu

work_root="$(mktemp -d)"
trap 'rm -rf "$work_root"' EXIT

first_target="$work_root/reproducible-a"
second_target="$work_root/reproducible-b"
package_name="$(sed -n 's/^name = "\([^"]*\)"/\1/p' Cargo.toml | sed -n '1p')"
version="$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | sed -n '1p')"

if command -v git >/dev/null 2>&1 && git rev-parse --git-dir >/dev/null 2>&1; then
    SOURCE_DATE_EPOCH="${SOURCE_DATE_EPOCH:-$(git log -1 --format=%ct)}"
else
    SOURCE_DATE_EPOCH="${SOURCE_DATE_EPOCH:-0}"
fi
export SOURCE_DATE_EPOCH

CARGO_TARGET_DIR="$first_target" cargo package --locked --allow-dirty
CARGO_TARGET_DIR="$second_target" cargo package --locked --allow-dirty

first_crate="$first_target/package/$package_name-$version.crate"
second_crate="$second_target/package/$package_name-$version.crate"

test -s "$first_crate"
test -s "$second_crate"
cmp "$first_crate" "$second_crate"
sha256sum "$first_crate" "$second_crate"

echo "reproducible package check: byte-identical archives"
