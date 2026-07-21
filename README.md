<p align="center">
  <b>Secure deterministic procedural avatars for Rust.</b><br>
  Bounded identity hashing, asset-free rendering, WebP/SVG output, typed cache keys, and explicit resource controls.
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
  <a href="https://github.com/valkyoth/hashavatar/blob/main/SECURITY.md">Security</a>
</div>

<br>

<p align="center">
  <a href="https://github.com/valkyoth/hashavatar">
    <img src="https://raw.githubusercontent.com/valkyoth/hashavatar/main/.github/images/hashavatar.webp" alt="hashavatar Rust crate overview">
  </a>
</p>

# hashavatar

`hashavatar` generates deterministic avatars from bounded identity bytes. It
ships no character artwork, sprite sheets, templates, HTTP server, CLI, or
filesystem-writing API. Raster and SVG output are produced from Rust drawing
code and returned to the caller.

The normal path validates dimensions and identity lengths, derives a
namespace-isolated identity, resolves a complete style, exposes conservative
resource information, and binds output and cache keys to one prepared request.

The examples below describe the published `1.3.0` API. Development and support
state lives in [Current Status](https://github.com/valkyoth/hashavatar/blob/main/docs/CURRENT_STATUS.md).

## Start Here

| Need | Start with |
| --- | --- |
| New application workflow | `AvatarIdentity` + `AvatarRequest::builder(...).prepare()` |
| Existing concise integration | `AvatarBuilder::for_id(...)` |
| Automatic complete style | `.automatic_style()` |
| User-selected strict style | `AvatarRequestBuilder` or `.strict_style_validation()` |
| Caller-owned RGBA memory | `PreparedAvatar::render_into()` |
| SVG or encoded writer | `write_svg()` or `encode_to_writer()` |
| Stable cache routing | Typed identity, avatar, semantic, and build asset keys |
| Tenant isolation | `AvatarNamespace` |
| Public web endpoint | Crate validation plus application rate/concurrency limits |

New code should prefer a prepared request. `AvatarBuilder` remains supported
for established integrations and short examples.

## Install

Default SHA-512 identity derivation, WebP encoding, and SVG rendering:

```toml
[dependencies]
hashavatar = "1.3.0"
```

Enable additional raster encoders explicitly:

```toml
[dependencies]
hashavatar = { version = "1.3.0", features = ["png", "jpeg", "gif"] }
```

Enable BLAKE3 identity derivation:

```toml
[dependencies]
hashavatar = { version = "1.3.0", features = ["blake3"] }
```

`blake3` and `xxh3` are mutually exclusive crate-wide identity modes. Extra
raster encoders and `serde` support for public style enums are disabled by
default.

## Quick Start

Prepare one immutable request and use it for metadata, keys, and output:

```rust
use hashavatar::prelude::*;

let namespace = AvatarNamespace::new("tenant-a", "avatars-v2")?;
let identity = AvatarIdentity::new_with_namespace(namespace, "user-123")?;
let prepared = AvatarRequest::builder(identity)
    .size(256, 256)
    .kind(AvatarKind::Robot)
    .background(AvatarBackground::Transparent)
    .accessory(AvatarAccessory::Glasses)
    .color(AvatarColor::Gold)
    .expression(AvatarExpression::Happy)
    .shape(AvatarShape::Circle)
    .prepare()?;

let key = prepared.avatar_asset_key();
let image = prepared.render()?;
let svg = prepared.render_svg();
let webp = prepared.encode(AvatarOutputFormat::WebP)?;

assert_eq!(image.dimensions(), (256, 256));
assert!(svg.starts_with("<svg "));
assert!(!webp.is_empty());
assert!(!key.to_hex().is_empty());

# Ok::<(), Box<dyn std::error::Error>>(())
```

Preparation validates the specification, resolves style compatibility, and
freezes the identity/spec/style tuple used by layout reports, resource budgets,
cache keys, rendering, SVG, and encoding.

## Automatic Style

Derive family, background, accessory, accent color, expression, and frame from
distinct identity digest bytes:

```rust
use hashavatar::prelude::*;

let identity = AvatarIdentity::new("user@example.com")?;
let prepared = AvatarRequest::builder(identity)
    .size(256, 256)
    .automatic_style()
    .prepare()?;

let style = prepared.resolved_style();
let bytes = prepared.encode(AvatarOutputFormat::WebP)?;

assert!(style.is_automatically_derived());
assert!(!bytes.is_empty());

# Ok::<(), Box<dyn std::error::Error>>(())
```

Automatic mapping is deterministic. Changing the identity hash mode, tenant,
style version, size, seed, or style inputs deliberately changes the output and
its typed asset key.

## Explicit Style Validation

`AvatarRequest` rejects explicit accessories and expressions that a family
cannot render. This is the recommended boundary for user-selected styles:

```rust
use hashavatar::prelude::*;

let identity = AvatarIdentity::new("user@example.com")?;
let prepared = AvatarRequest::builder(identity)
    .kind(AvatarKind::Cat)
    .background(AvatarBackground::Ocean)
    .accessory(AvatarAccessory::Hat)
    .expression(AvatarExpression::Winking)
    .shape(AvatarShape::Squircle)
    .prepare()?;

assert!(prepared
    .layout_report()
    .family_capabilities()
    .supports_accessories());

# Ok::<(), Box<dyn std::error::Error>>(())
```

Legacy `AvatarBuilder` and free functions preserve their 1.x deterministic
skip behavior for unsupported face layers. Existing code can opt into strict
validation with `.strict_style_validation()` or migrate with `.prepare()`.
Use `.legacy_v1_compatibility()` on a new request only when reproducing old
output intentionally; `resolved_style()` reports any ignored layer.

## Concise Builder

For a short established workflow:

```rust
use hashavatar::prelude::*;

let svg = AvatarBuilder::for_id("user@example.com")
    .size(256, 256)
    .namespace("tenant-a", "avatars-v2")
    .kind(AvatarKind::Fox)
    .background(AvatarBackground::Starry)
    .shape(AvatarShape::Circle)
    .render_svg()?;

assert!(svg.contains("fox avatar"));

# Ok::<(), AvatarError>(())
```

The builder owns or borrows the supplied identifier until consumed. Cloning a
builder may create another identifier copy. For sensitive identifiers, prefer
a short-lived borrow from protected storage or pass a keyed pseudonym.

## Namespaces

Namespaces separate the same identifier across tenants, products, and visual
rollouts:

```rust
use hashavatar::prelude::*;

let a = AvatarIdentity::new_with_namespace(
    AvatarNamespace::new("customer-a", "v1")?,
    "user-123",
)?;
let b = AvatarIdentity::new_with_namespace(
    AvatarNamespace::new("customer-b", "v1")?,
    "user-123",
)?;

assert_ne!(a.cache_key(), b.cache_key());

# Ok::<(), Box<dyn std::error::Error>>(())
```

Namespace components are length-prefixed before hashing, so embedded separator
bytes cannot create tenant/style ambiguity. Treat `style_version` as an
application-controlled visual rollout identifier.

## Caller-Owned Output

Render into validated tightly packed or padded RGBA8 storage:

```rust
use hashavatar::prelude::*;

let identity = AvatarIdentity::new("user-123")?;
let prepared = AvatarRequest::builder(identity).size(128, 128).prepare()?;
let budget = prepared.resource_budget();
let mut pixels = vec![0_u8; budget.minimum_rgba8_surface_bytes()];
let mut surface = RasterSurfaceMut::new_rgba8(
    &mut pixels,
    prepared.spec().width(),
    prepared.spec().height(),
    budget.minimum_rgba8_stride(),
)?;

prepared.render_into(&mut surface)?;

# Ok::<(), Box<dyn std::error::Error>>(())
```

The 1.x adapter still uses one sanitized temporary `RgbaImage`; this API makes
destination ownership and stride explicit but is not a zero-allocation claim.
Caller padding is preserved. Dimension, stride, capacity, renderer-shape, and
row-count mismatches fail closed before success is reported.

Write SVG or encoded output into a caller-owned sink:

```rust
use hashavatar::prelude::*;

let identity = AvatarIdentity::new("user-123")?;
let prepared = AvatarRequest::builder(identity).automatic_style().prepare()?;
let mut svg = Vec::new();
let mut webp = Vec::new();

prepared.write_svg(&mut svg)?;
prepared.encode_to_writer(AvatarOutputFormat::WebP, &mut webp)?;

assert!(svg.starts_with(b"<svg "));
assert!(!webp.is_empty());

# Ok::<(), Box<dyn std::error::Error>>(())
```

Partial output remains caller-owned if a writer or codec fails. SVG currently
builds one temporary `String`, and codecs may allocate private scratch buffers.

## Typed Cache Keys

Keep key types distinct until the storage boundary:

```rust
use hashavatar::prelude::*;

let identity = AvatarIdentity::new("user-123")?;
let prepared = AvatarRequest::builder(identity)
    .size(256, 256)
    .automatic_style()
    .prepare()?;

let identity_key: IdentityCacheKey = prepared.identity_cache_key();
let avatar_key: AvatarAssetKey = prepared.avatar_asset_key();
let semantic_webp: SemanticEncodedAssetKey =
    prepared.encoded_asset_key(AvatarOutputFormat::WebP);
let deployment_webp: BuildEncodedAssetKey = prepared.encoded_asset_key_for_build(
    AvatarOutputFormat::WebP,
    EncoderBuildId::from_bytes([7_u8; 32]),
);

assert_ne!(identity_key.to_hex(), avatar_key.to_hex());
assert_ne!(semantic_webp.to_hex(), deployment_webp.to_hex());

# Ok::<(), Box<dyn std::error::Error>>(())
```

- `IdentityCacheKey` binds the active identity mode and identity.
- `AvatarAssetKey` adds the complete effective render tuple.
- `SemanticEncodedAssetKey` adds the fixed format contract.
- `BuildEncodedAssetKey` adds a caller-supplied deployment/encoder build ID.

These are cache-routing identifiers, not proofs that independently built
encoded bytes are identical. Hash actual output bytes for content-addressable
integrity. Keys are display-safe but correlatable and do not make guessable
identifiers anonymous.

## Visual Options

All public option enums provide `ALL`, `from_byte`, `as_str`, `Display`, and
`FromStr`. The optional `serde` feature serializes style enums with the same
lowercase labels.

| Type | Values |
| --- | --- |
| `AvatarKind` | `cat`, `dog`, `robot`, `fox`, `alien`, `monster`, `ghost`, `slime`, `bird`, `wizard`, `skull`, `paws`, `planet`, `rocket`, `mushroom`, `cactus`, `frog`, `panda`, `cupcake`, `pizza`, `icecream`, `octopus`, `knight`, `bear`, `penguin`, `dragon`, `ninja`, `astronaut`, `diamond`, `coffee-cup`, `shield` |
| `AvatarBackground` | `themed`, `white`, `black`, `dark`, `light`, `transparent`, `polka-dot`, `striped`, `checkerboard`, `grid`, `sunrise`, `ocean`, `starry` |
| `AvatarAccessory` | `none`, `glasses`, `hat`, `headphones`, `crown`, `bowtie`, `eyepatch`, `scarf`, `halo`, `horns` |
| `AvatarColor` | `default`, `neon-mint`, `pastel-pink`, `crimson`, `gold`, `deep-sea-blue` |
| `AvatarExpression` | `default`, `happy`, `grumpy`, `surprised`, `sleepy`, `winking`, `cool`, `crying` |
| `AvatarShape` | `square`, `circle`, `squircle`, `hexagon`, `octagon` |

One `AvatarStyleOptions` value has one accessory slot. Families without face
anchors reject explicit accessories/expressions in strict mode and skip them
deterministically in legacy mode. Query parsers should reject unknown labels
instead of silently mapping them to defaults.

## Output Formats

| Output | Feature | Notes |
| --- | --- | --- |
| WebP | built in | Default in-memory raster encoder |
| SVG | built in | Numeric generated markup returned as text or written to a sink |
| PNG | `png` | Optional lossless raster encoder |
| JPEG | `jpeg` | Optional; alpha is flattened over white |
| GIF | `gif` | Optional single-frame encoder with codec-owned quantization buffers |
| All optional raster formats | `all-formats` | Enables PNG, JPEG, and GIF |

AVIF is under 2.0 dependency and security review. JPEG XL is not currently
planned. The default graph does not include optional encoders.

## Identity Hash Modes

The default build uses SHA-512. Select at most one optional crate-wide mode:

| Mode | Cargo feature | Intended use |
| --- | --- | --- |
| SHA-512 | none | Default cryptographic identity derivation |
| BLAKE3 | `blake3` | Cryptographic identity derivation with upstream platform acceleration |
| XXH3-128 | `xxh3` | Fast non-adversarial visual distribution only |

XXH3-128 is not collision-resistant. Do not use it directly with
attacker-controlled or sensitive identifiers. Changing modes changes identities
and output; coordinate it with namespace `style_version` and cache invalidation.

For sensitive, guessable identifiers, map the raw identifier through a keyed
pseudonym at the application boundary:

```toml
[dependencies]
hashavatar = "1.3.0"
sanitization = "2.0.1"
sanitization-crypto-interop = { version = "2.0.1", features = ["blake3"] }
```

```rust
use hashavatar::{AvatarIdentity, AvatarIdentityError};
use sanitization::Secret;
use sanitization_crypto_interop::blake3::blake3_keyed_digest;

fn protected_identity(
    tenant_key: &Secret<[u8; 32]>,
    raw_identity: &[u8],
) -> Result<AvatarIdentity, AvatarIdentityError> {
    let pseudonym = Secret::new(
        tenant_key.with_secret(|key| blake3_keyed_digest(key, raw_identity)),
    );
    pseudonym.with_secret(|bytes| AvatarIdentity::new(bytes))
}
```

Keep tenant keys in an appropriate key-management boundary. The pseudonym is
still an avatar identifier, not an authentication token.

## Limits And Resource Policy

| Limit | Value |
| --- | --- |
| Width and height | `64..=2048` pixels |
| Maximum raster pixels | `4,194,304` |
| Maximum raw RGBA8 buffer | `16,777,216` bytes |
| Maximum identity input | `1024` bytes |
| Maximum namespace tenant | `128` bytes |
| Maximum namespace style version | `128` bytes |

Construct `AvatarSpec` from untrusted dimensions and propagate its typed error:

```rust
use hashavatar::AvatarSpec;

let spec = AvatarSpec::new(512, 512, 0)?;
let budget = spec.render_resource_budget(8);

assert_eq!(budget.raw_rgba_bytes_per_render(), 512 * 512 * 4);
assert_eq!(budget.raw_rgba_bytes_for_concurrent_renders(), 8 * 512 * 512 * 4);

# Ok::<(), Box<dyn std::error::Error>>(())
```

The crate bounds one request, not aggregate process memory or CPU. Services
must enforce body limits, dimensions, concurrency, rate limits, authentication,
cache policy, and response headers. Run CPU-heavy rendering away from async
runtime worker threads and account for codec overhead above the raw RGBA budget.

## Sensitive Output Cleanup

Internal identity preimages, digest owners, renderer seed copies, and temporary
crate-owned raster/encoder buffers use `sanitization` cleanup boundaries where
the dependency APIs permit it. Returned images, strings, encoded vectors, and
caller surfaces belong to the caller.

```rust
use hashavatar::prelude::*;
use sanitization::wipe;

let spec = AvatarSpec::new(256, 256, 0)?;
let options = AvatarOptions::new(AvatarKind::Cat, AvatarBackground::Transparent);
let mut bytes = hashavatar::encode_avatar_for_id(
    spec,
    "sensitive-user",
    AvatarOutputFormat::WebP,
    options,
)?;

// Send or otherwise consume the output.
wipe::vec(&mut bytes);

# Ok::<(), Box<dyn std::error::Error>>(())
```

Rendering and encoding are not constant-time. Geometry complexity, output size,
and duration may vary with identity-derived choices. Do not treat avatar timing
or output length as secret-preserving signals. See
[Security Controls](https://github.com/valkyoth/hashavatar/blob/main/docs/SECURITY_CONTROLS.md)
for accepted residuals and high-assurance service guidance.

## Determinism

For one concrete release, output is deterministic for the effective tuple:

```text
identity mode + namespace + identity bytes + catalog/render contract
+ dimensions + seed + family + background + accessory + color
+ expression + shape + output format
```

The 1.x corpus freezes one complete request, style, typed asset key, RGBA
digest, and SVG digest for every family. The current 1.x renderer still uses
floating-point family geometry and does not claim bit-identical raster output
across every architecture and compiler configuration. The 2.0 canonical
renderer is planned to replace that with fixed-point contracts.

Applications requiring exact 1.x pixels should pin the latest `1.3.x` release.
Read the [stability policy](https://github.com/valkyoth/hashavatar/blob/main/docs/STABILITY.md)
and [2.0 migration guide](https://github.com/valkyoth/hashavatar/blob/main/docs/MIGRATION_2.0.md)
before moving a persistent avatar or cache deployment across major versions.

## Features

| Feature | Default | Effect |
| --- | --- | --- |
| `blake3` | no | Replaces SHA-512 identity derivation with BLAKE3 |
| `xxh3` | no | Replaces SHA-512 with non-cryptographic XXH3-128 |
| `png` | no | Adds PNG output |
| `jpeg` | no | Adds JPEG output |
| `gif` | no | Adds single-frame GIF output |
| `all-formats` | no | Enables PNG, JPEG, and GIF |
| `serde` | no | String serde for public style enums |
| `kani` | no | Reserved verifier harness feature; no normal runtime API |
| `fuzzing` | no | Internal fuzz surface; rejected in ordinary release builds |

Do not use `--all-features`: `blake3` and `xxh3` are intentionally mutually
exclusive. The repository tests every valid hash/format combination.

## API Map

- `AvatarIdentity`, `AvatarNamespace`, and `AvatarIdentityOptions`: bounded,
  domain-separated identity derivation.
- `AvatarRequest`, `AvatarRequestBuilder`, and `PreparedAvatar`: recommended
  validated request workflow.
- `AvatarBuilder` and `StrictAvatarBuilder`: established concise workflow.
- `AvatarSpec`: dimensions, seed, and raw render resource budget.
- `AvatarStyleOptions`: family, background, accessory, color, expression, and
  shape.
- `ResolvedStyle`, `LayoutReport`, and `ResourceBudget`: prepared metadata and
  effective behavior.
- `RasterSurfaceMut`: checked caller-owned RGBA8 storage.
- `AvatarOutputFormat`: enabled raster encoder selection.
- `IdentityCacheKey`, `AvatarAssetKey`, `SemanticEncodedAssetKey`, and
  `BuildEncodedAssetKey`: typed cache layers.
- `AvatarError`, `AvatarRequestError`, `StrictAvatarError`,
  `RasterSurfaceError`, and lower-level typed errors: non-panicking untrusted
  input handling.

Complete signatures and per-item security notes are on
[docs.rs](https://docs.rs/hashavatar).

## Project Documentation

- [Current status](https://github.com/valkyoth/hashavatar/blob/main/docs/CURRENT_STATUS.md)
- [Security controls](https://github.com/valkyoth/hashavatar/blob/main/docs/SECURITY_CONTROLS.md)
- [Dependency policy](https://github.com/valkyoth/hashavatar/blob/main/docs/DEPENDENCIES.md)
- [Stability and versioning](https://github.com/valkyoth/hashavatar/blob/main/docs/STABILITY.md)
- [Panic policy](https://github.com/valkyoth/hashavatar/blob/main/docs/PANIC_POLICY.md)
- [Kani proof scope](https://github.com/valkyoth/hashavatar/blob/main/docs/KANI.md)
- [Release process](https://github.com/valkyoth/hashavatar/blob/main/docs/RELEASE.md)
- [2.0 migration](https://github.com/valkyoth/hashavatar/blob/main/docs/MIGRATION_2.0.md)
- [2.0 implementation plan](https://github.com/valkyoth/hashavatar/blob/main/docs/PLAN_TOWARDS_2.0.md)
- [Provenance](https://github.com/valkyoth/hashavatar/blob/main/docs/PROVENANCE.md)
- [Release notes](https://github.com/valkyoth/hashavatar/tree/main/release-notes)
- [Changelog](https://github.com/valkyoth/hashavatar/blob/main/CHANGELOG.md)

## Website

The public generator and integration reference live in
[`hashavatar-website`](https://github.com/valkyoth/hashavatar-website). HTTP
routing, response headers, caching, blocking-worker policy, rate limits, and
abuse controls remain application concerns.

## License

Licensed under either:

- [Apache License 2.0](https://github.com/valkyoth/hashavatar/blob/main/LICENSE-APACHE)
- [MIT License](https://github.com/valkyoth/hashavatar/blob/main/LICENSE-MIT)
