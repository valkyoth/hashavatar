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

## Compatibility

- No intentional public API changes.
- No intentional avatar visual fingerprint changes.
