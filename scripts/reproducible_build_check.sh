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
version="0.1.0-alpha.5"
CARGO_TARGET_DIR="$first_target" cargo package -p "$package_name" --locked --allow-dirty --no-verify
CARGO_TARGET_DIR="$second_target" cargo package -p "$package_name" --locked --allow-dirty --no-verify

first_crate="$first_target/package/$package_name-$version.crate"
second_crate="$second_target/package/$package_name-$version.crate"
test -s "$first_crate"
test -s "$second_crate"
cmp "$first_crate" "$second_crate"
sha256sum "$first_crate" "$second_crate"

# Source-only companion dependencies are intentionally absent from crates.io,
# so Cargo cannot assemble their registry archives during prereleases. File-list
# evidence plus compiled workspace/package checks cover them until stable publish.
for package_name in hashavatar-formats hashavatar; do
    first_list="$work_root/$package_name-a.list"
    second_list="$work_root/$package_name-b.list"
    cargo package -p "$package_name" --locked --allow-dirty --list >"$first_list"
    cargo package -p "$package_name" --locked --allow-dirty --list >"$second_list"
    cmp "$first_list" "$second_list"
done

echo "reproducible package check: core archive is byte-identical"
echo "reproducible package check: formats/facade file lists validated; archives deferred"
