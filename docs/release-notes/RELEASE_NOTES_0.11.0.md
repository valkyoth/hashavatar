# hashavatar 0.11.0

`0.11.0` polishes the visual layer model introduced in `0.10.0`.

## Changes

- Added explicit `AvatarKind::supports_face_layers()` support metadata.
- Documented that non-face families accept accessory/expression options but
  skip those layers deterministically.
- Updated SVG frame shapes so non-square shapes use clip paths, matching the
  raster renderer's transparent outside-frame masking behavior.
- Fixed malformed Paws-family SVG output where one toe-pad ellipse emitted a
  color value in the `ry` radius attribute instead of a numeric radius.
- Lowered glasses placement slightly for dog, robot, monster, ghost, wizard,
  and knight families.
- Tuned eyepatch, horns, bowtie, crown, hat, and headphones placement for
  families where those overlays were visibly off-center.
- Added tests for supported face-layer families, unsupported fallback families,
  SVG frame clipping, and SVG radius-attribute integrity.

## Compatibility

Baseline `AvatarOptions` rendering is unchanged. Styled SVG output for
non-square frame shapes now clips content to the selected frame shape; this is
an intentional visual polish change for style-aware SVG callers.
