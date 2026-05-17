# Dependency Policy

`hashavatar` keeps the published crate dependency graph focused on rendering:

- `image` for raster buffers and encoders
- `palette` for color conversion
- `rand` for deterministic seeded variation
- `sha2` for identity hashing
- `subtle` for constant-time identity digest comparison
- optional `blake3` for BLAKE3 identity hashing when the `blake3` feature is enabled
- optional `xxhash-rust` for XXH3-128 identity distribution when the `xxh3` feature is enabled
- `zeroize` for clearing derived identity digests and temporary hash preimage buffers

`sha2` remains the default identity dependency. `blake3` and `xxhash-rust` are
explicit opt-in dependencies so default users keep the smaller conservative
dependency graph.

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
- Optional hash dependency features must be tested with `cargo test
  --all-features` before release.

`scripts/validate-dependencies.sh` enforces the current direct dependency allowlist.
