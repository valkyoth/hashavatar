# Release Checks

Before tagging or publishing a stable release:

```bash
cargo update
cargo outdated
scripts/stable_release_gate.sh release
cargo publish --dry-run
```

`cargo outdated` is optional when the tool is not installed, but dependency
freshness still has to be checked before release through crates.io, docs.rs,
upstream repositories, and RustSec advisories.

The stable gate runs on the pinned development toolchain from
`rust-toolchain.toml`. The normal project checks also install and run focused
compatibility checks against the `Cargo.toml` MSRV, currently Rust `1.90.0`.
Package verification, documentation generation, fuzz harness compilation, and
bounded Kani proof execution run as part of the same gate. In `check` mode, an
unavailable or incompatible Kani verifier and missing `cargo-sbom` are reported
as explicit skips. In `release` mode, `cargo-kani 0.67.0`, its pinned Kani Rust
`1.90.0` toolchain, and `cargo-sbom 0.10.0` are mandatory; a missing or
mismatched tool fails closed. The reproducibility check creates two
non-overrideable target directories under a fresh temporary root and compares
the actual `.crate` archives byte for byte.

The crate package should contain the reusable library, metadata, documentation,
and policy scripts. It should not contain binaries, the demo/API server, fuzz
harnesses, or generated build output.
