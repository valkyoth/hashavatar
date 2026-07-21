<p align="center">
  <b>Canonical fixed-point rendering core for Hashavatar.</b><br>
  Bounded identity input, stateless trait derivation, one validated scene, and deterministic CPU/SVG execution.
</p>

<div align="center">
  <a href="https://github.com/valkyoth/hashavatar">Hashavatar</a>
  |
  <a href="https://docs.rs/hashavatar-core">Docs.rs</a>
  |
  <a href="https://github.com/valkyoth/hashavatar/blob/main/docs/CURRENT_STATUS.md">Current Status</a>
  |
  <a href="https://github.com/valkyoth/hashavatar/blob/main/docs/SECURITY_CONTROLS.md">Security Controls</a>
</div>

<br>

<p align="center">
  <a href="https://github.com/valkyoth/hashavatar">
    <img src="https://raw.githubusercontent.com/valkyoth/hashavatar/main/.github/images/hashavatar.webp" alt="hashavatar Rust crate overview">
  </a>
</p>

# hashavatar-core

`hashavatar-core` contains the canonical safe-Rust renderer used by the
Hashavatar 2.0 work. Alpha.4 adds bounded typed palettes, expressions, and
multi-accessory layout to all 31 existing families, 13 backgrounds, and five
frame shapes. Every layer compiles to validated fixed-point primitives and
paths shared by exact CPU compositing and SVG documents/fragments.

Most applications should use the `hashavatar` facade. This companion crate is
not published during the 2.0 alpha, beta, or release-candidate cycle.

## Example

```rust
use hashavatar_core::{
    AvatarAccessory, AvatarBackground, AvatarExpression, AvatarKind,
    AvatarPalette, AvatarRequest, AvatarShape, AvatarStyle,
};

let style = AvatarStyle::new(
    AvatarKind::Dragon,
    AvatarBackground::Starry,
    AvatarShape::Octagon,
)
.with_palette(AvatarPalette::DeepSeaBlue)
.with_expression(AvatarExpression::Happy)
.with_accessory(AvatarAccessory::Halo)?;
let prepared = AvatarRequest::new(256, 256, 0, b"user-123", style)?.prepare()?;
let image = prepared.render_rgba()?;
let svg = prepared.render_svg()?;

assert_eq!(image.dimensions(), (256, 256));
assert_eq!(image.pixels().len(), 256 * 256 * 4);
assert!(svg.starts_with("<svg "));
assert_eq!(prepared.style(), style);
# Ok::<(), hashavatar_core::CatError>(())
```

## Current Boundary

- `no_std` with `alloc`; no `image`, codec, filesystem, clock, entropy, thread,
  network, web, or async dependency.
- Input dimensions and identity/namespace lengths are bounded before work.
- Trait values are independently derived with labeled SHA-512 calls rather
  than a mutable RNG stream.
- Scene commands and fixed-point layouts are private and validated before
  execution.
- CPU RGBA8 and SVG consume the same immutable scene.
- Every built-in family, background, and frame compiles into that scene.
- Fixed-capacity typed accessories, expressions, and integer palette roles
  resolve before compilation with immutable decision reporting.
- Explicit incompatibilities fail closed; automatic substitutions and
  rejections are deterministic and visible through `LayoutReport`.
- Caller-provided strided surfaces preserve padding and share the owned-image
  executor.
- Pixel digests exclude padding and bind the dimensions and pixel contract ID.
- Returned pixels and SVG belong to the caller and are not secret containers.

Alpha.4 APIs and pixels remain explicitly unstable until the 2.0 beta freeze.
