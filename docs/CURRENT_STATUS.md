# Current Status

## Supported Release

The latest crates.io release is `hashavatar 1.3.0`.

The signed `v1.3.0` tag and `release/1.3` branch preserve the final 1.x
renderer and migration API. The maintenance branch accepts serious security
and correctness fixes only. New features and intentional visual changes belong
to 2.0.

## Development Line

`main` is preparing `2.0.0-alpha.1`, the first workspace and canonical-scene
vertical slice described in [PLAN_TOWARDS_2.0.md](PLAN_TOWARDS_2.0.md).

The 2.0 prerelease policy is deliberate:

- alpha, beta, and release-candidate tags are pushed to GitHub;
- prerelease crates are not published to crates.io;
- `hashavatar-website` follows each reviewed tag through local path or checkout
  dependencies;
- every tag receives local release checks, GitHub CI and CodeQL, downstream
  integration testing, and a permanent pentest summary;
- crates.io publication resumes only for the approved stable `2.0.0` packages.

This keeps unstable APIs and pixels out of the public registry while the real
website continuously exercises the exact tagged source.

## Toolchain

| Item | Version or policy |
| --- | --- |
| MSRV | Rust `1.90.0` |
| Development toolchain | Rust `1.97.1` |
| CodeQL | GitHub default setup |
| Unsafe Rust | Forbidden in first-party library code |
| Stable identity mode | SHA-512 |
| Stable default raster format | WebP |

The repository checks supported feature combinations on the MSRV and uses the
pinned development toolchain for full local and release gates.

## Release Evidence

The `v1.3.0` release completed:

- all valid SHA-512, BLAKE3, XXH3, and format feature matrices;
- formatting, strict Clippy, rustdoc, and integration tests;
- Rust `1.90.0` compatibility checks;
- dependency, license, RustSec, unsafe, and panic-policy checks;
- fuzz-harness compilation and five bounded Kani proofs;
- byte-identical package archive checks and SBOM generation;
- `cargo-semver-checks` against `v1.2.0`;
- crates.io publish dry-run;
- independent pentesting, green GitHub validation, and successful
  `hashavatar-website` integration testing.

Version-specific details live in [release notes](../release-notes/) and the
[changelog](../CHANGELOG.md). Starting with 2.0 prereleases, permanent pentest
summaries live under [`security/pentest`](../security/pentest/README.md).
