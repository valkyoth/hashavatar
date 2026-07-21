# hashavatar 2.0.0-alpha.5

`2.0.0-alpha.5` is a source-only implementation-stop milestone. It is not
tagged or published to crates.io, and its API, keys, and pixels remain
prerelease.

## Prepared Request Boundary

- Adds owned, redacted `AvatarIdentity` construction that retains no raw input.
- Adds the recommended `AvatarRequestBuilder` while retaining direct request
  convenience constructors.
- Adds `ResourceBudget`, `ReusableRgbaBuffer`, `IdentityCacheKey`,
  `AvatarAssetKey`, `CatalogVersion`, and `RenderContractId`.
- Renames the primary typed error to `AvatarError`; `CatError` remains a
  transitional alias for the vertical-slice API.

## Established Formats

- Adds the isolated `hashavatar-formats` package.
- Enables lossless WebP by default and admits explicit PNG, JPEG, and GIF
  features plus `all-formats`.
- Adds allocating and writer APIs, reusable RGBA scratch, typed format errors,
  exact alpha capability metadata, semantic encoded keys, and caller-bound
  build keys.
- Freezes image-rs 0.25.10 settings for lossless WebP, best/adaptive PNG,
  quality-92 JPEG over white, and speed-1 single-frame GIF.

## Assurance

- Corrects the initial catalog port after the first deployed 1.3-versus-2.0
  website review exposed generic palettes and over-simplified family geometry.
- Restores family-aware default colors, light themed fields with accent motifs,
  and the defining geometry needed to recognize all 31 existing subjects.
- Intentionally rebases prerelease canonical pixels while preserving frozen
  catalog IDs, stateless derivation, and alpha.4 layout behavior.
- Adds ordered per-family pixel fingerprints alongside the aggregate catalog
  KAT, so failures identify the affected family directly.
- Adds exact WebP/PNG decode-to-canonical comparisons, bounded JPEG/GIF lossy
  evidence, writer-failure tests, key tests, scratch-reuse tests, and a bounded
  format/writer fuzz target.
- Adds codec-free/default/per-format/all-format dependency and build gates,
  MSRV checks, package inspection, a reproducible core archive, and stable
  formats/facade file-list evidence. Their registry archives remain deferred
  while prerelease dependencies are intentionally unpublished.

The alpha.5 visual baseline is accepted only after reviewing complete raster
and SVG contact sheets and testing the exact candidate in
`hashavatar-website`. Exact 1.3 pixels remain available from the `release/1.3`
branch; 2.0 preserves subject identity and visual intent, not the old floating-
point raster implementation.

Pentest the exact implementation-stop commit announced by the maintainer. The
permanent digest is added only after the external review passes.
