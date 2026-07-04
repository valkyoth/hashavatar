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
bounded Kani proof execution run as part of the same gate. Kani uses the
documented Rust `1.90.0` verifier toolchain when installed; an unavailable or
incompatible verifier is printed as an explicit skip, not treated as completed
formal verification. Optional SBOM generation runs when `cargo-sbom` is
installed.

The crate package should contain the reusable library, metadata, documentation,
and policy scripts. It should not contain binaries, the demo/API server, fuzz
harnesses, or generated build output.
