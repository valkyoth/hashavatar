<p align="center">
  <b>Canonical deterministic procedural avatars for Rust.</b><br>
  Stateless identity traits, validated fixed-point scenes, bounded CPU rasterization, and SVG from one source of truth.
</p>

<div align="center">
  <a href="https://crates.io/crates/hashavatar">Crates.io</a>
  |
  <a href="https://docs.rs/hashavatar">Docs.rs</a>
  |
  <a href="https://github.com/valkyoth/hashavatar/blob/main/docs/CURRENT_STATUS.md">Current Status</a>
  |
  <a href="https://github.com/valkyoth/hashavatar/blob/main/docs/SECURITY_CONTROLS.md">Security Controls</a>
  |
  <a href="https://github.com/valkyoth/hashavatar/blob/main/docs/PLAN_TOWARDS_2.0.md">2.0 Plan</a>
  |
  <a href="https://github.com/valkyoth/hashavatar/security/policy">Security</a>
</div>

<br>

<p align="center">
  <a href="https://github.com/valkyoth/hashavatar">
    <img src="https://raw.githubusercontent.com/valkyoth/hashavatar/main/.github/images/hashavatar.webp" alt="hashavatar Rust crate overview">
  </a>
</p>

# hashavatar

`hashavatar` is the recommended facade for deterministic avatar generation.
The `2.0.0-alpha.5` source tree combines the portable canonical renderer with
an isolated established-format provider. All 31 families, 13 backgrounds, five
frames, typed palettes, expressions, and multi-accessory stacks compile to one
private validated Q16.16 scene used by CPU RGBA8 and SVG.

Lossless WebP is enabled by default. PNG, JPEG, and GIF are explicit features
owned by `hashavatar-formats`; `hashavatar-core` remains codec-free
`no_std + alloc`. Hashavatar has no filesystem API, CLI, server, async runtime,
user-supplied SVG, or HTTP policy.

## Release Status

The latest crates.io release is `1.3.0`, maintained on
[`release/1.3`](https://github.com/valkyoth/hashavatar/tree/release/1.3). The
`2.0.0-alpha.x` line uses named implementation-stop commits. Exact commit SHAs
are tested through GitHub and `hashavatar-website`; prereleases are neither
tagged nor uploaded to crates.io.

To test alpha.5 from a local checkout:

```toml
[dependencies]
hashavatar = { path = "../hashavatar" }
```

## Visual Compatibility

Hashavatar 2.0 uses a new fixed-point canonical renderer, so it does not promise
pixel equality with 1.3. It does preserve the existing 31 subjects, family-
appropriate default palettes, and their recognizable visual signatures. The
full raster and SVG catalogs are reviewed against the deployed 1.3 catalog;
ordered per-family fingerprints then prevent unreviewed drift within 2.0.

Applications that require exact 1.3 pixels should pin `hashavatar = "=1.3.0"`.
Applications trialing 2.0 must use a new style-version and cache namespace. See
the [migration guide](docs/MIGRATION_2.0.md) for the rollout contract.

## Prepared Request

```rust
use hashavatar::{
    AvatarBackground, AvatarIdentity, AvatarKind, AvatarRequest, AvatarShape,
    AvatarStyle,
};

let style = AvatarStyle::new(
    AvatarKind::Robot,
    AvatarBackground::Ocean,
    AvatarShape::Circle,
);
let identity = AvatarIdentity::with_namespace(
    b"tenant-a",
    b"website-v2-alpha5",
    b"user-123",
)?;
let prepared = AvatarRequest::builder(identity)
    .size(256, 256)
    .style(style)
    .prepare()?;

let report = prepared.scene_report();
let rgba = prepared.render_rgba()?;
let svg = prepared.render_svg()?;

assert_eq!(rgba.dimensions(), (256, 256));
assert_eq!(rgba.pixels().len(), report.rgba_bytes());
assert!(svg.starts_with("<svg"));

# Ok::<(), hashavatar::AvatarError>(())
```

Render into a padded caller-owned surface without modifying padding:

```rust
use hashavatar::{
    AvatarBackground, AvatarKind, AvatarRequest, AvatarShape, AvatarStyle,
    RgbaSurfaceMut,
};

let style = AvatarStyle::new(
    AvatarKind::Ghost,
    AvatarBackground::Transparent,
    AvatarShape::Squircle,
);
let prepared = AvatarRequest::new(128, 128, 0, b"user-123", style)?.prepare()?;
let stride = 128 * 4 + 16;
let mut storage = vec![0_u8; stride * 128];
let mut surface = RgbaSurfaceMut::new(&mut storage, 128, 128, stride)?;
prepared.render_into(&mut surface)?;
let digest = surface.pixel_digest()?;

assert_eq!(surface.visible_row_bytes(), 128 * 4);
assert_eq!(digest, prepared.render_rgba()?.pixel_digest()?);
# Ok::<(), hashavatar::AvatarError>(())
```

Embed multiple deterministic fragments safely by choosing distinct prefixes:

```rust
use hashavatar::{
    AvatarBackground, AvatarKind, AvatarRequest, AvatarShape, AvatarStyle,
    SvgOptions,
};

let style = AvatarStyle::new(
    AvatarKind::Planet,
    AvatarBackground::Starry,
    AvatarShape::Hexagon,
);
let prepared = AvatarRequest::new(128, 128, 0, b"user-123", style)?.prepare()?;
let fragment = prepared.render_svg_with(SvgOptions::fragment("profile-avatar")?)?;
assert!(fragment.starts_with("<g id=\"profile-avatar-scene\">"));
# Ok::<(), hashavatar::AvatarError>(())
```

## Layered Style

Explicit styles are strict by default. Unsupported face layers, duplicate
slots, and collisions return typed errors before scene execution:

```rust
use hashavatar::{
    AccessoryStack, AvatarAccessory, AvatarBackground, AvatarExpression,
    AvatarKind, AvatarPalette, AvatarRequest, AvatarShape, AvatarStyle,
};

let accessories = AccessoryStack::from_slice(&[
    AvatarAccessory::Hat,
    AvatarAccessory::Eyepatch,
    AvatarAccessory::Bowtie,
])?;
let style = AvatarStyle::new(
    AvatarKind::Cat,
    AvatarBackground::Sunrise,
    AvatarShape::Circle,
)
.with_palette(AvatarPalette::Gold)
.with_expression(AvatarExpression::Winking)
.with_accessories(accessories);
let prepared = AvatarRequest::new(256, 256, 0, b"user-123", style)?.prepare()?;

assert_eq!(prepared.resolved_style().accessories().len(), 3);
assert_eq!(prepared.layout_report().accessory_decision_count(), 3);
# Ok::<(), hashavatar::AvatarError>(())
```

`AvatarStyle::automatic(...)` derives optional layers from independent labeled
traits and applies a frozen fallback policy. Explicit callers can opt into the
same behavior with `StyleResolutionPolicy::AutomaticFallback`. Every adjusted,
substituted, or rejected request is visible through `LayoutReport`; no layer is
silently skipped. See the [layered style contract](docs/LAYERED_STYLE_CONTRACT.md).

`AvatarIdentity` hashes bounded input immediately, redacts `Debug`, and owns a
clear-on-drop digest. `PreparedAvatar` retains the private scene, requested and
resolved styles, layout decisions, resource budget, public named trait samples,
and public correlatable asset keys, but not the raw identifier, tenant,
style-version, or identity digest. `AvatarKind::ALL`,
`AvatarBackground::ALL`, and `AvatarShape::ALL`
provide the frozen catalog order; family capability declarations are available
through `AvatarKind::capabilities()` and `AVATAR_FAMILY_CAPABILITIES`.

## Encoded Formats

The default facade encodes lossless WebP without exposing image-library types:

```rust
use hashavatar::{
    AvatarBackground, AvatarKind, AvatarRequest, AvatarShape, AvatarStyle,
    formats::{AvatarOutputFormat, encode},
};

let style = AvatarStyle::new(
    AvatarKind::Dragon,
    AvatarBackground::Starry,
    AvatarShape::Octagon,
);
let prepared = AvatarRequest::new(256, 256, 0, b"user-123", style)?.prepare()?;
let encoded = encode(&prepared, AvatarOutputFormat::WebP)?;

assert_eq!(encoded.metadata().media_type(), "image/webp");
assert_eq!(encoded.metadata().encoded_len(), encoded.bytes().len());
# Ok::<(), Box<dyn std::error::Error>>(())
```

Feature selection is explicit:

```toml
[dependencies]
hashavatar = { path = "../hashavatar", default-features = false, features = ["png"] }
```

| Feature | Default | Contract |
| --- | --- | --- |
| `webp` | Yes | Lossless RGBA WebP |
| `png` | No | Lossless RGBA, best/adaptive settings |
| `jpeg` | No | Quality 92, alpha flattened over white |
| `gif` | No | Single-frame speed-1 palette quantization |
| `all-formats` | No | All four established formats |

`encode_to_writer` and `encode_to_writer_with_scratch` support streaming and
allocation reuse. Writer errors may leave a partial prefix. Semantic keys bind
the canonical asset and encoder settings; build keys additionally bind a
caller-supplied encoder build ID. See the
[format contract](docs/FORMAT_CONTRACT.md).

## Contracts

- Dimensions are restricted to `64..=2048` per side.
- Identity input is restricted to 1024 bytes.
- Tenant and style-version components are restricted to 128 bytes each.
- Components are length-prefixed and SHA-512 domain separated.
- Prepared requests bind identity, resolved style, dimensions, seed, catalog,
  render contract, resource budget, and public asset keys transactionally.
- Catalog IDs and capability behavior follow the documented
  [catalog contract](docs/CATALOG_CONTRACT.md).
- Accessory stacks contain at most four entries and are canonicalized by
  z-band, typed slot, and stable accessory ID.
- Explicit unsupported layers and collisions fail closed; automatic fallback
  is deterministic and completely reported.
- Palette roles, family anchors, transforms, and expression geometry use only
  integer or checked fixed-point arithmetic.
- Traits are sampled independently by stable labels; adding a trait does not
  consume mutable RNG state or shift existing traits.
- Geometry is private signed Q16.16 fixed point with checked construction.
- Scenes contain at most 64 commands, eight paths, 48 points per path, and
  eight levels each of clips and opacity groups.
- Scene validation runs before raster or SVG execution.
- All backgrounds and non-square frame clips are scene commands shared by both
  executors. Transparent output clears prior caller-surface pixels.
- External raster output is straight-alpha sRGB RGBA8. Owned output is tightly
  packed; caller surfaces may use validated padded strides.
- Source-over compositing, gradient interpolation, clipping, integer curve
  lowering, and pixel-center sampling are integer-only and specified.
- SVG numeric values are exact decimal representations of the same Q16.16
  scene values and are parser-tested as XML.
- First-party Rust code forbids `unsafe` and production panic-like paths.
- Disabled formats fail before canonical rendering or writer modification.
- Codec settings, alpha behavior, partial output, key semantics, scratch
  accounting, and cleanup limits are documented per format.

`SceneReport::rgba_bytes()` gives the exact returned raster allocation.
`estimated_pixel_tests()` gives a conservative CPU-work value for application
admission and concurrency policy. A caller serving untrusted traffic must
still enforce process-wide concurrency and rate limits.

## Crates

| Package | Alpha.5 role |
| --- | --- |
| `hashavatar` | Recommended facade, default WebP workflow, and common import path |
| `hashavatar-core` | `no_std + alloc` identity, catalog, keys, scene, CPU RGBA8, and SVG |
| `hashavatar-formats` | `std` writer/owned WebP, PNG, JPEG, and GIF encoding boundary |

The private scene representation is not a general graphics API. Future schema,
heapless, AVIF, and GPU packages will consume narrow reviewed boundaries
without making internal scene layout a compatibility promise.

## Security Boundaries

Hashavatar creates public visual artifacts; it is not an authentication or
password-hashing system. SHA-512 separates bounded namespace and identity
components but does not stop offline guessing of low-entropy identifiers.
Applications handling sensitive email addresses, usernames, or internal IDs
should pass a domain-separated keyed pseudonym rather than the original value.

`sanitization` guards temporary preimages, derived digest storage, reusable
Hashavatar-owned RGBA storage, failed owned-encoding buffers, and JPEG
conversion scratch. Successful RGBA, SVG, and encoded bytes are caller-owned
public output. Codec-owned buffers cannot be wiped by Hashavatar. Allocation
timing can reveal bounded input length, and render time can reveal visual
complexity. See [Security Controls](docs/SECURITY_CONTROLS.md) for exact
guarantees and accepted limitations.

## Verification

```bash
scripts/checks.sh
scripts/check_format_features.sh
scripts/check_kani.sh
cargo test --workspace --release
cargo run --example catalog_sheet
cargo run --example catalog_raster_sheet
cargo run --example layer_raster_sheet
cargo run --example encoded_webp
```

The raster examples write fixed-path PPM review sheets under
`target/visual-review/`. The layer sheet includes the baseline plus every
accessory and expression for each face-capable family.

The standard gate checks formatting, strict Clippy, debug and release KATs,
rustdoc, MSRV `1.90.0`, package metadata, dependency and license policy,
RustSec, fuzz harness compilation, source size, unsafe boundaries, and
production panic policy, format feature isolation, and codec round trips. CI
additionally compiles `hashavatar-core` for WASM,
AArch64 Linux, and 32-bit x86 Linux.

## License

Licensed under either [Apache License 2.0](LICENSE-APACHE) or
[MIT](LICENSE-MIT), at your option.
