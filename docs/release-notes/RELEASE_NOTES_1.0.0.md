# hashavatar 1.0.0

`1.0.0` is the first stable release of `hashavatar`.

This release does not add new avatar families, backgrounds, visual layers,
hash modes, output formats, or runtime dependencies. It freezes the public API
shape and documents the rendering stability contract built through the 0.x
series.

## Stable Contract

- Public Rust API changes now follow Cargo semver expectations.
- Explicit rendering output is intended to remain stable within the `1.x`
  series for the same active identity hash mode, namespace, identity, avatar
  options, dimensions, and seed, except for documented correctness or security
  fixes.
- Automatic style rendering remains deterministic, but future minor releases
  may change automatic distribution when public enum `ALL` lists gain new
  variants.
- Services that need deliberate visual rollouts should use
  `AvatarNamespace::new(tenant, style_version)` and bump `style_version` only
  when they are ready for a new visual distribution.

## Documentation

- Added `docs/STABILITY.md` with the stable API, rendering, security/resource,
  and residual-risk policies.
- Updated README guidance for the 1.0 stable contract.
- Added the stability policy to release metadata validation so it remains part
  of the published crate package.

## Security Posture

- The crate remains a pure library crate with no HTTP server, CLI, filesystem
  writing API, async runtime, or network dependency.
- Public dimensions, identity inputs, and namespace components remain bounded.
- Public render APIs return typed errors for invalid inputs instead of
  panicking.
- `AvatarIdentityError` keeps the rejected length available through structured
  accessors, but its display text no longer prints the exact rejected byte
  count.
- Temporary renderer seed and intermediate identity digest copies are guarded
  with `zeroize::Zeroizing` before their final required by-value copies.
- Starry raster backgrounds incorporate identity digest bytes in their
  deterministic local star-position generator.
- Internal identity digest byte access is defensive against future
  out-of-range renderer mistakes, and the avatar fuzz harness now samples the
  full supported dimension range.
- The default build remains SHA-512 identity hashing plus WebP encoding.
- Optional BLAKE3, XXH3-128, PNG, JPEG, and GIF support remain explicit Cargo
  features.

## Compatibility Notes

- No visual variants were added in this release.
- No intentional golden fingerprint changes were made for `1.0.0`.
- The known residuals documented in `docs/STABILITY.md` and
  `docs/SECURITY_CONTROLS.md` still apply: family-specific geometry uses some
  floating-point arithmetic, rendering is not constant-time, and service-level
  concurrency/rate limiting belongs in callers such as `hashavatar-api`.
