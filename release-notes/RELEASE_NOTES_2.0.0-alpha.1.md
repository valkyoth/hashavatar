# hashavatar 2.0.0-alpha.1

`2.0.0-alpha.1` is the first source-only Hashavatar 2.0 architecture release.
It is tagged for GitHub, pentest, and `hashavatar-website` integration testing
but is not published to crates.io. Production users remain on `1.3.x`.

## Workspace

- Converted the repository to a Cargo resolver-3 workspace.
- Added `hashavatar-core 0.1.0-alpha.1` as a `no_std + alloc` canonical core.
- Replaced the root package with a thin `hashavatar 2.0.0-alpha.1` facade.
- Removed the 1.x renderer from `main`; it remains supported on `release/1.3`.
- Kept every current Rust source file at or below 500 lines.

## Cat Vertical Slice

- Added bounded borrowed `CatRequest` construction and preparation.
- Added length-prefixed, domain-separated SHA-512 identity derivation without
  retaining caller identity or namespace bytes.
- Added independent label-derived `CatTraitVector` samples with known-answer
  tests and no mutable rendering RNG.
- Added private checked signed Q16.16 geometry and a flat scene capped at 16
  commands.
- Added a Cat compiler with a themed background, ears, head, muzzle, eyes, and
  nose.
- Added scene validation for dimensions, command count, paint, coordinates,
  ellipse bounds, and nondegenerate triangles.
- Added one canonical safe-Rust CPU executor returning tightly packed
  straight-alpha RGBA8.
- Added deterministic SVG serialization from the same scene with exact Q16.16
  decimal values, an 8 KiB fail-closed document bound, and no caller-controlled
  markup.
- Added `SceneReport` with exact RGBA allocation and conservative pixel-test
  accounting.

## Assurance

- Added debug/release pixel fingerprint and repeatability tests.
- Added parser-backed SVG and command-count parity tests.
- Added malformed-scene and public-bound regression tests.
- Replaced legacy fuzz targets with a bounded request/prepare/raster/SVG target.
- Replaced 1.x Kani proofs with five focused core arithmetic and admission
  harnesses.
- Added WASM, AArch64 Linux, and 32-bit x86 Linux core compile jobs.
- Updated dependency, panic, unsafe, package, and release checks for the
  workspace.
- Excluded GitHub administration, archived design drafts, fuzz sources,
  pentest evidence, and generated output from the facade crate archive.
- Reproducibility checks compare the complete core archive byte-for-byte. The
  facade file list and full workspace build are checked separately because its
  source-only core dependency is intentionally unavailable from crates.io
  during prereleases.
- Runtime dependencies are limited to `sanitization` and
  `sanitization-crypto-interop`; XML parsing remains dev/fuzz-only.

## Current Limits

Alpha.1 supports only Cat, themed background, RGBA8, and SVG. It does not yet
provide the existing catalog, layered styles, encoders, image compatibility,
schema, heapless storage, cache keys, or GPU execution. APIs and pixels may
change in later prereleases.
