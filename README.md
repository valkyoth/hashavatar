<p align="center">
  <b>Secure deterministic procedural avatars for Rust.</b><br>
  Stable identity hashing, safe Rust rendering, in-memory WebP/SVG output, and release-gated security evidence.
</p>

<div align="center">
  <a href="https://docs.rs/hashavatar">Docs.rs</a>
  |
  <a href="docs/SECURITY_CONTROLS.md">Security Controls</a>
  |
  <a href="docs/STABILITY.md">Stability Policy</a>
  |
  <a href="docs/VERSION_PLAN.md">Roadmap</a>
  |
  <a href="SECURITY.md">Security</a>
</div>

<br>

<p align="center">
  <img src="https://raw.githubusercontent.com/valkyoth/hashavatar/main/.github/images/hashavatar.webp" alt="hashavatar Rust crate overview">
</p>

# hashavatar

`hashavatar` is a Rust crate for deterministic, procedural avatar generation. It is designed for services that need stable user or tenant avatars without bundled artwork, sprite sheets, external asset packs, or filesystem-side effects.

The crate starts conservative: validated avatar dimensions, bounded identity input, namespace-isolated hashing, safe Rust rendering, in-memory raster encoding, SVG string rendering, and a release process with dependency, audit, fuzz, package, SBOM, and reproducibility checks.

## Current Status

The current crate version is `1.1.1`.

Implemented now:

- Pure library crate; no bundled demo server and no CLI binary.
- Deterministic avatars derived from SHA-512 identity hashes by default.
- Optional BLAKE3 and XXH3-128 identity derivation behind explicit Cargo
  features.
- Public enum variant lists use single-source `ALL` slices and byte-to-variant
  helpers for deterministic option derivation.
- Visual layer options for accessories, accent palettes, expressions, and
  frame shapes through `AvatarStyleOptions`.
- Automatic style derivation uses distinct identity digest bytes for kind,
  background, accessory, color, expression, and shape.
- Namespace-aware identity derivation for tenant isolation and visual rollouts.
- Length-prefixed hash components to avoid delimiter ambiguity.
- Avatar families through `AvatarKind`: animals, characters, fantasy/sci-fi
  faces, playful objects, and symbols. Current labels are listed in the public
  option catalog below.
- Background modes through `AvatarBackground`: fixed, transparent, patterned,
  gradient, and star-field canvas treatments.
- Visual layers through `AvatarAccessory`, `AvatarColor`,
  `AvatarExpression`, and `AvatarShape`.
- In-memory `WebP` encoding through `AvatarOutputFormat`; `PNG`, `JPEG`, and
  single-frame `GIF` export are explicit opt-in features.
- Compact SVG string rendering.
- Fluent `AvatarBuilder` API for common render, encode, SVG, and cache-key
  workflows.
- Typed low-level errors plus unified `AvatarError` for high-level builder
  calls.
- Optional `serde` feature for public style enums only. `AvatarIdentity` is
  intentionally not serializable.
- Opaque identity cache keys through `AvatarIdentity::cache_key()` and
  `AvatarBuilder::cache_key()`.
- Private `AvatarSpec` fields so dimensions must pass construction-time validation.
- No public path-writing helpers; callers own their storage and filesystem boundary.
- `#![forbid(unsafe_code)]` in library code.
- Golden visual regression fingerprints.
- Isolated fuzz harness for avatar identities, families, backgrounds, SVG
  rendering, default WebP encoding, and feature-gated encoder paths.
- Local release gates for formatting, clippy, tests, docs, dependency policy, RustSec advisories, package contents, SBOM generation, reproducible build checks, and crates.io publish dry runs.

Planned or intentionally external:

- HTTP serving, rate limits, cache headers, security headers, observability, and abuse controls live in [`hashavatar-api`](https://github.com/valkyoth/hashavatar-api).
- Additional output formats such as AVIF or JPEG XL require dependency-policy review before admission.
- Larger identity inputs should be normalized or mapped by the application before calling this crate.

## Trust Dashboard

| Area | Status |
| --- | --- |
| License | `MIT OR Apache-2.0` |
| MSRV | Rust `1.90.0` |
| Crate shape | Library only |
| Runtime graph | `image`, `palette`, `rand`, `sanitization`, `sanitization-crypto-interop`, transitive `sha2`, `subtle`; optional `blake3`, `xxhash-rust`, `image/png`, `image/jpeg`, `image/gif` |
| Unsafe policy | `#![forbid(unsafe_code)]` |
| Filesystem policy | No public path-writing APIs |
| Dimension limits | `64..=2048` pixels per side |
| Identity limits | 1024 bytes per identity input |
| Namespace limits | 128 bytes per tenant/style-version component |
| Hashing posture | SHA-512 default with length-prefixed domain, namespace, style, and identity components; optional BLAKE3 and non-cryptographic XXH3-128 |
| SVG posture | Generated numeric markup only; caller input is not inserted into SVG fragments |
| Release evidence | fmt, clippy, tests, docs, deny, audit, fuzz harness compile, package check, SBOM, reproducibility |

Security-control details live in [docs/SECURITY_CONTROLS.md](docs/SECURITY_CONTROLS.md). Dependency policy lives in [docs/DEPENDENCIES.md](docs/DEPENDENCIES.md). Panic policy lives in [docs/PANIC_POLICY.md](docs/PANIC_POLICY.md). Stable API and rendering policy lives in [docs/STABILITY.md](docs/STABILITY.md).

Future version planning lives in [docs/VERSION_PLAN.md](docs/VERSION_PLAN.md).
`hashavatar` remains a single image-generation crate; low-level core planning
is kept internal unless a future release has a concrete image-generation reason
to split it.

## Rust Version Support

The minimum supported Rust version is Rust `1.90.0`. New deployments should
prefer the latest stable Rust; as of June 30, 2026, that is Rust `1.96.1`.

Compatibility evidence for `1.1.1`:

| Rust | Local Evidence |
| --- | --- |
| `1.90.0` | ✓ full release gate |
| `1.91.0` | ✓ `cargo check --features all-formats` |
| `1.92.0` | ✓ `cargo check --features all-formats` |
| `1.93.0` | ✓ `cargo check --features all-formats` |
| `1.94.0` | ✓ `cargo check --features all-formats` |
| `1.95.0` | ✓ `cargo check --features all-formats` |
| `1.96.0` | ✓ `cargo check --features all-formats` |
| `1.96.1` | ✓ `cargo check --features all-formats` |

Optional hash modes are mutually exclusive, so `hashavatar` cannot use a single
`--all-features` evidence run. The `1.90.0` MSRV was also checked with
`cargo check --features "blake3 all-formats"` and
`cargo check --features "xxh3 all-formats"`.

## Install

```toml
[dependencies]
hashavatar = "1.1.1"
```

Optional identity hash modes and extra raster encoders are disabled by default.
Hash modes are mutually exclusive, so enable at most one of `blake3` or `xxh3`:

```toml
[dependencies]
hashavatar = { version = "1.1.1", features = ["blake3"] }
```

Enable additional raster formats explicitly:

```toml
[dependencies]
hashavatar = { version = "1.1.1", features = ["png", "jpeg", "gif"] }
```

Or enable every optional raster encoder at once:

```toml
[dependencies]
hashavatar = { version = "1.1.1", features = ["all-formats"] }
```

Enable string serialization/deserialization for public style enums:

```toml
[dependencies]
hashavatar = { version = "1.1.1", features = ["serde"] }
```

Combine these as needed, for example `features = ["blake3", "png", "serde"]`.

For a local checkout:

```toml
[dependencies]
hashavatar = { path = "../hashavatar" }
```

The crate is dual-licensed:

```toml
license = "MIT OR Apache-2.0"
```

## Builder API

`AvatarBuilder` is the recommended entry point for application code. It keeps
the same validation and security boundaries as the lower-level functions while
avoiding long positional argument lists.

```rust
use hashavatar::prelude::*;

fn main() -> Result<(), AvatarError> {
    let svg = AvatarBuilder::for_id("user@example.com")
        .size(256, 256)
        .namespace("tenant-a", "v2")
        .kind(AvatarKind::Robot)
        .background(AvatarBackground::Transparent)
        .accessory(AvatarAccessory::Glasses)
        .shape(AvatarShape::Circle)
        .render_svg()?;

    println!("{svg}");
    Ok(())
}
```

Use `.automatic_style()` when you want kind, background, accessory, color,
expression, and shape to be derived from distinct identity digest bytes.

```rust
use hashavatar::prelude::*;

fn main() -> Result<(), AvatarError> {
    let bytes = AvatarBuilder::for_id("user@example.com")
        .size(256, 256)
        .automatic_style()
        .encode(AvatarOutputFormat::WebP)?;

    println!("{} bytes", bytes.len());
    Ok(())
}
```

For cache storage, prefer the opaque cache-key API over deriving keys from
internal digest bytes.

```rust
use hashavatar::prelude::*;

fn main() -> Result<(), AvatarError> {
    let cache_key = AvatarBuilder::for_id("user@example.com")
        .namespace("tenant-a", "v2")
        .cache_key()?;

    println!("{cache_key}");
    Ok(())
}
```

Cache keys are stable and display-safe, but they still allow correlation: the
same identity maps to the same key.

## Stable Contract

The `1.x` series keeps the crate's public API shape and documented rendering contract stable.
Patch releases should not intentionally change output for the same explicit
rendering tuple except for correctness or security fixes. Minor releases may
add opt-in features, output formats, avatar families, backgrounds, or visual
layer variants when they are documented and tested. Automatic style rendering
can change distribution when public enum `ALL` lists grow, so services that
need precise visual rollout control should use namespace `style_version`
values deliberately.

See [docs/STABILITY.md](docs/STABILITY.md) for the full semver and rendering
policy.

## Limits

| Limit | Value |
| --- | --- |
| Minimum width/height | `64` |
| Maximum width/height | `2048` |
| Maximum raster pixels | `4,194,304` |
| Maximum raw RGBA buffer | `16,777,216` bytes |
| Maximum identity input | `1024` bytes |
| Maximum namespace tenant | `128` bytes |
| Maximum namespace style version | `128` bytes |

These limits are enforced by constructors and render entry points. They are intended to make the safe path the normal path for public web endpoints.

`AvatarSpec::default()` is a fixed deterministic convenience value:
`256x256` with seed `1`. Public services should normally construct
`AvatarSpec` from validated request parameters with `AvatarSpec::new(...)`
rather than treating `Default` as a production policy or source of randomness.
The `seed` argument is a caller-controlled style variant mixed into the
identity-derived renderer RNG. Changing it deliberately produces a different
deterministic visual variant for the same identity; it is not a replacement for
identity hashing or namespace style-version rollouts.

## Public Option Catalog

All public option enums expose an `ALL` slice, `from_byte`, `as_str`,
`Display`, and `FromStr` support. With the optional `serde` feature enabled,
these enums serialize and deserialize as the same lowercase string labels.
Byte-to-variant mapping always indexes through `ALL`, so adding variants does
not require duplicated modulo constants in caller code.

| Enum | Controls | Values |
| --- | --- | --- |
| `AvatarKind` | Base avatar family | `cat`, `dog`, `robot`, `fox`, `alien`, `monster`, `ghost`, `slime`, `bird`, `wizard`, `skull`, `paws`, `planet`, `rocket`, `mushroom`, `cactus`, `frog`, `panda`, `cupcake`, `pizza`, `icecream`, `octopus`, `knight`, `bear`, `penguin`, `dragon`, `ninja`, `astronaut`, `diamond`, `coffee-cup`, `shield` |
| `AvatarBackground` | Canvas/background treatment | `themed`, `white`, `black`, `dark`, `light`, `transparent`, `polka-dot`, `striped`, `checkerboard`, `grid`, `sunrise`, `ocean`, `starry` |
| `AvatarAccessory` | Optional accessory layer | `none`, `glasses`, `hat`, `headphones`, `crown`, `bowtie`, `eyepatch`, `scarf`, `halo`, `horns` |
| `AvatarColor` | Optional accent palette | `default`, `neon-mint`, `pastel-pink`, `crimson`, `gold`, `deep-sea-blue` |
| `AvatarExpression` | Optional expression overlay | `default`, `happy`, `grumpy`, `surprised`, `sleepy`, `winking`, `cool`, `crying` |
| `AvatarShape` | Optional frame shape | `square`, `circle`, `squircle`, `hexagon`, `octagon` |
| `AvatarOutputFormat` | Raster encoding format | `webp`; optional `png`, `jpg`, and `gif` with matching Cargo features |

`AvatarOptions` is the stable baseline option type for callers that only need
`kind` and `background`. `AvatarStyleOptions` carries the full visual style
tuple: `kind`, `background`, `accessory`, `color`, `expression`, and `shape`.

Each style has one accessory slot. For example, a caller can request
`eyepatch` or `hat`, but not both in the same `AvatarStyleOptions`. Keeping one
slot avoids ambiguous draw order and collision rules; a future version can add
typed accessory slots if the project needs combinations such as headwear plus
facewear.

Accessories and expressions require face anchors. These families currently have
calibrated face-layer anchors: `cat`, `dog`, `robot`, `fox`, `alien`,
`monster`, `ghost`, `slime`, `bird`, `wizard`, `skull`, `frog`, `panda`,
`octopus`, `knight`, `bear`, `penguin`, `dragon`, `ninja`, and `astronaut`.
Non-face families such as `paws`, `planet`, `rocket`, `diamond`,
`coffee-cup`, and `shield` skip accessory/expression layers deterministically
instead of placing them at arbitrary canvas coordinates. Accent colors and
frame shapes are canvas-level layers and still apply.

Use `AvatarKind::supports_face_layers()` when mapping public endpoint query
parameters. If it returns `false`, requested accessories and expressions are
accepted but become deterministic no-ops for that family. This keeps automatic
style derivation total while avoiding awkward combinations such as an eyepatch
on a paw print or planet.

Suggested public endpoint query mapping:

| Query parameter | Rust type | Validation |
| --- | --- | --- |
| `kind` | `AvatarKind` | Parse with `FromStr`; reject unsupported labels. |
| `background` | `AvatarBackground` | Parse with `FromStr`; reject unsupported labels. |
| `accessory` | `AvatarAccessory` | Parse with `FromStr`; allow no-op fallback when `kind.supports_face_layers()` is `false`. |
| `color` | `AvatarColor` | Parse with `FromStr`; `default` keeps the family palette. |
| `expression` | `AvatarExpression` | Parse with `FromStr`; allow no-op fallback when `kind.supports_face_layers()` is `false`. |
| `shape` | `AvatarShape` | Parse with `FromStr`; applied as a raster mask and SVG clip path. |
| `format` | `AvatarOutputFormat` | Parse with `FromStr`; SVG uses the `render_avatar_svg_*` APIs. |
| `size` | `AvatarSpec` | Construct with `AvatarSpec::new`; reject invalid dimensions. |

Keep request parsing, rate limiting, authentication, and concurrency limits in
the API service. This crate intentionally only validates rendering inputs and
returns typed errors.

## Style Recipes

These are useful starting points for public APIs and examples:

| Use case | Suggested style |
| --- | --- |
| Stable classic avatars | `AvatarOptions::new(kind, background)` |
| Fully automatic variety | `render_avatar_auto_for_id` or `AvatarStyleOptions::from_identity` |
| Profile pictures with transparent backgrounds | `background = transparent`, `shape = circle` or `squircle` |
| Dense UI lists | `shape = square`, `background = themed`, `accessory = none` |
| Playful public profiles | One face accessory, one expression, one accent color, and a frame shape |
| Security-sensitive services | SHA-512 or BLAKE3 identity derivation, stable namespace, explicit concurrency limits |

For public query parameters, prefer parsing labels with `FromStr` and returning
a normal validation error for unknown labels. Do not silently map unsupported
strings to defaults; silent fallback makes cache keys and user expectations
harder to reason about.

## Example: Encode WebP

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

assert!(!bytes.is_empty());

# Ok::<(), Box<dyn std::error::Error>>(())
```

The returned bytes can be sent as an HTTP response, uploaded to object storage, written to a caller-selected path, or cached by a CDN.

## Example: Render SVG

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
assert!(svg.contains("alien avatar"));

# Ok::<(), Box<dyn std::error::Error>>(())
```

Use SVG when you need vector output, easy inspection, text storage, or post-processing by application code.

## Example: Namespaced Tenants

```rust
use hashavatar::{
    AvatarBackground, AvatarKind, AvatarNamespace, AvatarOptions, AvatarOutputFormat,
    AvatarSpec, encode_avatar_for_namespace,
};

let namespace = AvatarNamespace::new("customer-a", "v2")?;
let spec = AvatarSpec::new(256, 256, 0)?;

let bytes = encode_avatar_for_namespace(
    spec,
    namespace,
    "user-123",
    AvatarOutputFormat::WebP,
    AvatarOptions::new(AvatarKind::Cat, AvatarBackground::Themed),
)?;

assert!(!bytes.is_empty());

# Ok::<(), Box<dyn std::error::Error>>(())
```

Use namespaces when the same user identifier must not collide visually across tenants, products, or style-version rollouts.

## Example: Deterministic Options From Bytes

```rust
use hashavatar::{AvatarBackground, AvatarKind, AvatarOptions};

let digest_bytes = [42_u8, 199_u8];
let options = AvatarOptions::new(
    AvatarKind::from_byte(digest_bytes[0]),
    AvatarBackground::from_byte(digest_bytes[1]),
);

assert!(AvatarKind::ALL.contains(&options.kind));
assert!(AvatarBackground::ALL.contains(&options.background));
```

The `from_byte` helpers use each enum's `ALL` slice, so new public variants do
not require duplicated modulo constants in caller code.

## Example: Automatic Visual Layers

```rust
use hashavatar::{AvatarSpec, render_avatar_auto_for_id};

let spec = AvatarSpec::new(256, 256, 0)?;
let image = render_avatar_auto_for_id(spec, "layered@hashavatar.app")?;

assert_eq!(image.width(), 256);

# Ok::<(), Box<dyn std::error::Error>>(())
```

Automatic mode derives these top-level choices from distinct SHA-512 digest
bytes:

| Choice | Digest byte |
| --- | --- |
| `AvatarKind` | `AVATAR_STYLE_KIND_BYTE` |
| `AvatarBackground` | `AVATAR_STYLE_BACKGROUND_BYTE` |
| `AvatarAccessory` | `AVATAR_STYLE_ACCESSORY_BYTE` |
| `AvatarColor` | `AVATAR_STYLE_COLOR_BYTE` |
| `AvatarExpression` | `AVATAR_STYLE_EXPRESSION_BYTE` |
| `AvatarShape` | `AVATAR_STYLE_SHAPE_BYTE` |

## Example: Manual Visual Layers

```rust
use hashavatar::{
    AvatarAccessory, AvatarBackground, AvatarColor, AvatarExpression, AvatarKind,
    AvatarShape, AvatarSpec, AvatarStyleOptions, render_avatar_svg_style_for_id,
};

let spec = AvatarSpec::new(256, 256, 0)?;
let style = AvatarStyleOptions::new(
    AvatarKind::Robot,
    AvatarBackground::Themed,
    AvatarAccessory::Glasses,
    AvatarColor::Gold,
    AvatarExpression::Happy,
    AvatarShape::Circle,
);

let svg = render_avatar_svg_style_for_id(spec, "robot@hashavatar.app", style)?;

assert!(svg.contains("robot avatar"));
assert!(svg.contains("accessory-glasses"));

# Ok::<(), Box<dyn std::error::Error>>(())
```

Existing `AvatarOptions::new(kind, background)` callers keep the old baseline
visual behavior. Use `AvatarStyleOptions::from_options(options)` when you want
to pass legacy options through a style-aware API without enabling extra layers.
Accessories and expressions are rendered only for avatar families with
calibrated face anchors. Non-face families such as `paws`, `planet`, and
`rocket` skip those layers deterministically instead of drawing them in
misleading positions. Color and frame-shape layers still apply.

`AvatarStyleOptions` intentionally has one accessory field. Applications that
want multiple accessory concepts should model that at the product layer for now
and choose the single most important `AvatarAccessory` before calling this
crate.

## Identity Hash Mode

The default build uses SHA-512 identity derivation. Optional hash modes are
crate-wide Cargo feature choices, not runtime API choices:

- Default features: SHA-512.
- `blake3`: BLAKE3.
- `xxh3`: XXH3-128.

The `blake3` and `xxh3` features are mutually exclusive. Enabling both is a
compile-time error. Changing hash mode changes generated identities, so bump
your namespace style version when intentionally migrating output.

### BLAKE3 Feature Example

```toml
[dependencies]
hashavatar = { version = "1.1.1", features = ["blake3"] }
```

```rust
use hashavatar::{
    AvatarBackground, AvatarKind, AvatarNamespace, AvatarOptions, AvatarSpec,
    render_avatar_svg_for_namespace,
};

let namespace = AvatarNamespace::new("customer-a", "v3")?;
let spec = AvatarSpec::new(256, 256, 0)?;

let svg = render_avatar_svg_for_namespace(
    spec,
    namespace,
    "user-123",
    AvatarOptions::new(AvatarKind::Alien, AvatarBackground::Themed),
)?;

assert!(svg.contains("alien avatar"));

# Ok::<(), Box<dyn std::error::Error>>(())
```

### XXH3-128 Feature Example

```toml
[dependencies]
hashavatar = { version = "1.1.1", features = ["xxh3"] }
```

```rust
use hashavatar::{
    AvatarBackground, AvatarKind, AvatarNamespace, AvatarOptions,
    AvatarOutputFormat, AvatarSpec, encode_avatar_for_namespace,
};

let namespace = AvatarNamespace::new("public-demo", "v3")?;
let spec = AvatarSpec::new(256, 256, 0)?;

let bytes = encode_avatar_for_namespace(
    spec,
    namespace,
    "demo-user-123",
    AvatarOutputFormat::WebP,
    AvatarOptions::new(AvatarKind::Robot, AvatarBackground::Themed),
)?;

assert!(!bytes.is_empty());

# Ok::<(), Box<dyn std::error::Error>>(())
```

XXH3-128 is fast and useful for non-adversarial distribution, but it is not a
cryptographic hash. Keep SHA-512 or BLAKE3 for adversarial or user-controlled
identity inputs.

## Example: Raw Image Buffer

```rust
use hashavatar::{
    AvatarBackground, AvatarKind, AvatarOptions, AvatarSpec, render_avatar_for_id,
};

let spec = AvatarSpec::new(128, 128, 0)?;
let image = render_avatar_for_id(
    spec,
    "fox@hashavatar.app",
    AvatarOptions::new(AvatarKind::Fox, AvatarBackground::Themed),
)?;

assert_eq!(image.width(), 128);
assert_eq!(image.height(), 128);

# Ok::<(), Box<dyn std::error::Error>>(())
```

Use raw buffers when the caller wants to composite, inspect pixels, run custom encoding, or integrate with an existing image pipeline.

## Handling Untrusted Input

```rust
use hashavatar::{
    AvatarBackground, AvatarKind, AvatarOptions, AvatarOutputFormat, AvatarSpec,
    encode_avatar_for_id,
};

fn avatar_response_bytes(user_id: &str, requested_size: u32) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let spec = AvatarSpec::new(requested_size, requested_size, 0)?;
    let options = AvatarOptions::new(AvatarKind::Cat, AvatarBackground::Transparent);

    encode_avatar_for_id(spec, user_id, AvatarOutputFormat::WebP, options)
        .map_err(Into::into)
}
```

The crate rejects unsupported sizes and oversized identities. Applications
should still enforce their own routing, authentication, rate limiting, cache
policy, response headers, request body limits, and concurrency limits. A single
maximum-size raster render needs up to `MAX_AVATAR_RGBA_BYTES` raw RGBA bytes
before encoder overhead, so public services should bound simultaneous large
renders at the API layer.

### Caller-Owned Output Cleanup

Encode APIs sanitize internal temporary raster buffers after encoding, but the
returned `Vec<u8>` belongs to the caller. Render APIs return an `RgbaImage`
owned by the caller. High-assurance applications that treat avatar output as
sensitive should clear those buffers after use:

```rust
use hashavatar::{
    AvatarBackground, AvatarKind, AvatarOptions, AvatarOutputFormat, AvatarSpec,
    encode_avatar_for_id, render_avatar_for_id,
};
use sanitization::{sanitize_bytes, unsafe_wipe::volatile_sanitize_vec};

let spec = AvatarSpec::new(256, 256, 0)?;
let options = AvatarOptions::new(AvatarKind::Cat, AvatarBackground::Transparent);

let mut bytes = encode_avatar_for_id(
    spec,
    "sensitive-user-id",
    AvatarOutputFormat::WebP,
    options,
)?;
// Send, store, or otherwise consume `bytes`.
volatile_sanitize_vec(&mut bytes);

let mut image = render_avatar_for_id(spec, "sensitive-user-id", options)?;
// Composite, inspect, or encode `image`.
sanitize_bytes(image.as_mut());

# Ok::<(), Box<dyn std::error::Error>>(())
```

### Concurrent Render Limits

This crate bounds each individual render, not process-wide memory pressure.
Public services should combine `MAX_AVATAR_RGBA_BYTES` with their own memory
budget and reject or queue excess work. For example, a Tokio-based API can use
a semaphore around render work:

```rust
use std::sync::Arc;

use hashavatar::{
    AvatarBackground, AvatarKind, AvatarOptions, AvatarRenderResourceBudget, AvatarSpec,
    render_avatar_for_id,
};
use tokio::sync::Semaphore;

fn render_permits_for_budget(memory_budget_bytes: usize) -> usize {
    let max_spec = AvatarSpec::new(2048, 2048, 0).expect("maximum avatar spec is valid");
    AvatarRenderResourceBudget::max_concurrent_renders_for_memory_budget(
        max_spec,
        memory_budget_bytes,
    )
    .max(1)
}

async fn render_with_limit(
    semaphore: Arc<Semaphore>,
    id: &str,
    requested_size: u32,
) -> Result<image::RgbaImage, Box<dyn std::error::Error>> {
    let spec = AvatarSpec::new(requested_size, requested_size, 0)?;
    let budget = spec.render_resource_budget(1);
    if budget.raw_rgba_bytes_per_render() > memory_budget_bytes_for_one_request() {
        return Err("requested avatar exceeds per-request render budget".into());
    }

    let _permit = semaphore.acquire().await?;
    let options = AvatarOptions::new(AvatarKind::Cat, AvatarBackground::Transparent);

    Ok(render_avatar_for_id(spec, id, options)?)
}

fn memory_budget_bytes_for_one_request() -> usize {
    32 * 1024 * 1024
}
```

For async web servers, run CPU-heavy rendering on an appropriate blocking
worker pool when needed, and keep the semaphore at the service boundary so it
accounts for all concurrent requests.

## API Reference Summary

Important public entry points:

- `AvatarBuilder::for_id(id)` for fluent SVG, raster, encode, and cache-key workflows
- `AvatarError` for high-level builder APIs
- `AvatarSpec::new(width, height, seed) -> Result<AvatarSpec, AvatarSpecError>`
- `AvatarSpec::render_resource_budget(concurrent_renders) -> AvatarRenderResourceBudget`
- `AvatarRenderResourceBudget::max_concurrent_renders_for_memory_budget(spec, memory_budget_bytes) -> usize`
- `AvatarIdentity::new(input) -> Result<AvatarIdentity, AvatarIdentityError>`
- `AvatarIdentity::new_with_options(options, input) -> Result<AvatarIdentity, AvatarIdentityError>`
- `AvatarIdentity::cache_key() -> String`
- `AvatarIdentityOptions::new(namespace)`
- `AvatarNamespace::new(tenant, style_version) -> Result<AvatarNamespace, AvatarIdentityError>`
- `AvatarOptions::new(kind, background)`
- `AvatarKind::supports_face_layers()`
- `AvatarStyleOptions::new(kind, background, accessory, color, expression, shape)`
- `AvatarStyleOptions::summary() -> String`
- `AvatarStyleOptions::from_identity(identity)`
- `encode_avatar_for_id(...)`
- `encode_avatar_style_for_id(...)`
- `encode_avatar_auto_for_id(...)`
- `encode_avatar_for_namespace(...)`
- `render_avatar_for_id(...)`
- `render_avatar_style_for_id(...)`
- `render_avatar_auto_for_id(...)`
- `render_avatar_for_namespace(...)`
- `render_avatar_with_identity_options(...)`
- `render_avatar_svg_for_id(...)`
- `render_avatar_svg_style_for_id(...)`
- `render_avatar_svg_auto_for_id(...)`
- `render_avatar_svg_for_namespace(...)`
- `render_avatar_svg_with_identity_options(...)`

Lower-level identity-specific renderers are available for callers that want direct control over a specific avatar family.

## Output Formats

| Format | API value | Notes |
| --- | --- | --- |
| WebP | `AvatarOutputFormat::WebP` | Default encoder and recommended format for modern web delivery. |
| PNG | `AvatarOutputFormat::Png` | Optional `png` feature. Lossless and broadly compatible. |
| JPEG | `AvatarOutputFormat::Jpeg` | Optional `jpeg` feature. Transparent pixels are composited over white. |
| GIF | `AvatarOutputFormat::Gif` | Optional `gif` feature. Legacy-compatible single-frame output; the encoder performs internal quantization buffers that `hashavatar` cannot sanitize, so prefer WebP or PNG for high-assurance deployments. |
| SVG | `render_avatar_svg_*` | Returns a string rather than raster bytes. |

AVIF and JPEG XL are not exposed because they add dependency or encoder maturity tradeoffs that have not cleared the crate's dependency policy.

## Determinism

For a concrete crate release, the output is deterministic for the tuple:

```text
crate identity hash mode + namespace tenant + namespace style version + identity bytes + avatar kind + background + dimensions + seed
```

Within the `1.x` series, explicit output for that tuple should remain stable
except for documented correctness or security fixes. This makes the crate
suitable for stable CDN-backed avatar URLs and golden regression tests.
Namespace hashing uses length-prefixed components, so embedded separator bytes
cannot create tenant/style-version ambiguity. The default SHA-512 path keeps the
pre-0.7 identity preimage stable; optional crate-wide hash modes are
domain-separated.

For style-aware rendering, the deterministic tuple also includes
`accessory`, `color`, `expression`, and `shape`. Existing `AvatarOptions`
entry points keep those extra layer choices at `none`, `default`, `default`,
and `square`, so explicitly selected `AvatarOptions` output remains stable
within a major release unless a documented correctness or security fix requires
new output. Automatic style rendering derives choices through public `ALL`
variant lists, so adding variants in a future minor release can change
automatic distribution. Services that need controlled rollouts should keep
their existing namespace style version until they intentionally migrate. Some
family/layer combinations are deterministic no-ops when the layer has no
sensible anchor for that family.

Frame shapes are applied as masks in raster output and as SVG clip paths in SVG
output. Non-square shapes therefore trim the background, avatar body, color
accent, accessory layer, and expression layer consistently before drawing the
frame border.

The renderer still uses floating-point geometry in family-specific drawing
paths. Frame-shape raster hit-testing uses integer arithmetic as of `1.0.0`,
which reduces one source of platform rounding variance. The project tests
golden fingerprints on the release platform, but it does not yet claim formal
bit-identical raster output across every CPU architecture, compiler backend,
and optimization mode.

The procedural cat renderer seeds its internal RNG from bytes `32..64` of the
identity digest and uses the lower digest bytes for direct visual parameters.
That keeps RNG state separate from directly observed parameter bytes. The
change intentionally updates cat-family golden fingerprints in `0.7.0`.

`AvatarIdentity` equality uses constant-time digest comparison. Rendering and
encoding are not constant-time: shape counts, geometry, encoded size, and SVG
length can vary with identity digest bytes. Applications with strict side
channel requirements should not treat avatar render timing or output size as
secret-preserving signals.

When identity values are sensitive and an API must reduce render-time
observability, add the mitigation at the service boundary where request timing
is controlled:

```rust
use std::time::{Duration, Instant};

use hashavatar::{
    AvatarBackground, AvatarKind, AvatarOptions, AvatarSpec, render_avatar_for_id,
};

fn render_with_min_latency(
    id: &str,
    target_latency: Duration,
) -> Result<image::RgbaImage, Box<dyn std::error::Error>> {
    let started = Instant::now();
    let spec = AvatarSpec::new(256, 256, 0)?;
    let result = render_avatar_for_id(
        spec,
        id,
        AvatarOptions::new(AvatarKind::Monster, AvatarBackground::Themed),
    );

    let elapsed = started.elapsed();
    if elapsed < target_latency {
        std::thread::sleep(target_latency - elapsed);
    }

    Ok(result?)
}
```

For public web services, prefer CDN caching and stable cache keys so repeated
requests for the same avatar do not repeatedly expose renderer timing. In async
servers, use an async timer rather than blocking a runtime worker thread.

Encode APIs clear temporary raster buffers after encoding. Returned `Vec<u8>`
encoded bytes and `RgbaImage` render outputs are caller-owned; applications
with strict memory-sanitization requirements should clear those buffers after
use.

## Testing And Release Evidence

The repository includes:

- same-input stability tests
- different-input divergence tests
- raster export round-trip tests
- SVG safety and compactness tests
- enum parsing tests
- automatic visual layer derivation tests
- style-aware raster and SVG layer tests
- transparent background checks
- golden visual fingerprint tests
- fuzz harness compilation
- `cargo deny` policy
- RustSec advisory scanning
- reproducible package/build checks
- SBOM generation
- crates.io publish dry run

Run the standard local gate:

```bash
scripts/checks.sh
```

Run the fuller release gate:

```bash
scripts/stable_release_gate.sh check
```

## Provenance

The repository is intended to remain code-generated and asset-free. For a direct statement of how the visuals are produced, see [PROVENANCE.md](PROVENANCE.md).

## Web API And Demo

The crate is focused on reusable rendering code. The public HTTP API and demo website live in the separate [`hashavatar-api`](https://github.com/valkyoth/hashavatar-api) project.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) and [docs/release-notes](docs/release-notes)
for version-by-version details.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))
