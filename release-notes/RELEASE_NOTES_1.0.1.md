# hashavatar 1.0.1

`1.0.1` is a maintenance and documentation release for `hashavatar`.

## Changed

- Refreshed compatible transitive dependencies in `Cargo.lock` and
  `fuzz/Cargo.lock`.
- Updated the GitHub CI `taiki-e/install-action` pin to `v2.79.14`.
- Lowered the manifest and toolchain MSRV to Rust `1.90.0` after validating the
  release gate and compatibility matrix.
- Added a polished README header with the project image, documentation links,
  and Rust version support table.
- Kept the README image in the repository while excluding `.github/images/**`
  from the published crate package.

## Security And Hardening

- Added a debug assertion for out-of-range internal identity digest byte access
  so future renderer mistakes fail in debug/test builds while release builds
  remain non-panicking.
- Changed `AvatarSpec::pixel_count()` and `AvatarSpec::rgba_buffer_len()` to
  use saturating multiplication as defense-in-depth for future dimension-limit
  changes.
- Clarified that the transient by-value seed argument passed to
  `StdRng::from_seed` is part of the documented zeroization residual.

## Compatibility Evidence

- Rust `1.90.0`: full release gate.
- Rust `1.91.0` through `1.96.0`: `cargo check --features all-formats`.
- Rust `1.90.0`: additional checks for `blake3 all-formats` and
  `xxh3 all-formats`.
