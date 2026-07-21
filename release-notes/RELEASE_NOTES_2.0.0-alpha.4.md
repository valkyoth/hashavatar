# hashavatar 2.0.0-alpha.4

`2.0.0-alpha.4` is a source-only implementation-stop milestone. It is not
tagged or published to crates.io, and its API and pixels remain prerelease.

## Layered Style And Layout

- Adds a fixed-capacity four-entry `AccessoryStack` and typed accessory slots.
- Adds glasses, hat, headphones, crown, bowtie, eyepatch, scarf, halo, and
  horns as composable layers.
- Adds default, happy, grumpy, surprised, sleepy, winking, cool, and crying
  expressions.
- Adds default, neon mint, pastel pink, crimson, gold, and deep sea blue
  integer palettes with resolved semantic color roles.
- Adds calibrated integer anchors and transforms for every face-capable family.

## Deterministic Resolution

- Explicit styles reject unsupported families, duplicate slots, and exclusion
  collisions with typed errors.
- Automatic styles use a frozen fallback order and report every accepted,
  adjusted, substituted, or rejected layer.
- `ResolvedStyle` and `LayoutReport` expose the effective immutable result.
- Canonical ordering is independent of caller insertion order.

## Canonical Execution And Assurance

- Accessories and expressions emit bounded canonical scene primitives shared
  by CPU RGBA8 and SVG.
- Layer-free alpha.3 aggregate pixels remain unchanged.
- Complete family/layer matrices, capacity tests, permutation tests, collision
  fuzzing, parser-backed SVG checks, pixel-distinct catalog checks, a ninth
  Kani proof, and a layered raster corpus cover the new behavior.

Pentest the exact implementation-stop commit announced by the maintainer. The
permanent digest is added only after the external review passes.
