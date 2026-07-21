# hashavatar 2.0.0-alpha.3

`2.0.0-alpha.3` is a source-only implementation-stop milestone. It is not
tagged or published to crates.io, and its API and pixels remain prerelease.

## Existing Catalog Port

- Added explicit generic preparation through `AvatarRequest`, `AvatarStyle`,
  and `PreparedAvatar`.
- Ported all 31 existing families to semantic compilers over the private
  canonical scene.
- Ported all 13 backgrounds: themed, white, black, dark, light, transparent,
  polka-dot, striped, checkerboard, grid, sunrise, ocean, and starry.
- Ported square, circle, squircle, hexagon, and octagon frame shapes.
- Added immutable family capability declarations and frozen catalog IDs.

## Canonical Execution

- Added validated ellipse and path clipping beside rectangular clipping.
- Raster and SVG execute the same frame clip and family/background scene.
- Transparent rendering directly clears prior caller-surface pixels.
- Catalog trait samples are independently SHA-512-derived under family scopes;
  catalog growth does not consume mutable RNG state.

## Assurance

- Executes all 2,015 family/background/frame combinations in integration tests.
- Freezes an aggregate pixel fingerprint for every family.
- Checks every family at minimum, representative, and maximum dimensions.
- Extends all fuzz harnesses across catalog selections and adds an eighth Kani
  proof for bounded byte-to-catalog mapping.
- Includes SVG and dependency-free raster contact-sheet generators for visual
  review.

Pentest the exact implementation-stop commit announced by the maintainer. The
permanent digest is added only after the external review passes.
