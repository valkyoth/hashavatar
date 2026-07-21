# hashavatar 2.0.0-alpha.2

`2.0.0-alpha.2` is a source-only implementation-stop milestone. It is not
tagged, published to crates.io, or API/pixel compatible with alpha.1.

## Canonical Renderer

- Added bounded rectangles, ellipses, lines, paths, even-odd/nonzero fill,
  fixed-step integer quadratic/cubic lowering, clips, opacity groups, linear
  gradients, and integer source-over compositing.
- Expanded private scene validation and public resource reporting for path
  points and stack depth.
- Kept fixed-point and scene layouts private and `no_std + alloc` compatible.

## Caller Contracts

- Added `RgbaSurfaceMut` for validated padded caller buffers and `render_into`.
- Added `PixelDigest`, bound to dimensions, visible rows, and a versioned pixel
  contract while excluding row padding.
- Streamed pixel digest input through clear-on-drop SHA-512 state without a
  second image-sized preimage allocation.
- Added SVG document/fragment options, escaped accessibility fields,
  deterministic caller prefixes, and streaming writer output.
- Added explicit pixel, digest/derivation, failure, SVG, and canonical execution
  specifications under `docs/`.

## Assurance

- Expanded known-answer, padded-surface, malformed-stack, SVG parser, writer
  failure, compositing, Kani, fuzz, and cross-platform checks.
- Retained first-party `unsafe` prohibition, panic policy, dependency policy,
  source-size limit, MSRV checks, and source-only prerelease policy.
- Bound release-plan loading to the repository-owned plan instead of accepting
  a user-controlled filesystem path.

Pentest the exact implementation-stop commit announced by the maintainer. The
permanent digest is added only after the external review passes.
