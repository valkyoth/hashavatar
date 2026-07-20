# Stability Policy

`hashavatar` 1.0.0 is the first stable release of the public crate API and the
documented rendering contract.

## Public API

The crate follows Cargo semver for exported Rust API items:

- Removing or renaming public items requires a major version.
- Changing public function signatures, error types, constructor behavior, or
  feature names requires a major version unless the change is strictly
  additive and source-compatible.
- `AvatarKind`, `AvatarBackground`, `AvatarAccessory`, `AvatarColor`,
  `AvatarExpression`, `AvatarShape`, and `AvatarOutputFormat` are currently
  exhaustive Rust enums; they are not marked `#[non_exhaustive]`.
- Adding a variant to one of those enums is therefore a breaking API change and
  is not allowed in the remaining 1.x series. New variants require a major
  release or a new additive type that does not alter an existing enum.
- New optional Cargo features may be added in minor releases when they are
  disabled by default and documented.

## Rendering Contract

For a concrete crate release and explicitly selected options, the intended
stable rendering tuple is:

```text
crate identity hash mode + namespace tenant + namespace style version + identity bytes + avatar kind + background + accessory + color + expression + shape + dimensions + seed
```

Within a major release, patch releases should not intentionally change output
for the same tuple except to fix a correctness or security bug. Minor releases
may add source-compatible APIs and opt-in capabilities, but existing exhaustive
enum variant sets remain frozen for 1.x.

`AvatarOptions` keeps `accessory = none`, `color = default`,
`expression = default`, and `shape = square`. `AvatarStyleOptions` includes the
full visual-layer tuple.

Automatic style rendering derives options from public enum `ALL` lists. Those
lists and the equivalent `CatalogVersion::LEGACY_V1` manifests remain frozen
with their explicit IDs and weights for the rest of 1.x. The active renderer is
identified by `RenderContractId::LEGACY_V1`. A future mapping or rendering
contract must receive a new identifier instead of changing either legacy
contract in place. Services should continue using
`AvatarNamespace::new(tenant, style_version)` for controlled visual rollouts.

The typed cache-key hierarchy introduced in 1.2 is also contract-versioned:

- `IdentityCacheKey` covers the active identity hash mode and derived identity;
- `AvatarAssetKey` additionally covers catalog, renderer, dimensions, seed, and
  every effective style layer, canonicalizing legacy layers a family ignores;
- `SemanticEncodedAssetKey` from `encoded()` additionally covers the output
  format and fixed encoder settings as a semantic request key;
- `BuildEncodedAssetKey` from `encoded_for_build()` additionally binds a
  caller-supplied `EncoderBuildId` for deployment-specific byte caches.

The distinct encoded-key types prevent a semantic request key from being used
accidentally where a deployment-specific byte key is required.
Serialization necessarily erases that distinction. Integrations should retain
the nominal type through their cache API and convert to hexadecimal only inside
the final storage adapter.

Neither encoded-key method is a digest of actual output bytes. Content-addressed
storage must hash the bytes after encoding. Applications own the build-ID
policy and should include resolved codec versions, target, relevant build
flags, and application revision whenever those can affect output.

The older string `cache_key()` output remains frozen for existing 1.1 caches.
Typed keys are the preferred path for new application caches.

Legacy rendering skips unsupported face layers. Strict validation is additive:
`AvatarStyleOptions::validate_strict()` and `StrictAvatarBuilder` reject those
combinations without changing legacy output.

## Security And Resource Contract

- Public dimensions remain bounded by `AvatarSpec`.
- Identity and namespace input lengths remain bounded before hashing.
- Public rendering APIs return typed errors for invalid inputs instead of
  panicking.
- The crate remains a pure library crate with no HTTP server, CLI, filesystem
  writing API, async runtime, or network dependency.
- The default build keeps SHA-512 identity hashing and WebP encoding.
- Optional hash modes and extra raster encoders remain explicit Cargo feature
  choices.

## Known Residuals

The renderer still uses floating-point arithmetic in family-specific geometry.
Frame-shape hit-testing uses integer arithmetic, and golden fingerprints protect
the release-platform output, but the crate does not claim formal bit-identical
raster output across every CPU architecture and compiler backend.

Rendering time, SVG size, and encoded raster size are not constant-time with
respect to the identity digest. High-assurance services should use stable cache
keys, CDN caching, rate limits, concurrency limits, and fixed-minimum-latency
wrappers when their threat model requires it.
