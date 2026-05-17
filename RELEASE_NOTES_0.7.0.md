# hashavatar 0.7.0

`hashavatar` 0.7.0 starts the post-0.6 roadmap by making identity hashing explicit and extensible while keeping SHA-512 as the conservative default.

## Highlights

- Added `AvatarHashAlgorithm`
- Added `AvatarIdentityOptions`
- Kept SHA-512 as the default identity hash
- Preserved the default SHA-512 identity preimage for existing callers
- Added optional BLAKE3 identity derivation behind the `blake3` Cargo feature
- Added optional XXH3-128 identity derivation behind the `xxh3` Cargo feature
- Added domain separation for non-default hash algorithms
- Added render, encode, and SVG entry points that accept identity hash options
- Added feature-gated tests for BLAKE3 and XXH3-128 rendering

## Compatibility

- Existing callers that use `AvatarIdentity::new`, `AvatarIdentity::new_with_namespace`, `render_avatar_for_id`, `render_avatar_for_namespace`, `render_avatar_svg_for_id`, `render_avatar_svg_for_namespace`, `encode_avatar_for_id`, or `encode_avatar_for_namespace` continue to use SHA-512.
- Default SHA-512 rendering output is intended to remain stable from `0.6.0`.
- BLAKE3 and XXH3-128 output is intentionally different from SHA-512 output.
- `AvatarHashAlgorithm::Blake3` exists only when the `blake3` feature is enabled.
- `AvatarHashAlgorithm::Xxh3_128` exists only when the `xxh3` feature is enabled.

## Security And Quality

- SHA-512 remains the default for adversarial settings.
- BLAKE3 is provided by the upstream `blake3` crate and uses dependency-provided acceleration where available.
- XXH3-128 is non-cryptographic and should only be used for non-adversarial identity distribution.
- All hash input components remain length-prefixed.
- Non-default algorithms include an explicit algorithm domain component.
- Oversized identity and namespace inputs are rejected before hashing for every enabled algorithm.
- Tests cover parser round-trips, algorithm separation, optional feature paths, and oversized input rejection.
