# Dependency Policy

`hashavatar` keeps the published crate dependency graph focused on rendering:

- `image` for raster buffers and WebP encoding
- `palette` for color conversion
- `rand` for deterministic seeded variation
- optional `serde` for string serialization/deserialization of public style
  enums when the `serde` feature is enabled
- `sha2` for identity hashing, with its `zeroize` feature enabled so hasher
  block buffers participate in upstream `ZeroizeOnDrop` cleanup
- `subtle` for constant-time identity digest comparison
- optional `blake3` for BLAKE3 identity hashing when the `blake3` feature is
  enabled, with its `zeroize` feature enabled so hasher and XOF reader state
  can be explicitly cleared
- optional `xxhash-rust` for XXH3-128 identity distribution when the `xxh3` feature is enabled
- optional `image/png` encoder support when the `png` feature is enabled
- optional `image/jpeg` encoder support when the `jpeg` feature is enabled
- optional `image/gif` encoder support when the `gif` feature is enabled
- `zeroize` for clearing derived identity digests and temporary hash preimage buffers

Dev-only test dependencies:

- `roxmltree` for parser-backed SVG well-formedness tests and fuzz harness
  validation
- `serde_json` for feature-gated serde round-trip tests

`sha2` remains the default identity dependency, and WebP remains the default
raster encoder. `blake3`, `xxhash-rust`, `serde`, `image/png`, `image/jpeg`,
and `image/gif` are explicit opt-in features so default users keep the smaller
conservative dependency graph and only compile extra support they use. The
`blake3` and `xxh3` features are mutually exclusive because identity hashing is
a crate-wide mode, not a runtime selection.

The crate must not depend on web frameworks, async runtimes, network clients, or service infrastructure. Those concerns belong in `hashavatar-api`.

Dependency changes should be reviewed for:

- whether the latest stable crate version is being used
- security advisory history
- default features
- transitive dependency growth
- license compatibility with `MIT OR Apache-2.0`
- whether the dependency is needed by the reusable crate or only by an application

## Freshness Policy

- Prefer the latest compatible stable release of each direct dependency.
- Before adding or changing a dependency, check current upstream information
  from crates.io, docs.rs, the crate repository, and RustSec advisories. Use
  web search when needed to confirm the crate is still maintained and that the
  chosen API reflects current guidance.
- Do not pin an older crate version unless there is a documented reason, such
  as a security concern, MSRV constraint, regression, license issue, or
  unacceptable transitive dependency growth.
- Re-check dependency freshness before stable releases with `cargo update`,
  `cargo audit`, `cargo deny check`, and `cargo outdated` when available.
- New optional dependencies must be justified in README/docs and covered by
  tests for their enabled feature path.

## Optional Hash Dependencies

- `blake3` is admitted for callers that want BLAKE3 identity derivation and
  dependency-provided SIMD support where the crate and platform provide it.
- `xxhash-rust` is admitted only for non-cryptographic XXH3-128 identity
  distribution. Do not present XXH3-128 as an adversarial collision-resistant
  identity hash, and do not recommend it for user-controlled identifiers unless
  the application first maps those identifiers through its own cryptographic
  boundary.
- Optional dependency features must be tested in valid feature combinations
  before release. Do not use `cargo test --all-features` because the `blake3`
  and `xxh3` identity-hash modes are intentionally mutually exclusive.

`scripts/validate-dependencies.sh` enforces the current direct dependency allowlist.

## Crate Boundary

`hashavatar` is intentionally a single image-generation crate. Raster buffers,
SVG rendering, encoders, deterministic identity hashing, and public avatar
options are kept together so the published API stays focused on producing
avatars.

Lower-level planning helpers should remain internal unless a future
image-generation use case justifies exposing them. A separate non-rendering
core crate is not part of the current roadmap.
