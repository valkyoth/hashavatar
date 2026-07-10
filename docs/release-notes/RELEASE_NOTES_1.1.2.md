# hashavatar 1.1.2

`1.1.2` is a maintenance release focused on dependency, Rust toolchain, and CI
pin freshness. It does not intentionally change the public API, avatar
rendering behavior, or visual fingerprints.

## Dependencies

- Updated `sanitization` from `1.2.2` to `1.2.4`.
- Updated `sanitization-crypto-interop` from `1.2.2` to `1.2.4`.
- Refreshed compatible transitive dependencies in the root and fuzz
  lockfiles.
- Confirmed all other direct runtime, optional, development, and fuzz
  dependencies remain current.

Both sanitization crates continue to declare Rust `1.90` support.

## Rust Support

- Updated the pinned development and release toolchain from Rust `1.96.1` to
  Rust `1.97.0`.
- Kept `Cargo.toml` `rust-version = "1.90"` as the public MSRV.
- Retained focused CI compatibility checks on Rust `1.90.0`.
- Retained the Kani verifier default on its documented Rust `1.90.0`
  toolchain, independently of the development compiler.

## CI And Tooling

- Updated the immutable `taiki-e/install-action` pin from the `v1.1.1` tag's
  `v2.82.8` to `v2.83.0` (including the intervening post-release `v2.82.9`
  maintenance update).
- Confirmed `actions/checkout` remains current at `v7.0.0`.
- Confirmed `Swatinem/rust-cache` remains current at `v2.9.1`.
- Confirmed the repository's audit, deny, fuzz, outdated, and Kani tooling is
  current at release preparation time.
- Updated the fuzz dependency-policy check to use the current `cargo-deny`
  global-option ordering.

## Compatibility

- No intentional public API changes.
- No intentional avatar visual fingerprint changes.
- The default SHA-512 mode and optional BLAKE3/XXH3 feature model are
  unchanged.
- WebP remains the default raster encoder; PNG, JPEG, and GIF remain opt-in.

## Verification

- Passed all-format compatibility checks on Rust `1.90.0`, `1.91.0`,
  `1.92.0`, `1.93.0`, `1.94.0`, `1.95.0`, `1.96.0`, `1.96.1`, and `1.97.0`.
- Passed the stable release gate: formatting, metadata, dependency boundary,
  unsafe and panic policies, clippy, unit tests, doctests, MSRV feature checks,
  documentation, cargo-deny, RustSec audit, fuzz harness compilation, four
  bounded Kani proofs, reproducible packaging, SBOM generation, and crates.io
  publish dry run.
- Confirmed both the root and fuzz direct dependency sets are current with
  `cargo outdated`.
