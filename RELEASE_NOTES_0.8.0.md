# hashavatar 0.8.0

`hashavatar` 0.8.0 prepares the crate for future core-boundary work without
adding new runtime dependencies or promising `no_std` support yet.

## Highlights

- Bumped the crate to `0.8.0`.
- Added an internal render plan used by raster, SVG, and encode paths.
- Changed public enum `ALL` lists from manually sized arrays to slices.
- Added `from_byte` helpers for deterministic enum selection:
  - `AvatarHashAlgorithm::from_byte`
  - `AvatarKind::from_byte`
  - `AvatarBackground::from_byte`
  - `AvatarOutputFormat::from_byte`
- Added tests that protect public enum parser/display behavior.
- Documented which dependencies belong outside a future `no_std + alloc` core.

## Compatibility

- Avatar rendering output is intended to stay stable from `0.7.0`.
- Public enum `ALL` associated constants now have slice type
  `&'static [Self]` instead of fixed-size array types.
- Existing render, SVG, and encode entry points keep the same behavior.
- `no_std` is still only a future direction, not a supported public contract.

## Security And Quality

- The dependency graph remains no larger than `0.7.0`.
- Enum byte derivation uses `ALL.len()` rather than duplicated modulo counts.
- Tests cover parser/display round trips, documented enum label order, and
  byte-to-enum derivation.
- The internal render plan keeps deterministic avatar decisions separate from
  output encoding concerns.
