# hashavatar

`hashavatar` is a Rust crate for deterministic, procedural avatar generation.

It is designed as a code-only alternative to asset-pack-based avatar systems: the output is generated from an identity hash and drawn from geometric primitives rather than bundled sprites, licensed art packs, or pre-rendered character sheets.

## Features

- Deterministic avatars derived from `SHA-512`
- Multiple avatar families: `cat`, `dog`, `robot`, `fox`, `alien`, `monster`, `ghost`, `slime`, `bird`, `wizard`, `skull`, `paws`, `planet`, `rocket`, `mushroom`, `cactus`, `frog`, `panda`, `cupcake`, `pizza`, `icecream`, `octopus`, `knight`
- Multiple background modes: `themed`, `white`, `black`, `dark`, `light`, `transparent`
- In-memory encoding for `WebP`, `PNG`, `JPEG`, `GIF`, plus SVG string rendering
- Namespace-aware identity hashing for multi-tenant or versioned rollouts
- Public API suitable for web apps, services, and batch jobs
- Built-in dimension validation for internet-facing avatar endpoints
- Built-in identity and namespace length limits for untrusted inputs

## Why Use It

- No bundled avatar art assets
- Stable output for a given namespace and identity tuple
- Small modern default output through `WebP`
- Simple integration into Rust servers and applications
- Suitable for CDN-backed avatar URLs because output is deterministic

## Installation

Add the crate to your project:

```toml
[dependencies]
hashavatar = "0.6.0"
```

If you are using it from a local checkout:

```toml
[dependencies]
hashavatar = { path = "../hashavatar" }
```

## Core Concepts

The main types are:

- `AvatarSpec`: image dimensions and rendering seed
- `AvatarIdentity`: stable hash-backed identity derived from input bytes
- `AvatarNamespace`: stable tenant/style namespace for deterministic isolation
- `AvatarKind`: avatar family such as `Cat`, `Dog`, `Robot`, `Ghost`, `Planet`, or `Panda`
- `AvatarBackground`: background mode: `Themed`, `White`, `Black`, `Dark`, `Light`, or `Transparent`
- `AvatarOptions`: avatar family plus background mode
- `AvatarOutputFormat`: raster output format for encoded bytes

In most applications, you only need:

- `AvatarSpec`
- `AvatarOptions`
- `encode_avatar_for_id(...)`
- or `render_avatar_svg_for_id(...)`

For multi-tenant products or staged visual rollouts, also use:

- `AvatarNamespace`
- `encode_avatar_for_namespace(...)`
- `render_avatar_for_namespace(...)`
- `render_avatar_svg_for_namespace(...)`

## Basic Usage

### Encode To WebP

```rust
use hashavatar::{
    AvatarBackground, AvatarKind, AvatarOptions, AvatarOutputFormat, AvatarSpec,
    encode_avatar_for_id,
};

let spec = AvatarSpec::new(256, 256, 0)?;
let bytes = encode_avatar_for_id(
    spec,
    "robot@hashavatar.app",
    AvatarOutputFormat::WebP,
    AvatarOptions::new(AvatarKind::Robot, AvatarBackground::Transparent),
)?;

# Ok::<(), Box<dyn std::error::Error>>(())
```

This returns encoded image bytes ready to:

- write to disk
- send as an HTTP response
- upload to object storage

### Encode To PNG

```rust
use hashavatar::{
    AvatarBackground, AvatarKind, AvatarOptions, AvatarOutputFormat, AvatarSpec,
    encode_avatar_for_id,
};

let spec = AvatarSpec::new(256, 256, 0)?;
let png = encode_avatar_for_id(
    spec,
    "dog@hashavatar.app",
    AvatarOutputFormat::Png,
    AvatarOptions::new(AvatarKind::Dog, AvatarBackground::Themed),
)?;

# Ok::<(), Box<dyn std::error::Error>>(())
```

### Render To SVG

```rust
use hashavatar::{
    AvatarBackground, AvatarKind, AvatarOptions, AvatarSpec, render_avatar_svg_for_id,
};

let spec = AvatarSpec::new(256, 256, 0)?;
let svg = render_avatar_svg_for_id(
    spec,
    "alien@hashavatar.app",
    AvatarOptions::new(AvatarKind::Alien, AvatarBackground::Transparent),
)?;

assert!(svg.starts_with("<svg "));

# Ok::<(), Box<dyn std::error::Error>>(())
```

This is useful when you want:

- vector output
- text-based storage
- easier inspection or post-processing

### Render To An Image Buffer

If you want the raw image before encoding:

```rust
use hashavatar::{
    AvatarBackground, AvatarKind, AvatarOptions, AvatarSpec, render_avatar_for_id,
};

let spec = AvatarSpec::new(256, 256, 0)?;
let image = render_avatar_for_id(
    spec,
    "fox@hashavatar.app",
    AvatarOptions::new(AvatarKind::Fox, AvatarBackground::Themed),
)?;

assert_eq!(image.width(), 256);
assert_eq!(image.height(), 256);

# Ok::<(), Box<dyn std::error::Error>>(())
```

## Recommended Integration Patterns

### In A Rust Web App

The usual pattern is:

1. accept an identity string from a route or query parameter
2. choose `AvatarKind`, `AvatarBackground`, and output format
3. call `encode_avatar_for_id(...)`
4. return the bytes with the correct content type

Example:

```rust
use hashavatar::{
    AvatarBackground, AvatarKind, AvatarOptions, AvatarOutputFormat, AvatarSpec,
    encode_avatar_for_id,
};

fn generate_avatar_bytes(user_id: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let spec = AvatarSpec::new(256, 256, 0)?;
    encode_avatar_for_id(
        spec,
        user_id,
        AvatarOutputFormat::WebP,
        AvatarOptions::new(AvatarKind::Cat, AvatarBackground::Transparent),
    )
    .map_err(Into::into)
}
```

Use these content types:

- `image/webp`
- `image/png`
- `image/jpeg`
- `image/gif`
- `image/svg+xml`

### For Public Avatar URLs

For internet-facing avatar endpoints:

- use deterministic IDs
- validate requested size
- keep requested dimensions within the crate-supported `64..=2048` pixel range
- cache aggressively at the CDN edge
- treat the full request tuple as the cache key:
  - identity
  - avatar kind
  - background
  - format
  - size

### For Batch Jobs

Use the crate when:

- pre-generating avatars for users
- backfilling a media bucket
- generating static assets during a build or migration

## Choosing Settings

### Avatar Family

Use `AvatarKind` to select the visual family:

- `Cat`
- `Dog`
- `Robot`
- `Fox`
- `Alien`
- `Monster`
- `Ghost`
- `Slime`
- `Bird`
- `Wizard`
- `Skull`
- `Paws`
- `Planet`
- `Rocket`
- `Mushroom`
- `Cactus`
- `Frog`
- `Panda`
- `Cupcake`
- `Pizza`
- `Icecream`
- `Octopus`
- `Knight`

### Background Mode

Use `AvatarBackground` to control the canvas:

- `Themed`: stylized background chosen by the renderer
- `White`: pure white background for cleaner export and compositing
- `Black`: pure black background for dark surfaces
- `Dark`: softer charcoal background for dark UI previews
- `Light`: subtle off-white background for softer neutral output
- `Transparent`: fully transparent canvas for compositing onto another surface

### Output Format

Use `AvatarOutputFormat` for raster output:

- `WebP`: recommended default for modern web delivery
- `Png`: useful for compatibility or lossless workflows
- `Jpeg`: legacy-compatible export; transparent pixels are composited over white
- `Gif`: legacy-compatible single-frame export

AVIF and JPEG XL are not currently exposed because they introduce a larger encoder dependency tree or lack a stable first-party encoder in the crate's current image stack.

Use `render_avatar_svg_for_id(...)` when you need vector output.

## API Reference Summary

Important public entry points:

- `AvatarSpec::new(width, height, seed) -> Result<AvatarSpec, AvatarSpecError>`
- `AvatarIdentity::new(input) -> Result<AvatarIdentity, AvatarIdentityError>`
- `AvatarNamespace::new(tenant, style_version) -> Result<AvatarNamespace, AvatarIdentityError>`
- `AvatarOptions::new(kind, background)`
- `encode_avatar_for_id(...)`
- `render_avatar_for_id(...)`
- `render_avatar_svg_for_id(...)`

If you need lower-level control, the crate also exposes identity-specific renderer functions for certain families.

## Determinism And Stability

The crate is designed so that:

- the same identity and options produce the same output
- different identities produce different procedural results
- output remains suitable for regression testing

The test suite includes:

- same-input stability checks
- different-input divergence checks
- raster export round-trips
- enum parsing checks
- transparent background checks
- visual fingerprint regression tests

## What's New In 0.6.0

- Removed the bundled demo web server from the crate
- Removed mandatory `axum` and `tokio` dependencies from the crate package
- Removed the bundled `hashavatar-cli` binary so the package is a pure library crate
- Moved web/API usage to the separate [`hashavatar-api`](https://github.com/valkyoth/hashavatar-api) project
- Added crate-focused security policy checks and a fuzz harness for avatar inputs

## What's New In 0.5.0

- Starting with `0.5.0`, project licensing is dual `MIT OR Apache-2.0`
- Added separate `LICENSE-MIT` and `LICENSE-APACHE` files
- Removed the previous EUPL license files
- Added Fluxheim-style local and GitHub CI checks through `scripts/checks.sh`
- Pinned GitHub Actions to immutable commit SHAs for CodeQL-friendly workflow hardening
- Moved demo-server WebP rendering and encoding onto Tokio's blocking task pool
- Added defense-in-depth HTTP security headers to demo HTML, image, and error responses

## What's New In 0.4.2

- Moved public repository and homepage metadata to GitHub
- Added GitHub contributor, security, issue, pull request, and CI files
- Kept docs.rs as the canonical Rust API documentation URL

## What's New In 0.4.1

- Updated direct dependencies to current compatible releases
- Moved deterministic randomization from `rand` 0.9 to `rand` 0.10
- Moved SHA-2 hashing from `sha2` 0.10 to `sha2` 0.11

## What's New In 0.4.0

- Added `AvatarBackground::Transparent` for transparent raster and SVG output
- Added `AvatarBackground::Black`, `AvatarBackground::Dark`, and `AvatarBackground::Light`
- Added `JPEG` and `GIF` raster export formats
- Added new avatar families: `planet`, `rocket`, `mushroom`, `cactus`, `frog`, and `panda`
- Added new food and adventure families: `cupcake`, `pizza`, `icecream`, `octopus`, and `knight`
- Improved visual variation for the `ghost`, `slime`, `wizard`, and `skull` families
- Added stricter input and dimension validation for safer public endpoints
- Removed a vulnerable transitive dependency path while keeping raster drawing code asset-free
- Refreshed the demo presets around `@hashavatar.app` sample identities

## Provenance

The repository is intended to remain code-generated and asset-free.

For a direct statement of how the visuals are produced, see:

- [`PROVENANCE.md`](./PROVENANCE.md)

## Web API And Demo

The crate is focused on reusable rendering code. The public HTTP API and demo
website live in the separate
[`hashavatar-api`](https://github.com/valkyoth/hashavatar-api) project.

## Security Checks

The repository includes executable policy checks for release metadata, package
contents, dependency scope, unsafe code, reviewed panic-like sites, fuzz harness
compilation, dependency licenses, and RustSec advisories:

```bash
scripts/checks.sh
```

For more detail, see:

- [`docs/SECURITY_CONTROLS.md`](./docs/SECURITY_CONTROLS.md)
- [`docs/DEPENDENCIES.md`](./docs/DEPENDENCIES.md)
- [`docs/PANIC_POLICY.md`](./docs/PANIC_POLICY.md)
- [`docs/RELEASE.md`](./docs/RELEASE.md)

## License

Licensed under either of:

- Apache License, Version 2.0 ([`LICENSE-APACHE`](./LICENSE-APACHE))
- MIT license ([`LICENSE-MIT`](./LICENSE-MIT))
