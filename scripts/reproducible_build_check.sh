#!/usr/bin/env sh
set -eu

first_target="${HASHAVATAR_REPRO_TARGET_A:-target/reproducible-a}"
second_target="${HASHAVATAR_REPRO_TARGET_B:-target/reproducible-b}"

if command -v git >/dev/null 2>&1 && git rev-parse --git-dir >/dev/null 2>&1; then
    SOURCE_DATE_EPOCH="${SOURCE_DATE_EPOCH:-$(git log -1 --format=%ct)}"
else
    SOURCE_DATE_EPOCH="${SOURCE_DATE_EPOCH:-0}"
fi
export SOURCE_DATE_EPOCH

CARGO_TARGET_DIR="$first_target" cargo build --release --locked
CARGO_TARGET_DIR="$second_target" cargo build --release --locked

cargo package --locked --allow-dirty --list >/tmp/hashavatar-package-files-a.txt
cargo package --locked --allow-dirty >/tmp/hashavatar-package-a.txt
cargo package --locked --allow-dirty --list >/tmp/hashavatar-package-files-b.txt
cargo package --locked --allow-dirty >/tmp/hashavatar-package-b.txt

cmp -s /tmp/hashavatar-package-files-a.txt /tmp/hashavatar-package-files-b.txt
sha256sum /tmp/hashavatar-package-files-a.txt

echo "reproducible build check: ok"
