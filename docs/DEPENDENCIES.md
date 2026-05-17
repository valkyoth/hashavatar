# Dependency Policy

`hashavatar` keeps the published crate dependency graph focused on rendering:

- `image` for raster buffers and encoders
- `palette` for color conversion
- `rand` for deterministic seeded variation
- `sha2` for identity hashing

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

`scripts/validate-dependencies.sh` enforces the current direct dependency allowlist.
