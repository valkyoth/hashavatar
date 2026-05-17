#!/usr/bin/env sh
set -eu

if [ ! -d fuzz ]; then
    echo "fuzz checks: skipping; fuzz/ is not present"
    exit 0
fi

echo "fuzz checks: compile harnesses"
cargo check --manifest-path fuzz/Cargo.toml --bins

if cargo audit --version >/dev/null 2>&1 && [ -s fuzz/Cargo.lock ]; then
    echo "fuzz checks: RustSec advisories"
    cargo audit --no-fetch --stale --file fuzz/Cargo.lock
else
    echo "fuzz checks: skipping RustSec advisories; cargo-audit or fuzz/Cargo.lock unavailable"
fi

if cargo deny --version >/dev/null 2>&1 && [ -s fuzz/deny.toml ]; then
    echo "fuzz checks: dependency policy"
    cargo deny --manifest-path fuzz/Cargo.toml check licenses --config fuzz/deny.toml
else
    echo "fuzz checks: skipping dependency policy; cargo-deny or fuzz/deny.toml unavailable"
fi

echo "fuzz checks: ok"
