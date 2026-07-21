# Current Status

## Published Line

The latest crates.io release is `hashavatar 1.3.0`. The signed `v1.3.0` tag and
`release/1.3` branch preserve the final 1.x renderer. That branch accepts
serious security and correctness fixes; new features and intentional visual
changes belong to 2.0.

## Development Line

`main` implements `2.0.0-alpha.5`. This source-only prerelease assembles the
portable core, established-format provider, and recommended facade workflow:

- stateless, label-separated SHA-512 trait derivation;
- private checked Q16.16 geometry;
- a bounded validated private scene with primitives, paths, clips, opacity,
  gradients, and exact integer compositing;
- one safe-Rust CPU straight-alpha RGBA8 executor;
- owned and caller-provided padded surfaces through the same executor;
- versioned pixel digests over visible rows;
- deterministic SVG documents and fragments from the same scene;
- 31 family compilers, 13 backgrounds, and five frame shapes;
- family-aware default palettes and recognizable 1.3-derived subject geometry;
- immutable family capability declarations and frozen catalog IDs;
- fixed-capacity typed accessory stacks with six admitted semantic slots;
- nine accessory variants, eight expressions, and six integer palettes;
- calibrated integer anchors and transforms for every face-capable family;
- strict typed compatibility/collision errors and opt-in frozen automatic
  fallback;
- immutable resolved styles and complete accepted/adjusted/substituted/rejected
  layout reports;
- transparent surface clearing and rectangle, ellipse, and path clips shared
  by raster and SVG;
- owned redacted identities and a transactional request builder;
- typed identity/avatar asset keys plus catalog and render-contract IDs;
- conservative `ResourceBudget` and sanitized reusable RGBA storage;
- isolated `hashavatar-formats` writer and owned-output APIs;
- default lossless WebP plus explicit PNG, JPEG, GIF, and all-format features;
- typed format metadata, alpha capabilities, semantic encoded keys, and
  caller-bound build keys;
- an exhaustive 2,015 family/background/frame execution matrix;
- ordered per-family and aggregate debug/release pixel KATs plus parser-backed
  SVG checks;
- focused catalog, layout, surface, SVG-writer, and format/writer fuzz targets
  plus nine Kani harnesses;
- source-size enforcement at 500 lines per Rust file;
- portable core CI for WASM, AArch64 Linux, and 32-bit x86 Linux.

Two alpha.5 downstream reviews exposed visual regressions in the initial
catalog port. Family palettes, themed composition, defining geometry, face
anchors, accessory placement, and expression replacement semantics have been
corrected. The prerelease family and complete layered-corpus pixel baselines
were intentionally rebased after raster-sheet review. The remediated alpha.5
candidate requires a fresh exact-commit website comparison and pentest.

The alpha.5 implementation-stop candidate is ready for exact-commit external pentesting.
The milestone is not complete until findings are resolved, the permanent
pentest digest is added, GitHub is green, and `hashavatar-website` passes
against the final reviewed commit.

## Prerelease Policy

Alpha, beta, and release-candidate milestones are named commits, not tags or
crates.io releases. `hashavatar-website` follows each exact reviewed commit
through a local or Git dependency. Every milestone receives local release
checks, GitHub CI and CodeQL, downstream integration testing, and a permanent
digest under
[`security/pentest`](../security/pentest/README.md).

## Toolchain

| Item | Version or policy |
| --- | --- |
| MSRV | Rust `1.90.0` |
| Development toolchain | Rust `1.97.1` |
| Workspace resolver | Cargo resolver `3` |
| CodeQL | GitHub default setup |
| Unsafe Rust | Forbidden in first-party library code |
| Alpha.5 identity mode | Domain-separated SHA-512 with owned redacted identity |
| Alpha.5 core outputs | Canonical RGBA8, pixel digest, and semantic SVG |
| Alpha.5 formats | Default WebP; optional PNG, JPEG, and GIF through image 0.25.10 |
| Alpha.5 catalog | 31 families, 13 backgrounds, 5 frames, 9 accessories, 8 expressions, 6 palettes |

Version details live in [release notes](../release-notes/) and the
[changelog](../CHANGELOG.md).
