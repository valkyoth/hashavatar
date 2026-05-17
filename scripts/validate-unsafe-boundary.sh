#!/usr/bin/env sh
set -eu

if ! grep -q '^#!\[forbid(unsafe_code)\]$' src/lib.rs; then
    echo "unsafe boundary: src/lib.rs must keep #![forbid(unsafe_code)]" >&2
    exit 1
fi

if grep -RIn --include '*.rs' \
    -e 'unsafe[[:space:]]' \
    -e '#\[allow(unsafe_code)\]' \
    -e '#!\[allow(unsafe_code)\]' \
    src
then
    echo "unsafe boundary: unsafe code or unsafe-code allowance found in src/" >&2
    exit 1
fi

echo "unsafe boundary: ok"
