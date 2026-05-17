# hashavatar 0.9.0

`hashavatar` 0.9.0 adds a constrained `hashavatar-core` crate for deterministic
`no_std + alloc` planning while keeping rendering and encoders in the main
crate.

## Highlights

- Bumped `hashavatar` to `0.9.0`.
- Added `hashavatar-core` `0.9.0`.
- Added a `no_std + alloc` core boundary for:
  - validated avatar specs
  - bounded identity and namespace validation
  - SHA-512 identity hashing
  - optional BLAKE3 and XXH3-128 identity hashing
  - public option enums
  - structured render-plan construction
- Wired the main crate's render-plan path through `hashavatar-core`.
- Added no-default-feature checks for `hashavatar-core`.
- Added fuzz harness coverage for the core render-plan boundary.

## Compatibility

- Main `hashavatar` raster, SVG, and encode APIs keep their existing shape.
- Raster/SVG output is intended to stay stable from `0.8.0`.
- `hashavatar-core` does not render images or SVG strings. It provides
  deterministic planning primitives only.
- Applications that only need normal rendering should continue using
  `hashavatar`.

## Security And Quality

- `hashavatar-core` is `#![no_std]` and `#![forbid(unsafe_code)]`.
- `hashavatar-core` retains bounded dimensions and identity inputs.
- Identity digest equality remains constant-time.
- Temporary identity preimage buffers remain zeroized.
- `image`, `palette`, raster encoders, SVG allocation, `StdRng`, and service
  controls remain outside the core boundary.
- Fixed-point geometry remains future work before claiming bit-identical raster
  output across every architecture and compiler backend.
