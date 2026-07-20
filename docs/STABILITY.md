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
lists remain frozen with their enums for the rest of 1.x. The 2.0 migration may
introduce explicitly versioned catalogs; services should continue using
`AvatarNamespace::new(tenant, style_version)` for controlled visual rollouts.

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
