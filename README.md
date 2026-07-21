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
The `2.0.0-alpha.3` source tree ports all 31 existing families, 13 backgrounds,
and five frame shapes onto one private validated Q16.16 scene. Canonical
straight-alpha RGBA8, caller-provided surfaces, complete SVG documents, and SVG
fragments all execute that same scene.

Alpha.3 has no codecs, image-library types, mutable rendering RNG, filesystem
API, CLI, server, or user-supplied SVG. Optional formats and service-oriented
packages remain later milestones.

## Release Status

The latest crates.io release is `1.3.0`, maintained on
[`release/1.3`](https://github.com/valkyoth/hashavatar/tree/release/1.3). The
`2.0.0-alpha.x` line uses named implementation-stop commits. Exact commit SHAs
are tested through GitHub and `hashavatar-website`; prereleases are neither
tagged nor uploaded to crates.io.

To test alpha.3 from a local checkout:

```toml
[dependencies]
hashavatar = { path = "../hashavatar" }
```

## Catalog Request

```rust
use hashavatar::{
    AvatarBackground, AvatarKind, AvatarRequest, AvatarShape, AvatarStyle,
};

let style = AvatarStyle::new(
    AvatarKind::Robot,
    AvatarBackground::Ocean,
    AvatarShape::Circle,
);
let prepared = AvatarRequest::with_namespace(
    256,
    256,
    0,
    b"tenant-a",
    b"website-v2-alpha3",
    b"user-123",
    style,
)?
.prepare()?;

let report = prepared.scene_report();
let rgba = prepared.render_rgba()?;
let svg = prepared.render_svg()?;

assert_eq!(rgba.dimensions(), (256, 256));
assert_eq!(rgba.pixels().len(), report.rgba_bytes());
assert!(svg.starts_with("<svg"));

# Ok::<(), hashavatar::CatError>(())
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
# Ok::<(), hashavatar::CatError>(())
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
# Ok::<(), hashavatar::CatError>(())
```

`AvatarRequest` borrows input only until `prepare()`. `PreparedAvatar` retains
the private scene, explicit style, public named trait samples, and validated
resource report, but not the raw identifier, tenant, style-version, or identity
digest. `AvatarKind::ALL`, `AvatarBackground::ALL`, and `AvatarShape::ALL`
provide the frozen catalog order; family capability declarations are available
through `AvatarKind::capabilities()` and `AVATAR_FAMILY_CAPABILITIES`.

## Contracts

- Dimensions are restricted to `64..=2048` per side.
- Identity input is restricted to 1024 bytes.
- Tenant and style-version components are restricted to 128 bytes each.
- Components are length-prefixed and SHA-512 domain separated.
- Catalog IDs and capability behavior follow the documented
  [catalog contract](docs/CATALOG_CONTRACT.md).
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

`SceneReport::rgba_bytes()` gives the exact returned raster allocation.
`estimated_pixel_tests()` gives a conservative CPU-work value for application
admission and concurrency policy. A caller serving untrusted traffic must
still enforce process-wide concurrency and rate limits.

## Crates

| Package | Alpha.3 role |
| --- | --- |
| `hashavatar` | Recommended thin facade and stable import path |
| `hashavatar-core` | `no_std + alloc` catalog, identity, scene, CPU RGBA8, and SVG core |

The private scene representation is not a general graphics API. Future format,
schema, heapless, and GPU packages will consume narrow reviewed boundaries
without making internal scene layout a compatibility promise.

## Security Boundaries

Hashavatar creates public visual artifacts; it is not an authentication or
password-hashing system. SHA-512 separates bounded namespace and identity
components but does not stop offline guessing of low-entropy identifiers.
Applications handling sensitive email addresses, usernames, or internal IDs
should pass a domain-separated keyed pseudonym rather than the original value.

`sanitization` guards temporary preimages and derived digest storage. Returned
RGBA bytes and SVG strings are caller-owned public output and are not wiped by
the crate. Allocation timing can reveal bounded input length, and render time
can reveal visual complexity. See [Security Controls](docs/SECURITY_CONTROLS.md)
for the exact guarantees and accepted limitations.

## Verification

```bash
scripts/checks.sh
scripts/check_kani.sh
cargo test --workspace --release
cargo run --example catalog_sheet
cargo run --example catalog_raster_sheet
```

The standard gate checks formatting, strict Clippy, debug and release KATs,
rustdoc, MSRV `1.90.0`, package metadata, dependency and license policy,
RustSec, fuzz harness compilation, source size, unsafe boundaries, and
production panic policy. CI additionally compiles `hashavatar-core` for WASM,
AArch64 Linux, and 32-bit x86 Linux.

## License

Licensed under either [Apache License 2.0](LICENSE-APACHE) or
[MIT](LICENSE-MIT), at your option.
