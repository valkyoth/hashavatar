# hashavatar 1.2.0

`1.2.0` is an additive migration-preparation release. It gives applications
stable names for the Hashavatar 1.x catalog and renderer contracts, complete
cache keys, and an opt-in fail-closed style-validation path. Existing render
APIs and pixels remain compatible.

## Contract Metadata

- Added `CatalogVersion::LEGACY_V1` and `RenderContractId::LEGACY_V1`.
- Added frozen public catalog manifests with explicit IDs and weights for every
  built-in kind, background, accessory, color, expression, and shape.
- Routed automatic style derivation through the legacy catalog while preserving
  every existing byte-to-style mapping.
- Added `AvatarFamilyCapabilities` and `LEGACY_FAMILY_CAPABILITIES` so callers
  can inspect supported layers without duplicating family lists.

## Strict Style Validation

- Added `AvatarStyleOptions::validate_strict()` for direct validation.
- Added `AvatarBuilder::strict_style_validation()` and `StrictAvatarBuilder`
  for validation immediately before key derivation, rendering, SVG generation,
  or encoding.
- Strict mode rejects accessories and expressions for families without face
  anchors. Legacy APIs retain their deterministic skip behavior, so this
  release does not alter existing output.

## Complete Asset Keys

- Added `IdentityCacheKey` for a domain-separated active-hash-mode identity.
- Added `AvatarAssetKey` covering identity, catalog, render contract,
  dimensions, seed, and all effective style layers. Ignored legacy face layers
  are canonicalized so identical output receives one key.
- Added `SemanticEncodedAssetKey` derivation covering the avatar key, output
  format, and fixed encoder settings.
- Added `EncoderBuildId` and distinct `BuildEncodedAssetKey` derivation for
  applications that share byte caches across deployment builds. The nominal
  types prevent semantic and deployment-specific cache keys from being mixed.
  Actual encoded-byte hashing remains required for content-addressable
  integrity.
- Preserved `AvatarIdentity::cache_key()` and `AvatarBuilder::cache_key()` with
  their exact previous output for existing caches.
- Added `examples/asset_keys.rs` for the recommended typed-key workflow.

All three key types are public cache identifiers. They support stable storage
and correlation; they are not secrets, password hashes, or authorization
tokens.

## Keyed Identity Decision

Hashavatar 1.x does not accept or manage tenant keys. Key storage, rotation,
domain separation, and pseudonym lifetime belong to the application. Services
with sensitive, guessable identifiers should derive a pseudonym with a reviewed
keyed construction and pass only that pseudonym to Hashavatar. This keeps the
crate from implying a key-management guarantee it cannot enforce.

## Dependencies

- Updated `sanitization` and `sanitization-crypto-interop` from `1.2.5` to
  `2.0.1`.
- Migrated direct returned-buffer cleanup to `sanitization::wipe::bytes` and
  `sanitization::wipe::vec`.
- The 2.x dependencies retain the crate's Rust `1.90.0` MSRV and the existing
  SHA-512/BLAKE3 internal-state cleanup boundary.

## Verification

- Frozen string vectors cover every catalog ID, weight, and label.
- Every automatic input byte is checked against the previous public mapping.
- The family compatibility matrix covers every kind/accessory/expression tuple.
- Complete-key tests prove that every effective render input and enabled
  encoder contract changes the appropriate key, while ignored legacy layers
  canonicalize to one key.
- Frozen fixed-pixel encoded-byte fingerprints bind each encoder contract ID to
  its current WebP, PNG, JPEG, or GIF output settings independently of the
  selected identity-hash feature.
- SHA-512, BLAKE3, and XXH3 builds have frozen known-answer vectors for the
  identity, avatar, and semantic encoded key levels. Separate tests enforce the
  build-bound key's nominal type and build-ID separation.
- The standard gate runs the complete test suite in SHA-512, BLAKE3, and XXH3
  modes with every optional encoder and serde enabled in valid combinations.
- Owned raster cleanup now sanitizes the entire backing-vector capacity,
  including spare capacity supplied by custom renderers.
- The existing raster golden fingerprints remain unchanged.
- The complete local release gate passed, including Rust `1.90.0` feature
  checks, Clippy, rustdoc, RustSec, dependency/license policy, fuzz harnesses,
  all five Kani proofs, byte-identical package archives, required SBOM
  generation, and crates.io publish dry-run.

`1.2.0` must still complete independent pentest, GitHub validation, and
downstream `hashavatar-website` integration testing before tagging.
