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

The stable gate runs the normal project checks, package verification, documentation generation, fuzz harness compilation, and reproducibility checks. Optional SBOM generation runs when `cargo-sbom` is installed.

The crate package should contain the reusable library, metadata, documentation, and policy scripts. It should not contain binaries, the demo/API server, fuzz harnesses, or generated build output.
