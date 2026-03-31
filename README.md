# hashavatar

`hashavatar` is a Rust crate for deterministic, procedural avatar generation.

It is designed as a code-only alternative to asset-pack-based avatar systems: the output is generated from an identity hash and drawn from geometric primitives rather than bundled sprites, licensed art packs, or pre-rendered character sheets.

## Features

- Deterministic avatars derived from `SHA-512`
- Multiple avatar families: `cat`, `dog`, `robot`, `fox`, `alien`
- Multiple background modes: `themed`, `white`
- Export paths for `WebP`, `PNG`, and `SVG`
- Public API suitable for web apps, services, CLIs, and batch jobs

## Why Use It

- No bundled avatar art assets
- Stable output for a given identity string
- Small modern default output through `WebP`
- Simple integration into Rust servers and applications
- Suitable for CDN-backed avatar URLs because output is deterministic

## Installation

Add the crate to your project:

```toml
[dependencies]
hashavatar = "0.1"
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
- `AvatarKind`: avatar family such as `Cat`, `Dog`, or `Robot`
- `AvatarBackground`: background mode such as `Themed` or `White`
- `AvatarOptions`: avatar family plus background mode
- `AvatarOutputFormat`: raster output format for encoded bytes

In most applications, you only need:

- `AvatarSpec`
- `AvatarOptions`
- `encode_avatar_for_id(...)`
- or `render_avatar_svg_for_id(...)`

## Basic Usage

### Encode To WebP

```rust
use hashavatar::{
    AvatarBackground, AvatarKind, AvatarOptions, AvatarOutputFormat, AvatarSpec,
    encode_avatar_for_id,
};

let bytes = encode_avatar_for_id(
    AvatarSpec::new(256, 256, 0),
    "alice@example.com",
    AvatarOutputFormat::WebP,
    AvatarOptions::new(AvatarKind::Robot, AvatarBackground::White),
)?;

# Ok::<(), image::ImageError>(())
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

let png = encode_avatar_for_id(
    AvatarSpec::new(256, 256, 0),
    "bob@example.com",
    AvatarOutputFormat::Png,
    AvatarOptions::new(AvatarKind::Dog, AvatarBackground::Themed),
)?;

# Ok::<(), image::ImageError>(())
```

### Render To SVG

```rust
use hashavatar::{
    AvatarBackground, AvatarKind, AvatarOptions, AvatarSpec, render_avatar_svg_for_id,
};

let svg = render_avatar_svg_for_id(
    AvatarSpec::new(256, 256, 0),
    "carol@example.com",
    AvatarOptions::new(AvatarKind::Alien, AvatarBackground::White),
);

assert!(svg.starts_with("<svg "));
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

let image = render_avatar_for_id(
    AvatarSpec::new(256, 256, 0),
    "dave@example.com",
    AvatarOptions::new(AvatarKind::Fox, AvatarBackground::Themed),
);

assert_eq!(image.width(), 256);
assert_eq!(image.height(), 256);
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

fn generate_avatar_bytes(user_id: &str) -> Result<Vec<u8>, image::ImageError> {
    encode_avatar_for_id(
        AvatarSpec::new(256, 256, 0),
        user_id,
        AvatarOutputFormat::WebP,
        AvatarOptions::new(AvatarKind::Cat, AvatarBackground::White),
    )
}
```

Use these content types:

- `image/webp`
- `image/png`
- `image/svg+xml`

### For Public Avatar URLs

For internet-facing avatar endpoints:

- use deterministic IDs
- validate requested size
- cap maximum size
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

The repository also contains a CLI exporter:

```bash
cargo run --bin hashavatar-cli -- --id alice@example.com --kind robot --background white --format svg --output alice.svg
```

Batch export from a newline-delimited file:

```bash
cargo run --bin hashavatar-cli -- --input ids.txt --out-dir exports --kind fox --format webp
```

## Choosing Settings

### Avatar Family

Use `AvatarKind` to select the visual family:

- `Cat`
- `Dog`
- `Robot`
- `Fox`
- `Alien`

### Background Mode

Use `AvatarBackground` to control the canvas:

- `Themed`: stylized background chosen by the renderer
- `White`: pure white background for cleaner export and compositing

### Output Format

Use `AvatarOutputFormat` for raster output:

- `WebP`: recommended default for modern web delivery
- `Png`: useful for compatibility or lossless workflows

Use `render_avatar_svg_for_id(...)` when you need vector output.

## API Reference Summary

Important public entry points:

- `AvatarSpec::new(width, height, seed)`
- `AvatarOptions::new(kind, background)`
- `encode_avatar_for_id(...)`
- `render_avatar_for_id(...)`
- `render_avatar_svg_for_id(...)`
- `export_avatar_svg_for_id(...)`

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
- visual fingerprint regression tests

## Provenance

The repository is intended to remain code-generated and asset-free.

For a direct statement of how the visuals are produced, see:

- [`PROVENANCE.md`](./PROVENANCE.md)

## Local Demo

This repository also contains a demo web app:

```bash
cargo run
```

Then open:

```text
http://127.0.0.1:3000
```

## License

Licensed under EUPL-1.2. See [`LICENSE`](./LICENSE).
