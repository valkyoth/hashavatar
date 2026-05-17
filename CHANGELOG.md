# Changelog

## 0.7.0

- Bumped the crate to `0.7.0`.
- Added `AvatarHashAlgorithm` and `AvatarIdentityOptions`.
- Kept SHA-512 as the default identity hash and preserved the existing default identity preimage.
- Added optional BLAKE3 identity derivation behind the `blake3` Cargo feature.
- Added optional XXH3-128 identity derivation behind the `xxh3` Cargo feature.
- Added domain separation for non-default hash algorithms.
- Added generic render/encode/SVG entry points that accept identity hash options.
- Added tests for feature-gated hash modes, algorithm separation, parser round-trips, and oversized identity rejection across enabled algorithms.
- Documented the optional hash algorithms, dependency posture, and the non-cryptographic status of XXH3-128.
- Hardened JPEG alpha flattening with wider arithmetic intermediates.
- Hardened anti-aliased zero-length line drawing against NaN gradient propagation.
- Added `zeroize` cleanup for derived identity digests and temporary identity hash preimage buffers.
- Changed procedural cat RNG seeding to use 256 bits from the second half of
  the identity digest, intentionally updating cat-family golden fingerprints.

## 0.6.0

- Removed the bundled demo web server from the crate
- Removed mandatory `axum` and `tokio` dependencies from the crate package
- Removed the bundled `hashavatar-cli` binary so the package is a pure library crate
- Pointed web/API usage to the separate `hashavatar-api` project
- Added crate-focused security policy checks for release metadata, dependencies, unsafe code, panic-like sites, package contents, and fuzz harness compilation
- Added fuzz harness coverage for arbitrary avatar identities, families, backgrounds, SVG rendering, and PNG encoding
- Added release security documentation for dependency policy, panic policy, release gates, and crate security controls
- Changed public render APIs and `AvatarRenderer::render` to return `Result<_, AvatarSpecError>` for invalid dimensions instead of panicking
- Changed `AvatarSpec::new` to validate dimensions at construction and made `AvatarSpec` fields private
- Added enforced identity and namespace byte-length limits with typed errors to prevent accidental hashing of unbounded input
- Removed public path-writing export helpers so the crate does not provide filesystem APIs that can be wired to untrusted paths
- Changed namespace identity hashing to length-prefix tenant, style version, and input components, preventing embedded NUL separator ambiguity
- Hardened internal rectangle arithmetic with saturating and clamping operations
- Hardened internal polygon and ellipse rasterization against edge-case panics and large-radius precision loss
- Added post-0.6 version planning for pluggable hashing, no-std preparation, visual layers, variant expansion, and 1.0 stabilization
- Documented maintenance rules for dependency freshness, security review, GitHub CodeQL default setup, and self-testing expectations

## 0.5.0

- Starting with `0.5.0`, project licensing is dual `MIT OR Apache-2.0`
- Added `LICENSE-MIT` and `LICENSE-APACHE`
- Removed the previous EUPL license files
- Added Fluxheim-style local and GitHub CI checks through `scripts/checks.sh`
- Added Dependabot configuration for Cargo and GitHub Actions updates
- Pinned GitHub Actions to immutable commit SHAs for CodeQL-friendly workflow hardening
- Moved demo-server WebP rendering and encoding onto Tokio's blocking task pool
- Added defense-in-depth HTTP security headers to demo HTML, image, and error responses
- Updated `tokio` to `1.52.3`

## 0.4.2

- Moved public repository and homepage metadata to GitHub
- Added GitHub contributor, security, issue, pull request, and CI files
- Kept docs.rs as the canonical Rust API documentation URL

## 0.4.1

- Updated direct dependencies to current compatible releases
- Moved deterministic randomization from `rand` 0.9 to `rand` 0.10
- Moved SHA-2 hashing from `sha2` 0.10 to `sha2` 0.11

## 0.4.0

- Added `transparent` background support for raster and SVG avatars
- Added `black`, `dark`, and `light` background modes
- Added `JPEG` and `GIF` raster export formats
- Added new avatar families: `planet`, `rocket`, `mushroom`, `cactus`, `frog`, and `panda`
- Added new food and adventure families: `cupcake`, `pizza`, `icecream`, `octopus`, and `knight`
- Improved identity-driven variation for `ghost`, `slime`, `wizard`, and `skull`
- Added stricter input and dimension validation for safer public avatar endpoints
- Removed a vulnerable transitive dependency path while keeping drawing code asset-free
- Refreshed README examples and demo preset identities for the latest public API

## 0.3.0

- Added namespace-aware identity hashing through `AvatarNamespace`
- Declared `AVATAR_STYLE_VERSION` for stable visual contract tracking
- Added new avatar families: `ghost`, `slime`, `bird`, `wizard`, and `skull`
- Added golden visual regression fixtures for stable raster fingerprints
- Added stricter SVG regression coverage for determinism and minimal output
- Expanded the public API site with docs, metrics, SEO metadata, JSON-LD, sitemap, robots, favicon, and manifest support
- Added origin-side rate limiting, timeout handling, metrics, and object-storage deduplication in the API service
- Added OG/social preview image support
- Added a playful `paws` avatar family with variable cat paw colors and pad shapes

## 0.1.0

- Initial release of `hashavatar`
- Deterministic SHA-512-backed avatar generation
- Raster export in `WebP` and `PNG`
- SVG export support
- Initial families: `cat`, `dog`, `robot`, `fox`, `alien`, and `monster`
