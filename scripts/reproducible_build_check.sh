#!/usr/bin/env sh
set -eu

work_root="$(mktemp -d)"
trap 'rm -rf "$work_root"' EXIT

first_target="$work_root/reproducible-a"
second_target="$work_root/reproducible-b"

if command -v git >/dev/null 2>&1 && git rev-parse --git-dir >/dev/null 2>&1; then
    SOURCE_DATE_EPOCH="${SOURCE_DATE_EPOCH:-$(git log -1 --format=%ct)}"
else
    SOURCE_DATE_EPOCH="${SOURCE_DATE_EPOCH:-0}"
fi
export SOURCE_DATE_EPOCH

package_name="hashavatar-core"
version="0.1.0-alpha.2"
CARGO_TARGET_DIR="$first_target" cargo package -p "$package_name" --locked --allow-dirty --no-verify
CARGO_TARGET_DIR="$second_target" cargo package -p "$package_name" --locked --allow-dirty --no-verify

first_crate="$first_target/package/$package_name-$version.crate"
second_crate="$second_target/package/$package_name-$version.crate"
test -s "$first_crate"
test -s "$second_crate"
cmp "$first_crate" "$second_crate"
sha256sum "$first_crate" "$second_crate"

# The facade's source-only core dependency is intentionally absent from
# crates.io during prereleases, so Cargo cannot assemble its registry archive.
# File-list evidence plus the compiled workspace covers it until stable publish.
cargo package -p hashavatar --locked --allow-dirty --list >/dev/null
echo "reproducible package check: facade file list validated; registry archive deferred"

echo "reproducible package check: byte-identical archives"
