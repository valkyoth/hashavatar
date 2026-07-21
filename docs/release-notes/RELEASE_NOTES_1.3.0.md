# hashavatar 1.3.0

`1.3.0` is an additive migration release. It lets applications adopt the
request preparation, ownership, metadata, resource, and cache-key workflow
planned for 2.0 while continuing to use the frozen Hashavatar 1.x renderer.

## Prepared Requests

- Added `AvatarRequest` and `AvatarRequestBuilder`, which own a derived
  `AvatarIdentity` rather than retaining raw identity input.
- Added immutable `PreparedAvatar`, binding one validated spec, requested and
  effective styles, family capabilities, resource estimate, cache keys, and
  all output methods.
- Explicit request styles are strict by default. Unsupported family accessory
  or expression layers return `AvatarRequestError::Style`.
- Added opt-in `AvatarCompatibilityMode::LegacyV1` and
  `.legacy_v1_compatibility()` for applications that must reproduce the old
  deterministic skip behavior.
- Added `ResolvedStyle` so callers can inspect requested/effective styles,
  automatic derivation, and ignored legacy layers.
- Added `LayoutReport`, including the frozen catalog/render contract and family
  capabilities used by the prepared request.
- Added `.prepare()` to `AvatarBuilder` and `StrictAvatarBuilder`. These
  adapters retain each builder's existing compatibility behavior.

## Output Ownership And Resources

- Added `ResourceBudget` for minimum tight and actual declared strided surface
  accounting, the known 1.x render-adapter temporary allocation, returned-Vec
  initial base bytes, and writer-path base bytes. Codec scratch space and later
  output growth remain explicitly excluded.
- Added `RasterSurfaceMut::new_rgba8()` with checked stride, length, and
  dimension validation.
- Added `PreparedAvatar::render_into()` for tight or padded caller-owned RGBA8
  surfaces. The 1.x adapter still renders through one sanitized temporary
  `RgbaImage`; it validates renderer dimensions/storage and copies an exact
  checked row count before reporting success. Caller-surface dimension
  mismatches fail before rendering or temporary image allocation. It is not a
  zero-allocation renderer.
- Added `write_svg()` and `encode_to_writer()` for caller-owned sinks. Partial
  output remains owned by the caller after a writer/codec error. SVG and codec
  internals can still allocate documented temporary buffers.

## Compatibility Evidence

- Added external integration tests covering strict rejection, legacy fallback
  reporting, redacted diagnostics, existing-builder output/key parity, resource
  math, padded stride preservation, surface-mismatch rejection and destination
  preservation, short writes, writer failures, and WebP writer/`Vec` parity.
- Added an internal injected-render regression test proving that mismatched
  surfaces are rejected before renderer invocation or temporary allocation.
- Added `tests/compatibility_corpus_v1.tsv`, freezing one complete explicit
  request/style tuple, `AvatarAssetKey`, RGBA SHA-512 digest, and SVG SHA-512
  digest for every 1.x avatar family under the default SHA-512 identity mode.
- Existing golden raster fingerprints and established output helpers remain
  unchanged.
- Preserved width, height, and seed independently in both builders so invalid
  intermediate dimensions cannot silently reset a selected style seed.

## 2.0 Migration Decision

`docs/MIGRATION_2.0.md` documents the application migration sequence. Exact
1.x rendering remains supported throughout 1.x. Hashavatar 2.0 may deliberately
change pixels; applications requiring exact 1.x output should pin `=1.3.0`.
A separate compatibility renderer is not planned unless downstream demand
justifies maintaining and auditing two engines.

## Dependencies

- Bumped the root crate and fuzz path dependency metadata to `1.3.0`.
- Refreshed the transitive `libc` lockfile entry from `0.2.186` to `0.2.187`.
  No direct dependency declarations changed.

The full local stable release gate passed, including the Rust `1.90.0` feature
checks, all valid hash/format test matrices, strict Clippy/rustdoc, RustSec and
dependency policy, fuzz harness compilation, all five Kani proofs,
byte-identical package archives, SBOM generation, and crates.io publish
dry-run. `cargo-semver-checks` against `v1.2.0` passed for the default,
BLAKE3, and XXH3 feature sets, confirming that the public API changes are
additive. Independent pentesting completed cleanly after the reported
assurance findings were resolved. GitHub validation is green, and downstream
`hashavatar-website` integration testing completed successfully.
