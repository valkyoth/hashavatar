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

cargo package --manifest-path core/Cargo.toml --locked --allow-dirty --list >/tmp/hashavatar-core-package-files-a.txt
cargo package --manifest-path core/Cargo.toml --locked --allow-dirty >/tmp/hashavatar-core-package-a.txt
cargo package --manifest-path core/Cargo.toml --locked --allow-dirty --list >/tmp/hashavatar-core-package-files-b.txt
cargo package --manifest-path core/Cargo.toml --locked --allow-dirty >/tmp/hashavatar-core-package-b.txt

cargo package --locked --allow-dirty --no-verify --list >/tmp/hashavatar-package-files-a.txt
cargo package --locked --allow-dirty --no-verify --list >/tmp/hashavatar-package-files-b.txt
if cargo package --locked --allow-dirty --no-verify >/tmp/hashavatar-package-a.txt 2>/tmp/hashavatar-package-error-a.txt; then
    cargo package --locked --allow-dirty --no-verify >/tmp/hashavatar-package-b.txt
else
    cat /tmp/hashavatar-package-error-a.txt >&2
    if grep -q 'no matching package named `hashavatar-core` found' /tmp/hashavatar-package-error-a.txt; then
        echo "reproducible build check: hashavatar package archive is blocked until hashavatar-core exists in the crates.io index" >&2
    else
        exit 1
    fi
fi

cmp -s /tmp/hashavatar-core-package-files-a.txt /tmp/hashavatar-core-package-files-b.txt
cmp -s /tmp/hashavatar-package-files-a.txt /tmp/hashavatar-package-files-b.txt
sha256sum /tmp/hashavatar-core-package-files-a.txt
sha256sum /tmp/hashavatar-package-files-a.txt

echo "reproducible build check: ok"
