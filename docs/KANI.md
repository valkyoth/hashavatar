# Kani Verification Policy

`hashavatar` keeps a small set of Kani proof harnesses in the crate for pure
logic that is practical to model check. The harnesses are release evidence for
bounded spec, resource-budget, and geometry arithmetic invariants. They are not
a whole-crate formal verification claim and do not prove image-codec internals.

## Current Status

- Active release toolchain: Rust `1.97.1`.
- Kani verifier toolchain: Rust `1.90.0`.
- Locally targeted Kani: `cargo-kani 0.67.0`.
- Current admitted harness count: `5`.
- Current result: `scripts/check_kani.sh` verifies all five bounded
  no-default-features harnesses with 0 failures when the documented verifier is
  available.

Kani runs are compiler-integration-sensitive because Kani is a verifier with
its own compiler integration. Updating the normal development toolchain does
not automatically make every Kani release understand that compiler, so verifier
evidence records the exact Rust pairing separately from normal Cargo release
checks.

## How To Check

Run:

```sh
cargo kani --version
scripts/check_kani.sh
```

By default, `scripts/check_kani.sh` runs through the documented
`1.90.0-x86_64-unknown-linux-gnu` toolchain. Override this only for verifier
experiments:

```sh
HASHAVATAR_KANI_TOOLCHAIN=1.97.1-x86_64-unknown-linux-gnu scripts/check_kani.sh
```

If the installed Kani compiler is compatible, `scripts/check_kani.sh` runs the
admitted harness list explicitly:

```sh
cargo kani --harness <name> --no-default-features
```

The explicit list keeps the release gate practical for this image-generation
crate and avoids depending on full-crate harness discovery behavior.

Without arguments, missing or incompatible Kani tooling produces an explicit
successful skip so contributors can run the remaining local checks. Stable
release mode invokes `scripts/check_kani.sh --required`; anything other than
exactly `cargo-kani 0.67.0`, a missing Rust `1.90.0` verifier toolchain, or
compiler incompatibility then fails closed and blocks the release.

## Harness Scope

Current harnesses cover:

- `AvatarSpec::new` dimension validation and supported-buffer upper bounds
- `AvatarRenderResourceBudget` saturating concurrency memory estimates
- memory-budget division behavior without divide-by-zero paths
- rectangle construction, saturating edge arithmetic, and intersection bounds

The harnesses intentionally avoid:

- full avatar raster rendering
- WebP, PNG, JPEG, and GIF encoder internals
- SVG string assembly
- cryptographic hash implementations
- `rand::StdRng` internals
- gradient interpolation arithmetic, which Kani `0.67.0` verifies slowly due to
  many generated arithmetic side conditions and is covered by normal tests

Those areas remain covered by normal tests, golden fingerprints, fuzz harness
compilation, dependency policy, panic-policy checks, and release-package
validation. Kani is an additional bounded check for selected pure logic, not a
replacement for the rest of the gate.

The exact public option table and byte-to-variant mapping behavior remains
covered by ordinary tests. Kani `0.67.0` currently hits a compiler panic on the
associated-slice indexing and symbolic modulo forms used for those tables, so
the Kani harness avoids overclaiming table-selection proof coverage.

## Release Policy

Every stable release must run all admitted harnesses with the documented Kani
version and verifier toolchain. A missing verifier, incompatible compiler, or
failed harness blocks `scripts/stable_release_gate.sh release`. Updating the
Kani/toolchain pairing requires updating this document and passing the complete
admitted harness set before release.

## Upgrade Guidance

When a newer Kani release is available:

```sh
cargo install --locked kani-verifier
cargo kani setup
cargo kani --version
scripts/check_kani.sh
```

Revisit this document whenever the installed Kani release, documented Kani
toolchain, active release toolchain, MSRV, or harness scope changes.
