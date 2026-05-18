# hashavatar 0.13.0

`0.13.0` expands the background catalog and applies targeted determinism
hardening without adding new avatar families.

## Added

- Added seven `AvatarBackground` values:
  - `polka-dot`
  - `striped`
  - `checkerboard`
  - `grid`
  - `sunrise`
  - `ocean`
  - `starry`
- Implemented each new background in raster and SVG output.
- Added tests that prove the new raster backgrounds are distinct and that the
  new SVG backgrounds remain well-formed XML.

## Changed

- Raster frame-shape hit-testing now uses integer arithmetic for circle,
  squircle, hexagon, and octagon masks. This reduces one source of
  platform-specific floating-point rounding in clipped output.
- Automatic style derivation can now map some identities to different
  backgrounds because `AvatarBackground::ALL` contains more variants. Services
  that need old automatic output should keep their existing namespace
  `style_version` until they intentionally migrate.
- Documentation now avoids `cargo test --all-features` guidance because
  `blake3` and `xxh3` are intentionally mutually exclusive crate-wide identity
  hash modes.
- README fuzz wording now reflects default WebP coverage and feature-gated
  encoder paths instead of describing PNG as always available.

## Compatibility Notes

- Explicit `AvatarOptions` and `AvatarStyleOptions` selections remain
  deterministic for the selected kind, background, dimensions, seed, namespace,
  crate identity hash mode, and identity.
- The new background variants are public API and therefore affect automatic
  background distribution.
- No new avatar families were added in this release.
