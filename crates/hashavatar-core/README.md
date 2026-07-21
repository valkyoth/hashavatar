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
Hashavatar 2.0 work. Alpha.2 completes the bounded canonical renderer used by
the Cat vertical slice: validated fixed-point primitives and paths, exact
compositing, gradients, clips, opacity groups, caller surfaces, and SVG
documents/fragments.

Most applications should use the `hashavatar` facade. This companion crate is
not published during the 2.0 alpha, beta, or release-candidate cycle.

## Example

```rust
use hashavatar_core::CatRequest;

let prepared = CatRequest::new(256, 256, 0, b"user-123")?.prepare()?;
let traits = prepared.trait_vector();
let image = prepared.render_rgba()?;
let svg = prepared.render_svg()?;

assert_eq!(image.dimensions(), (256, 256));
assert_eq!(image.pixels().len(), 256 * 256 * 4);
assert!(svg.starts_with("<svg "));
assert_ne!(traits.head_width(), 0);
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
- Caller-provided strided surfaces preserve padding and share the owned-image
  executor.
- Pixel digests exclude padding and bind the dimensions and pixel contract ID.
- Returned pixels and SVG belong to the caller and are not secret containers.

Alpha.2 APIs and pixels remain explicitly unstable until the 2.0 beta freeze.
