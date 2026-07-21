# hashavatar 1.1.1

`1.1.1` is a maintenance release for `hashavatar` focused on dependency and CI
pin freshness. It does not intentionally change avatar rendering behavior or
the public API.

## Dependency Updates

- Updated `rand` from `0.10.1` to `0.10.2`.
- Updated optional `xxhash-rust` from `0.8.15` to `0.8.16`.
- Refreshed compatible transitive dependencies in the root lockfile:
  - `arrayvec` `0.7.7` to `0.7.8`
  - `chacha20` `0.10.0` to `0.10.1`
  - `hybrid-array` `0.4.12` to `0.4.13`
- Refreshed compatible transitive dependencies in the fuzz lockfile:
  - `chacha20` `0.10.0` to `0.10.1`
  - `hybrid-array` `0.4.12` to `0.4.13`

## CI

- Updated pinned `taiki-e/install-action` from `v2.82.7` to `v2.82.8`.
- Confirmed `actions/checkout` remains current at `v7.0.0`.
- Confirmed `Swatinem/rust-cache` remains current at `v2.9.1`.

## Documentation

- Updated README installation snippets and release metadata for `1.1.1`.
- Updated latest-stable Rust wording to Rust `1.96.1`.
- Added local `cargo check --features all-formats` compatibility evidence for
  Rust `1.96.1`.
- Documented the split between the Rust `1.96.1` development toolchain and the
  Rust `1.90.0` MSRV.

## Toolchain

- Switched `rust-toolchain.toml` from Rust `1.90.0` to Rust `1.96.1`.
- Kept `Cargo.toml` `rust-version = "1.90"` as the public MSRV.
- Added project checks that run focused compatibility coverage on Rust `1.90.0`
  in addition to the normal development-toolchain checks.

## Verification

- Added bounded Kani proof harnesses for pure invariants that matter to this
  crate: avatar spec bounds, render-resource memory math, and rectangle
  arithmetic.
- Added `scripts/check_kani.sh`, pinned by default to the Rust
  `1.90.0-x86_64-unknown-linux-gnu` verifier toolchain when Kani is installed.
- Added `docs/KANI.md` documenting that these harnesses are scoped bounded
  evidence, not a whole-crate formal-verification claim.
- Added the Kani check to the stable release gate. Missing or incompatible Kani
  is reported as an explicit verifier skip, not as a completed proof.

## Compatibility

- No intentional public API changes.
- No intentional avatar visual fingerprint changes.
