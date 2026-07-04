# Kani Verification Policy

`hashavatar` keeps a small set of Kani proof harnesses in the crate for pure
logic that is practical to model check. The harnesses are release evidence for
bounded spec, resource-budget, and geometry arithmetic invariants. They are not
a whole-crate formal verification claim and do not prove image-codec internals.

## Current Status

- Active release toolchain: Rust `1.96.1`.
- Kani verifier toolchain: Rust `1.90.0`.
- Locally targeted Kani: `cargo-kani 0.67.0`.
- Current result: `scripts/check_kani.sh` verifies 4 bounded
  no-default-features harnesses with 0 failures.

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
HASHAVATAR_KANI_TOOLCHAIN=1.96.1-x86_64-unknown-linux-gnu scripts/check_kani.sh
```

If the installed Kani compiler is compatible, `scripts/check_kani.sh` runs the
admitted harness list explicitly:

```sh
cargo kani --harness <name> --no-default-features
```

The explicit list keeps the release gate practical for this image-generation
crate and avoids depending on full-crate harness discovery behavior.

If Kani is not installed, the Rust `1.90.0` verifier toolchain is not present,
or the installed Kani compiler is incompatible with this crate, the script
prints an explicit skip and exits successfully. The stable release gate treats
that as a policy skip, not as completed formal verification.

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

For each future release, the project must choose one of these outcomes:

- run Kani proofs with a compatible Kani release
- pin a documented compatible Kani workflow
- document a verifier exception and the replacement evidence required before
  release

Replacement evidence may include deterministic tests, golden fingerprints,
fuzz-corpus evidence, panic-policy validation, dependency review, generated SVG
parser coverage, and release metadata checks, but it must be named explicitly.
A Kani skip is not the same as a proof.

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
