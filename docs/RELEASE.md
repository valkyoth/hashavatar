# Release Process

## Distribution Policy

The `1.3.x` line is maintained from `release/1.3` for serious security and
correctness fixes. New development occurs on `main` toward 2.0.

Hashavatar 2.0 prereleases are commit milestones:

- finish each alpha, beta, or RC step with a clearly named implementation-stop
  commit;
- record the exact base and candidate commit SHAs for review;
- do not create prerelease Git tags or GitHub releases;
- do not publish prerelease packages to crates.io;
- point `hashavatar-website` at the exact reviewed commit;
- require website integration, GitHub CI, CodeQL, and pentest evidence before
  beginning the next milestone;
- publish and tag only when stable `2.0.0` is approved.

Commit SHAs are immutable review identifiers. A pentest normally reviews
`<previous-stop>..<candidate-stop>` and scans the complete tree at the candidate
SHA. Any remediation creates a new candidate SHA and must be retested.

`scripts/release_crates.py` supports prerelease validation through `--check`
and `--prepare-only`; publication refuses SemVer prerelease versions.

## Every Prerelease Milestone

1. Update Cargo versions, changelog, root release note, current status, and
   `release-crates.toml`.
2. Commit a clearly named implementation stop.
3. Report the previous stop and candidate stop SHAs for pentesting.
4. Resolve all temporary root `PENTEST.md` findings and delete the file.
5. Retest each remediation candidate until no blocking finding remains.
6. Add `security/pentest/v<VERSION>.md` with the exact reviewed commit and
   reviewed range.
7. Run local gates, GitHub CI, CodeQL, and `hashavatar-website` against that
   exact commit.
8. Record completion in the roadmap and begin the next milestone without
   creating a prerelease tag.

Prerelease preparation uses:

```bash
scripts/release_crates.py --check
scripts/release_crates.py --prepare-only
```

## Stable crates.io Release

Before publishing a stable release:

```bash
cargo update
cargo outdated
scripts/stable_release_gate.sh release
scripts/release_crates.py --check
scripts/release_crates.py --require-tag
```

The stable candidate requires a clean commit, permanent PASS pentest summary,
green downstream/GitHub evidence, and a signed annotated `v<VERSION>` tag at
HEAD. The release script publishes selected crates in dependency order and
pauses between dependency layers so crates.io can index them.

## Package Boundaries

Published packages contain reusable libraries, their own technical README,
licenses, relevant policy documents, and examples. They exclude the website,
fuzz targets, generated output, temporary pentest input, repository
administration files, and archived design drafts.
