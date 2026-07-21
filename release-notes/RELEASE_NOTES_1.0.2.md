# hashavatar 1.0.2

`1.0.2` is an additive ergonomics and documentation release for `hashavatar`.

## Added

- `AvatarBuilder` as a fluent high-level API for common SVG, raster, encoded
  output, and cache-key workflows.
- `AvatarError` as a unified high-level error type for builder-based code.
- `AvatarIdentity::cache_key()` and `AvatarBuilder::cache_key()` for stable
  opaque cache identifiers without exposing the raw identity digest.
- `hashavatar::prelude` with the common application-facing types.
- Optional `serde` feature for public style enums. `AvatarIdentity` remains
  intentionally non-serializable.
- `AvatarStyleOptions::summary()` and `Display` for human-readable UI/log
  labels.
- Runnable examples for builder-based SVG rendering and cache-key derivation.

## Security And Compatibility

- The builder stores validation failures and returns them as `Result` values;
  invalid dimensions or namespace components do not panic.
- `AvatarBuilder` uses a redacted `Debug` implementation so accidental builder
  logging does not print the raw identity input.
- Cache keys are derived by hashing the internal digest under a separate
  cache-key domain. They are display-safe and opaque, but still correlate equal
  identities.
- Serde support is limited to public style enums. The identity digest type does
  not implement `Serialize` or `Deserialize`.
- No avatar visual fingerprints were intentionally changed.

## Documentation

- Documented the `AvatarSpec::new(width, height, seed)` seed as a deterministic
  style variant, not a replacement for identity hashing.
- Updated README installation examples, builder guidance, cache-key guidance,
  dependency policy, security controls, and API summary for `1.0.2`.
