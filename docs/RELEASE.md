# Release Checks

Before tagging or publishing a stable release:

```bash
scripts/stable_release_gate.sh release
cargo publish --dry-run
```

The stable gate runs the normal project checks, package verification, documentation generation, fuzz harness compilation, and reproducibility checks. Optional SBOM generation runs when `cargo-sbom` is installed.

The crate package should contain the reusable library, metadata, documentation, and policy scripts. It should not contain binaries, the demo/API server, fuzz harnesses, or generated build output.
