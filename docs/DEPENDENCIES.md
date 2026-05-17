# Dependency Policy

`hashavatar` keeps the published crate dependency graph focused on rendering:

- `image` for raster buffers and encoders
- `palette` for color conversion
- `rand` for deterministic seeded variation
- `sha2` for identity hashing

The crate must not depend on web frameworks, async runtimes, network clients, or service infrastructure. Those concerns belong in `hashavatar-api`.

Dependency changes should be reviewed for:

- security advisory history
- default features
- transitive dependency growth
- license compatibility with `MIT OR Apache-2.0`
- whether the dependency is needed by the reusable crate or only by an application

`scripts/validate-dependencies.sh` enforces the current direct dependency allowlist.
