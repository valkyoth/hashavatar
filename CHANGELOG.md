# Changelog

## 2.0.0-alpha.2

- Completed the bounded private canonical renderer with rectangles, ellipses,
  lines, paths, fixed-step integer curve lowering, fill rules, clips, opacity
  groups, gradients, and exact integer source-over compositing.
- Added validated caller-owned strided RGBA8 surfaces with padding preservation
  and pre-execution dimension checks.
- Added versioned SHA-512 pixel digests over dimensions and visible rows only.
- Streamed pixel digests through clear-on-drop SHA-512 state, avoiding an
  additional image-sized preimage allocation.
- Added SVG document and fragment modes, deterministic prefixed identifiers,
  escaped accessibility text, and explicit partial-writer behavior.
- Froze alpha.2 pixel, derivation, failure, SVG, and canonical execution
  specifications and expanded Kani, fuzzing, parser, and known-answer coverage.

## 2.0.0-alpha.1

- Created the resolver-3 `hashavatar` and `hashavatar-core` workspace.
- Added a source-only canonical Cat vertical slice with bounded request input,
  stateless labeled SHA-512 traits, private checked Q16.16 geometry, validated
  scenes, CPU RGBA8, and SVG from the same scene.
- Removed the 1.x engine from `main`; `release/1.3` remains the maintained
  published line.
- Removed runtime image, codec, RNG, color, serde, and alternate-hash
  dependencies from the alpha.1 workspace.
- Added focused KATs, XML parsing, malformed-scene tests, fuzzing, Kani,
  cross-target core builds, and a 500-line Rust source limit.
- Updated repository documentation and workspace release tooling for
  source-only prerelease tags.

## 2.0 Preparation

- Reorganized repository documentation under `docs/`, moved immutable release
  notes to the root `release-notes/` directory, and added a permanent
  `security/pentest/` evidence convention for each 2.0 prerelease and stable
  tag.
- Rewrote the canonical crate/GitHub README as a technical integration guide
  centered on prepared requests, strict style validation, caller-owned output,
  typed cache keys, resource limits, and security boundaries.
- Added a workspace-aware release manifest and self-tested release script that
  validates dependency order, packages every workspace crate, prohibits
  crates.io publication for prereleases, and requires a clean signed stable
  tag plus permanent pentest evidence before publication.
- Documented the maintained `release/1.3` branch, source-only 2.0 prerelease
  policy, future companion-crate README policy, and crate version matrix.
- No runtime behavior or published 1.3.0 package metadata changed.

## 1.3.0

- Added the additive `AvatarRequest`/`AvatarRequestBuilder` and immutable
  `PreparedAvatar` migration workflow without removing or changing established
  1.x render helpers.
- Made explicit prepared-request styles strict by default, with an opt-in
  `LegacyV1` policy and `ResolvedStyle` reporting for canonicalized accessory
  and expression fallbacks.
- Added `LayoutReport` for the resolved style, family capabilities, catalog
  version, and render contract bound to a prepared request.
- Added `ResourceBudget` for tight and actual declared strided surfaces,
  temporary-render memory, returned-Vec initial base bytes, and writer-path
  base bytes, with codec scratch space and later output growth explicitly
  excluded.
- Added validated, strided `RasterSurfaceMut` output and fail-closed errors for
  zero dimensions, short strides, short buffers, overflow, and request/surface
  dimension mismatches. Internal renderer dimensions/storage and exact row
  counts are validated before success, while mismatched caller dimensions are
  rejected before rendering or temporary image allocation.
- Preserved width, height, and seed independently in both builders so invalid
  intermediate dimensions cannot silently reset the selected style seed.
- Added `PreparedAvatar::write_svg()` and `encode_to_writer()` for
  caller-owned sinks with documented partial-output and internal-allocation
  semantics.
- Added `.prepare()` adapters to `AvatarBuilder` and `StrictAvatarBuilder` so
  existing applications can migrate request and cache-key ownership without
  changing their current style-compatibility behavior.
- Added downstream-only tests for strict and legacy resolution, redacted
  diagnostics, output/key parity, resource estimates, tight and padded
  surfaces, dimension-mismatch rejection and destination preservation, short
  writes, injected writer failures, and encoded output parity.
- Added an internal injected-render regression test proving that mismatched
  surfaces are rejected before renderer invocation or temporary allocation.
- Added a complete default-SHA-512 1.x compatibility corpus containing the
  explicit request/style tuple, typed asset key, RGBA SHA-512 digest, and SVG
  SHA-512 digest for every avatar family.
- Added `docs/MIGRATION_2.0.md` with prepared-workflow examples and the explicit
  decision that applications requiring exact 1.x pixels should pin 1.3; a
  separate compatibility renderer is deferred unless real demand appears.
- Bumped crate and fuzz path dependency metadata to `1.3.0` and refreshed the
  transitive `libc` lockfile entry from `0.2.186` to `0.2.187`. No direct
  dependency declarations changed.
- Existing raster and SVG rendering behavior remains unchanged.

## 1.2.0

- Added explicit `CatalogVersion::LEGACY_V1` and
  `RenderContractId::LEGACY_V1` identifiers for the frozen Hashavatar 1.x
  automatic-selection and rendering contracts.
- Added stable built-in catalog IDs and weights through public catalog
  manifests for kinds, backgrounds, accessories, colors, expressions, and
  shapes. Automatic style derivation now routes through the legacy catalog
  without changing its byte mapping.
- Added `AvatarFamilyCapabilities`, `LEGACY_FAMILY_CAPABILITIES`, and strict
  style validation through `AvatarStyleOptions::validate_strict()` and
  `AvatarBuilder::strict_style_validation()`.
- Added typed, domain-separated `IdentityCacheKey`, `AvatarAssetKey`,
  `SemanticEncodedAssetKey`, and `BuildEncodedAssetKey` APIs. Distinct encoded
  key types prevent semantic request keys from being passed accidentally to
  deployment-specific byte caches. Avatar keys cover the active hash mode,
  catalog, render contract, dimensions, seed, and effective style. Semantic
  encoded keys additionally cover output format and fixed encoder settings.
- Added `EncoderBuildId` and deployment-bound encoded keys for shared byte
  caches, while documenting that content-addressable integrity requires hashing
  the actual encoded bytes.
- Documented that cache adapters should retain nominal key types until the
  storage boundary because string serialization erases compile-time separation.
- Canonicalized unsupported legacy accessory/expression layers during asset-key
  derivation, preventing redundant cache keys for identical output.
- Changed owned raster cleanup to sanitize full backing-vector capacity,
  including custom-renderer spare capacity.
- Preserved the existing string `cache_key()` output for downstream caches.
- Added frozen catalog vectors, complete family/style compatibility coverage,
  complete-key mutation tests, SHA-512/BLAKE3/XXH3 known-answer key tests, and
  hash-independent fixed-pixel fingerprints for every encoder contract. The
  standard gate now runs all tests in every valid hash/all-format matrix.
- Added `examples/asset_keys.rs` and documented strict validation, typed cache
  keys, catalog contracts, and the application-owned keyed-pseudonym boundary.
- Updated `sanitization` and `sanitization-crypto-interop` from `1.2.5` to
  `2.0.1`, adopting the 2.x `sanitization::wipe` API while retaining the
  existing bounded secret-container and cleaned crypto-hasher boundaries.
- Updated immutable GitHub Actions pins to `actions/checkout` `v7.0.1` and
  `taiki-e/install-action` `v2.84.0`; confirmed `Swatinem/rust-cache` remains
  current at `v2.9.1`.
- Kept keyed identity derivation outside Hashavatar 1.x. Applications that need
  dictionary resistance should derive a tenant/domain pseudonym with a managed
  key before constructing an avatar identity.
- Bumped crate and fuzz path dependency metadata to `1.2.0`.
- Confirmed all direct crate dependencies are current before release.
- Existing raster golden fingerprints remain unchanged.

## 1.1.3

- Bumped the crate and fuzz path dependency metadata to `1.1.3`.
- Prevented encoded output growth from retiring unsanitized `Vec` allocations
  by routing encoders through a sanitizing writer with controlled replacement.
- Rejected custom renderer output whose dimensions or RGBA length do not match
  the validated `AvatarSpec` before encoding.
- Namespaced generated SVG definition IDs using only dimensions, frame shape,
  and background, preventing both cross-asset definition mismatches and
  identity cache-key disclosure.
- Made stable release mode require exact Kani and SBOM tool versions and changed
  reproducibility evidence to compare complete `.crate` archives in two
  non-overrideable temporary target directories.
- Updated `sanitization` and `sanitization-crypto-interop` from `1.2.4` to
  `1.2.5`, `serde` from `1.0.228` to `1.0.229`, optional `xxhash-rust` from
  `0.8.16` to `0.8.17`, and development `serde_json` from `1.0.150` to
  `1.0.151`, together with compatible lockfile dependencies.
- Corrected the documented Kani harness count from four to five and added a
  release-metadata check that prevents the source and documented inventories
  from drifting.
- Clarified that all current public option enums are exhaustive and that adding
  variants is a breaking change reserved for 2.0, rather than a 1.x minor
  release.
- Made `docs/PLAN_TOWARDS_2.0.md` the accepted future roadmap and marked
  `docs/VERSION_PLAN.md` as historical context.
- Documented that the crate remains a single package through `1.3.0` and that
  the workspace split begins at `2.0.0-alpha.1`.
- Updated README dependency examples, provenance, security controls, and
  release metadata for `1.1.3`. Raster fingerprints remain unchanged; SVG
  definition IDs intentionally change for safe, identity-independent inline
  composition.
- Updated current integration documentation for the renamed
  `hashavatar-website` reference project and recorded the clean pentest retest
  before the final documentation-only readiness commit.

## 1.1.2

- Bumped the crate to `1.1.2`.
- Updated `sanitization` and `sanitization-crypto-interop` from `1.2.2` to
  `1.2.4`.
- Refreshed compatible transitive dependencies in the root and fuzz
  lockfiles.
- Updated the pinned development toolchain from Rust `1.96.1` to Rust `1.97.0`
  while keeping the public MSRV at Rust `1.90.0`.
- Updated the pinned GitHub CI `taiki-e/install-action` commit from the tagged
  `1.1.1` release's `v2.82.8` to `v2.83.0`.
- Confirmed `actions/checkout` remains current at `v7.0.0` and
  `Swatinem/rust-cache` remains current at `v2.9.1`.
- Updated the fuzz dependency-policy invocation for the current `cargo-deny`
  CLI while remaining compatible with the version installed by GitHub CI.
- Moved identity, cache-key, and optional XXH3 temporary preimages into
  `sanitization::SecretVec` guards before sensitive bytes are written, ensuring
  full-capacity cleanup on normal return and unwinding.
- Removed temporary lowercase allocations from all public enum parsers.
- Expanded the panic-policy gate from `src/lib.rs` to every production Rust
  module.
- Documented offline dictionary enumeration of deterministic cache keys and
  keyed pseudonymization guidance for sensitive, guessable identifiers.
- Updated README installation snippets, Rust compatibility information, Kani
  documentation, and release metadata for `1.1.2`.

## 1.1.1

- Bumped the crate to `1.1.1`.
- Updated `rand` to `0.10.2` and `xxhash-rust` to `0.8.16`.
- Refreshed compatible transitive dependencies in the root and fuzz lockfiles,
  including `arrayvec`, `chacha20`, and `hybrid-array`.
- Updated the pinned GitHub CI `taiki-e/install-action` commit to `v2.82.8`.
- Confirmed `actions/checkout` remains current at `v7.0.0` and
  `Swatinem/rust-cache` remains current at `v2.9.1`.
- Updated README installation snippets, latest-stable Rust wording, and release
  metadata for `1.1.1`.
- Added local compatibility evidence for Rust `1.96.1`.
- Switched the pinned development toolchain to Rust `1.96.1` while keeping the
  crate MSRV at Rust `1.90.0`.
- Added CI/release checks that verify backward compatibility on Rust `1.90.0`.
- Added bounded Kani proof harnesses for `AvatarSpec` bounds, render-resource
  memory math, and rectangle arithmetic.
- Added `scripts/check_kani.sh` using the documented Rust `1.90.0` verifier
  toolchain when available, plus `docs/KANI.md` to define the proof scope and
  skip policy.

## 1.1.0

- Bumped the crate to `1.1.0`.
- Replaced direct `zeroize` usage with the native `sanitization` crate API.
- Added `sanitization` `1.2.2` with `alloc` support for digest, seed,
  preimage, pixel-buffer, and temporary encoder-buffer cleanup.
- Added `sanitization-crypto-interop` `1.2.2` so SHA-512 and optional BLAKE3
  hashing use the crypto crates' own hasher-state cleanup hooks through the
  `sanitization` sister crate.
- Removed direct `zeroize` dependency usage and the `sha2`/`blake3` zeroize
  feature hooks.
- Moved cache-key and identity SHA-512 hashing through the crypto interop
  helper, and moved optional BLAKE3 XOF output through the interop helper with a
  `sanitization::Secret` output buffer.
- Removed a redundant SHA-512 digest `Secret` wrapper now that the interop
  helper owns hasher-state cleanup and the caller already guards the returned
  digest.
- Moved the crate's direct `sha2` dependency to dev-dependencies; production
  SHA-512 hashing now reaches `sha2` through `sanitization-crypto-interop`.
- Guarded the optional XXH3 64-byte accumulator with `sanitization::Secret`.
- Promoted hash-preimage capacity checks from debug-only assertions to release
  assertions so future size-accounting drift cannot silently bypass temporary
  buffer sanitization.
- Collapsed optional XXH3 chunk capacity and length checks into one release
  assertion per chunk.
- Tightened `cargo-deny` duplicate crate policy from `warn` to `deny`.
- Updated `libfuzzer-sys` in the fuzz harness to `0.4.13`.
- Refreshed Cargo lockfiles with the latest compatible dependency versions.
- Split the former monolithic `src/lib.rs` into focused source files, including
  per-avatar raster and SVG renderer files, while preserving the public API and
  visual fingerprints.
- Clarified that preimage-capacity assertions are active in all builds as an
  intentional fail-secure sanitization guard.
- Made starry-background seed rotation precedence explicit without changing
  generated visuals.
- Hardened internal gradient color interpolation against future oversized
  callers.
- Added consistent `identity()` accessors and security notes to hashed dog and
  robot renderer structs, matching the existing hashed cat renderer.
- Updated GitHub Actions pins to `actions/checkout` `v7.0.0` and
  `taiki-e/install-action` `v2.82.3`; `Swatinem/rust-cache` remains current at
  `v2.9.1`.
- Refreshed README installation snippets and release metadata for `1.1.0`.

## 1.0.3

- Bumped the crate to `1.0.3`.
- Hardened encoded-output error paths by keeping encoder output in a
  `Zeroizing<Vec<u8>>` until successful return, so partially encoded bytes are
  scrubbed on encoder errors.
- Added debug/test assertions for exact identity, cache-key, and XXH3 chunk
  preimage capacities so future edits cannot silently introduce reallocations
  before zeroization.
- Redacted `AvatarBuilder` namespace tenant and style-version values from
  `Debug` output in addition to the identity input.
- Updated `docs/SECURITY_CONTROLS.md` to accurately describe accepted
  upper-digest-byte visual parameter usage for `1.x` visual stability.
- Updated GitHub Actions pins to `actions/checkout` `v6.0.3`,
  `Swatinem/rust-cache` `v2.9.1`, and `taiki-e/install-action` `v2.81.8`.
- Removed the temporary pentest report from the tree.

## 1.0.2

- Bumped the crate to `1.0.2`.
- Added `AvatarBuilder` as a fluent, validation-preserving entry point for
  rendering SVG, raster images, encoded WebP/optional formats, and cache keys.
- Added `AvatarError` as a unified high-level error type for builder callers.
- Added `AvatarIdentity::cache_key()` for stable opaque cache identifiers
  without exposing the raw identity digest.
- Added a `prelude` module for common application imports.
- Added an optional `serde` feature for public style enums only. `AvatarIdentity`
  intentionally remains non-serializable.
- Added `AvatarStyleOptions::summary()` and `Display` for UI/log labels.
- Expanded `AvatarSpec::new` and builder docs around the style-variant seed.
- Added runnable `examples/` for the builder and cache-key workflows.
- Removed the temporary gap analysis report from the tree.

## 1.0.1

- Bumped the crate to `1.0.1`.
- Refreshed compatible transitive dependencies in `Cargo.lock` and the fuzz
  harness lockfile.
- Updated the GitHub CI `taiki-e/install-action` pin to `v2.79.14`.
- Lowered the documented and manifest MSRV to Rust `1.90.0` after running the
  full release gate on `1.90.0` and compatibility checks through Rust `1.96.0`.
- Added the README header artwork and Rust version support table. The README
  image is kept in the repository but excluded from the published crate package.
- Added a debug assertion for future out-of-range identity digest byte access
  while keeping release builds non-panicking.
- Changed public avatar size helper arithmetic to saturating multiplication for
  future-proofing.
- Clarified that the `StdRng::from_seed` by-value seed argument copy is part of
  the documented zeroization residual.

## 1.0.0

- Bumped the crate to `1.0.0`.
- Declared the first stable public API and rendering contract for
  `hashavatar`.
- Added `docs/STABILITY.md` covering Cargo semver expectations, visual output
  stability, automatic-style distribution, security/resource contracts, and
  known residual risks.
- Updated README guidance for the stable contract and controlled visual
  rollouts through namespace `style_version` values.
- Added `docs/STABILITY.md` to release metadata/package validation so future
  releases keep the stability policy in the published crate.
- Added 1.0 release notes.
- Hardened the 1.0 release after pentest review by keeping the renderer RNG
  seed copy in a `Zeroizing` guard until seeding, guarding the intermediate
  identity digest copy, making starry raster backgrounds identity-dependent,
  clamping gradient interpolation inputs, and removing the exact rejected byte
  count from `AvatarIdentityError` display text.
- Documented target-specific `getrandom` considerations, XXH3 temporary
  preimage-copy overhead, and rectangle zero-size clamping behavior.
- Added defensive digest-byte indexing and widened the avatar fuzz harness so it
  covers the full supported `64..=2048` dimension range.
- No avatar families, backgrounds, visual layers, hash modes, output formats,
  or runtime dependencies were added in this release.

## 0.13.0

- Bumped the crate to `0.13.0`.
- Added seven `AvatarBackground` variants: `polka-dot`, `striped`,
  `checkerboard`, `grid`, `sunrise`, `ocean`, and `starry`.
- Implemented the new backgrounds for raster and SVG output with bounded,
  asset-free drawing paths.
- Added parser/display/`ALL` coverage, raster distinctness tests, and
  parser-backed SVG well-formedness coverage for the expanded background
  catalog.
- Replaced raster frame-shape hit-testing with integer arithmetic for circle,
  squircle, hexagon, and octagon masks, reducing floating-point sensitivity in
  shape clipping.
- Updated automatic visual golden fingerprints because the expanded
  `AvatarBackground::ALL` list changes automatic background distribution.
- Fixed documentation that still referenced `cargo test --all-features`; the
  `blake3` and `xxh3` feature modes are intentionally mutually exclusive.
- Refreshed README wording for feature-gated encoder testing and the expanded
  background catalog.

## 0.12.0

- Bumped the crate to `0.12.0`.
- Added eight built-in avatar families: `bear`, `penguin`, `dragon`,
  `ninja`, `astronaut`, `diamond`, `coffee-cup`, and `shield`.
- Implemented each new family in both raster and SVG rendering paths.
- Added face-layer anchor coverage for `bear`, `penguin`, `dragon`, `ninja`,
  and `astronaut`.
- Kept object/symbol families such as `diamond`, `coffee-cup`, and `shield`
  as deterministic no-ops for accessories and expressions.
- Updated enum drift tests, parser/display coverage, golden visual
  fingerprints, README catalog, and release notes for the expanded family list.
- Documented that automatic style derivation can map some identities to
  different families because `AvatarKind::ALL` now contains more variants.
- Removed the public `AvatarIdentity::seed()` helper and made the internal
  256-bit RNG seed helper private so callers do not accidentally depend on raw
  digest-derived seed material.
- Removed the public `AvatarIdentity::as_digest()` helper so normal rendering
  callers cannot copy or log full identity digests through the public API.
- Added a regression test and documentation that lock `AvatarSpec::default()`
  to the deterministic `256x256` seed-`1` convenience spec.
- Removed ad-hoc SVG string minification and added parser-backed SVG
  well-formedness coverage across avatar families, visual layers, representative
  identities, and the fuzz harness.
- Added `AvatarRenderResourceBudget` and `AvatarSpec::render_resource_budget`
  so service integrations can size render concurrency limits from API-visible
  memory estimates instead of only README prose.
- Hardened polygon scanline interpolation against debug-build integer overflow
  and added a dedicated fuzz target for arbitrary polygon rasterizer inputs.
- Removed runtime identity hash algorithm selection. SHA-512 remains the
  default crate-wide mode, while `blake3` and `xxh3` select mutually exclusive
  crate-wide optional modes.
- Replaced derived `AvatarIdentity` debug formatting with a redacted
  implementation so accidental `{:?}` logging does not expose identity digests.
- Added rustdoc and security-control guidance that `AvatarIdentity` clones are
  independently zeroized on drop, but high-assurance callers should keep clone
  lifetimes short to limit extra live digest copies.
- Enabled upstream hasher-state zeroization features for SHA-512 and BLAKE3,
  and explicitly zeroized BLAKE3 hasher/XOF reader state after digest
  derivation.
- Wrapped digest-derived renderer RNG seed copies in `zeroize::Zeroizing` so
  the temporary mixed seed is scrubbed immediately after RNG initialization,
  and documented `StdRng`'s non-zeroized expanded internal state as a known
  residual.
- Wrapped owned RGBA encode buffers and JPEG RGB flattening buffers in RAII
  zeroization guards so temporary pixel data is scrubbed during normal returns,
  encoder errors, and unwinding panics.
- Added a zero-dimension guard to polygon rasterization so fuzz-only zero-width
  or zero-height image inputs return cleanly instead of panicking.
- Moved PNG, JPEG, and GIF output behind explicit `png`, `jpeg`, and `gif`
  Cargo features, leaving WebP as the only default raster encoder.
- Added rustdoc plus security documentation warning that `image`'s internal GIF
  quantization buffers are not zeroized by `hashavatar`.
- Hardened rectangle intersection size calculation with saturating arithmetic
  for extreme internal coordinate ranges.
- Added a compile-time guard so the internal `fuzzing` feature cannot be used
  in ordinary non-fuzzing release builds.
- Documented that XXH3-128 is non-cryptographic and must not be used with
  adversarial, user-controlled, or sensitive identifiers unless the application
  first maps those identifiers through its own cryptographic boundary.

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
