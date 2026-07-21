# Release Process

## Distribution Policy

The `1.3.x` line is maintained from `release/1.3` for serious security and
correctness fixes. New development occurs on `main` toward 2.0.

Hashavatar 2.0 prereleases are source releases:

- create and push signed alpha, beta, and release-candidate Git tags;
- do not publish prerelease packages to crates.io;
- point `hashavatar-website` at the reviewed local checkout/tag;
- require website integration, GitHub CI, CodeQL, and pentest evidence for each
  tag;
- publish the workspace crates only when stable `2.0.0` is approved.

`scripts/release_crates.py` enforces this distinction. Its `--check` and
`--prepare-only` modes support prerelease validation, while publication refuses
any version containing a SemVer prerelease suffix.

## Every Tag

Before tagging any release:

1. Update Cargo versions, changelog, root `release-notes/`, current status, and
   `release-crates.toml`.
2. Confirm all root `PENTEST.md` scratch input is resolved and deleted.
3. Add `security/pentest/v<VERSION>.md` with a PASS disposition.
4. Run the version's complete local gate and package checks.
5. Test the exact candidate with `hashavatar-website`.
6. Commit only final evidence changes and wait for GitHub CI and CodeQL.
7. Create a signed annotated `v<VERSION>` tag and push it.

Prerelease preparation uses:

```bash
scripts/release_crates.py --check
scripts/release_crates.py --prepare-only
```

The preparation command validates packages but never uploads them.

## Stable crates.io Release

Before publishing a stable release:

```bash
cargo update
cargo outdated
scripts/stable_release_gate.sh release
scripts/release_crates.py --check
scripts/release_crates.py --require-tag
```

`cargo outdated` is optional when unavailable, but dependency freshness must
still be checked against crates.io, docs.rs, upstream repositories, and RustSec.

The stable gate runs on the pinned development toolchain and checks the
`Cargo.toml` MSRV, currently Rust `1.90.0`. Release mode requires the documented
Kani and SBOM tool versions, compares independently generated `.crate` archives
byte for byte, verifies package contents, and performs publish dry-runs.

The release script validates workspace versions and explicit dependency order,
requires a clean tagged commit and permanent pentest summary, reruns release
checks, publishes only entries enabled in `release-crates.toml`, and pauses
between dependency layers so crates.io can index newly published packages.

## Package Boundaries

Published packages contain reusable libraries, their own technical README,
licenses, relevant policy/evidence documents, and examples. They do not contain
the website, fuzz targets, generated build output, temporary pentest input, or
unreviewed binary tools.
