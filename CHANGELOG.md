# Changelog

## 0.11.0

- Bumped the crate to `0.11.0`.
- Added explicit `AvatarKind::supports_face_layers()` guidance for families
  that can place accessories and expressions.
- Made non-square SVG frame shapes clip the background, avatar body, accent,
  accessory, and expression content so SVG behavior matches raster masking.
- Fixed malformed Paws-family SVG output where one toe-pad ellipse wrote a
  color value into its `ry` radius attribute.
- Lowered glasses placement slightly for dog, robot, monster, ghost, wizard,
  and knight families.
- Tuned eyepatch, horns, bowtie, crown, hat, and headphones placement for
  families where those overlays were visibly off-center.
- Expanded tests for face-layer support, deterministic fallback behavior, and
  SVG frame clipping.

## 0.10.0

- Bumped the crate to `0.10.0`.
- Added visual layer enums: `AvatarAccessory`, `AvatarColor`,
  `AvatarExpression`, and `AvatarShape`.
- Added `AvatarStyleOptions` for explicit kind, background, accessory, color,
  expression, and frame-shape selection.
- Added style-aware raster, SVG, and encode entry points alongside the existing
  `AvatarOptions` API.
- Added automatic style rendering helpers that derive top-level style choices
  from distinct identity digest bytes.
- Kept existing `AvatarOptions` output unchanged by mapping it to no accessory,
  default color, default expression, and square frame.
- Added generic raster and SVG support for all baseline visual layers.
- Added family-aware face anchors for accessories and expressions, with
  deterministic no-op behavior for non-face families where those layers do not
  make sense.
- Expanded enum drift tests, automatic derivation tests, layer rendering tests,
  fuzz coverage, and golden visual fingerprints for representative layered
  avatars.

## 0.9.0

- Bumped the crate to `0.9.0`.
- Kept `hashavatar` as a single image-generation crate rather than publishing a
  separate core crate.
- Removed the near-term `no_std + alloc` crate split from the roadmap because
  the public project goal is raster/SVG avatar generation.
- Documented that lower-level planning boundaries should remain internal unless
  a future image-generation use case justifies exposing them.

## 0.8.0

- Bumped the crate to `0.8.0`.
- Added an internal render plan boundary shared by raster rendering, SVG
  rendering, and encoding entry points.
- Changed public enum `ALL` lists to slices so variant lists no longer carry
  duplicated manual array lengths.
- Added `from_byte` helpers for `AvatarHashAlgorithm`, `AvatarKind`,
  `AvatarBackground`, and `AvatarOutputFormat`.
- Added tests that fail if public enum `ALL` lists drift from parser/display
  behavior.
- Documented the future `no_std + alloc` core boundary and the dependencies
  that currently belong outside it.
- Added public raw RGBA buffer budget constants and `AvatarSpec` helpers for
  service-level memory/concurrency controls.
- Documented the current variable-time rendering and floating-point
  cross-platform determinism boundaries.
- Hardened antialiasing channel blending against non-finite or zero-total
  weights.
- Zeroized temporary owned raster buffers after encode APIs finish encoding.
- Added documentation for fixed-minimum-latency API wrappers that can reduce
  render-time side-channel observability in high-assurance deployments.

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
- Added constant-time equality for `AvatarIdentity`.
- Documented that rendering itself is not constant-time and should not be
  treated as secret-preserving against timing or output-size side channels.

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
