# hashavatar 0.12.0

`0.12.0` expands the built-in avatar catalog while keeping `hashavatar` as a
single, asset-free image-generation crate.

## Added

- Added raster and SVG renderers for eight new `AvatarKind` values:
  - `bear`
  - `penguin`
  - `dragon`
  - `ninja`
  - `astronaut`
  - `diamond`
  - `coffee-cup`
  - `shield`
- Added face-layer anchors for the new face families: `bear`, `penguin`,
  `dragon`, `ninja`, and `astronaut`.
- Added golden visual fingerprints for every new family.
- Updated README option catalogs and examples for the expanded family list.

## Compatibility Notes

- Explicit `AvatarOptions` selections remain deterministic for the selected
  kind, background, dimensions, seed, namespace, crate identity hash mode, and
  identity.
- Automatic style derivation uses `AvatarKind::ALL`, so adding new family
  variants changes the automatic family distribution. Services that need old
  automatic output should keep their existing namespace `style_version` until
  they intentionally migrate.
- `AvatarIdentity::seed()` and `AvatarIdentity::as_digest()` are removed from
  the public API, and the internal 256-bit RNG seed helper is private.
  Rendering callers should not handle raw identity digest bytes.
- `docs/SECURITY_CONTROLS.md` now states that identity preimage allocation does
  not hide input length at the allocator level. High-assurance callers should
  pad or normalize sensitive identifiers before passing them to the crate.
- `AvatarSpec::default()` is documented and tested as a fixed deterministic
  `256x256` seed-`1` convenience spec, not as a random or production policy
  default.
- SVG rendering no longer uses ad-hoc `String::replace` minification. The test
  suite and fuzz harness now parse generated SVG with `roxmltree` to verify
  XML well-formedness across families, layers, and representative identities.
- `AvatarRenderResourceBudget` and `AvatarSpec::render_resource_budget(...)`
  make raw RGBA memory estimates explicit for service-level render concurrency
  limits without adding an async runtime or semaphore dependency to the crate.
- Polygon scanline interpolation now widens coordinate deltas before rounding,
  and the fuzz harness includes a dedicated polygon rasterizer target for
  degenerate, negative-coordinate, and extreme-point inputs.
- Runtime identity hash algorithm selection has been removed. The crate uses
  SHA-512 by default, BLAKE3 when built with `blake3`, and XXH3-128 when built
  with `xxh3`. The `blake3` and `xxh3` features are mutually exclusive, and
  XXH3-128 remains documented as non-cryptographic and unsuitable for
  adversarial, user-controlled, or sensitive identifiers.
- `AvatarIdentity` debug formatting is now redacted, preventing accidental
  `{:?}` logging from exposing the raw 64-byte identity digest.
- `AvatarIdentity` rustdoc and security controls now state that clones are
  zeroized independently on drop, and that high-assurance callers should keep
  clone lifetimes short to avoid unnecessary live digest copies.
- SHA-512 is built with upstream `zeroize` support so its block buffer uses
  `ZeroizeOnDrop`. BLAKE3 is built with upstream `zeroize` support, and the
  BLAKE3 hasher plus XOF reader are explicitly zeroized after digest
  derivation.
- Digest-derived renderer RNG seed copies are now wrapped in
  `zeroize::Zeroizing`, so the temporary mixed seed is scrubbed after RNG
  initialization.
- Owned RGBA encode buffers and JPEG RGB flattening buffers now use RAII
  zeroization guards, so temporary pixel data is scrubbed during normal
  returns, encoder errors, and unwinding panics.
- Polygon rasterization now returns immediately for zero-width or zero-height
  images, keeping the fuzz-only polygon harness from reporting artificial
  zero-sized-image crashes.
- PNG, JPEG, and GIF output are now behind explicit `png`, `jpeg`, and `gif`
  Cargo features, leaving WebP as the only default raster encoder. The
  `AvatarOutputFormat::Gif` rustdoc and security controls call out that the
  `image` crate's internal GIF quantization buffers are not zeroized by
  `hashavatar`, so high-assurance deployments should prefer WebP or PNG.
- Rectangle intersection size calculation now uses saturating arithmetic for
  extreme internal coordinate ranges.
- The hidden `fuzzing` feature now has a compile-time guard that rejects
  ordinary non-fuzzing release builds, reducing the chance that internal fuzz
  harness entry points are accidentally exposed in production.
- `diamond`, `coffee-cup`, and `shield` are object/symbol families. They do
  not have face anchors, so accessories and expressions are deterministic
  no-ops for those families. Accent palettes and frame shapes still apply.

## Deferred

- Pattern, gradient, and environment backgrounds were not admitted in this
  release. They need their own bounded raster/SVG texture path and visual
  contrast review before becoming public API.
