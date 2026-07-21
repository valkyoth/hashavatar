# Preparing For Hashavatar 2.0

Hashavatar 1.3 adds an optional future-shaped request workflow before the 2.0
renderer and workspace exist. Applications can migrate request construction,
style validation, metadata, resource accounting, cache keys, and output
ownership now while retaining the frozen 1.x pixels.

The established `AvatarBuilder` and free functions remain supported throughout
1.x. This guide does not announce deprecations in 1.3.

## Prepare One Immutable Request

Derive a bounded `AvatarIdentity`, construct an `AvatarRequest`, and call
`prepare()` once:

```rust
use hashavatar::prelude::*;

let namespace = AvatarNamespace::new("tenant-a", "v2")?;
let identity = AvatarIdentity::new_with_namespace(namespace, "user-123")?;
let prepared = AvatarRequest::builder(identity)
    .size(256, 256)
    .seed(0)
    .kind(AvatarKind::Robot)
    .background(AvatarBackground::Transparent)
    .accessory(AvatarAccessory::Glasses)
    .shape(AvatarShape::Circle)
    .prepare()?;

let key = prepared.avatar_asset_key();
let image = prepared.render()?;

# Ok::<(), Box<dyn std::error::Error>>(())
```

`AvatarRequest` owns a derived identity rather than the raw input bytes. Its
`Debug` implementation is redacted. Preparation freezes the validated spec,
requested and effective styles, family capabilities, resource estimate, cache
keys, and output methods into one tuple.

Existing builder code can adopt preparation with one call:

```rust
use hashavatar::prelude::*;

let prepared = AvatarBuilder::for_id("user-123")
    .namespace("tenant-a", "v2")
    .size(256, 256)
    .automatic_style()
    .prepare()?;

# Ok::<(), AvatarError>(())
```

That adapter preserves the legacy builder's skip-on-unsupported behavior.

## Choose Style Compatibility Explicitly

New `AvatarRequest` builders are strict by default. An explicit accessory or
expression that the selected family cannot render returns
`AvatarRequestError::Style` instead of becoming a silent no-op.

Use `.legacy_v1_compatibility()` only while reproducing established 1.x
behavior. `PreparedAvatar::resolved_style()` then exposes both the requested
and canonical effective styles, including whether a layer was ignored.
Automatic style selection remains total and reports that it was automatically
derived.

## Move Cache Logic To PreparedAvatar

Use the typed keys on `PreparedAvatar`:

- `identity_cache_key()` identifies the active hash mode and identity.
- `avatar_asset_key()` adds the complete effective render tuple.
- `encoded_asset_key()` adds the semantic encoder contract.
- `encoded_asset_key_for_build()` adds caller-supplied deployment identity.

Keep these nominal types until the final cache adapter. Converting every key to
`String` early removes Rust's protection against mixing cache layers.

## Make Output Ownership Explicit

`write_svg()` and `encode_to_writer()` write into caller-owned sinks and return
writer or codec failures. Partial output remains owned by the caller after an
error. The SVG adapter still builds a temporary `String`, and raster encoders
may allocate codec scratch buffers.

`render_into()` accepts a validated caller-owned RGBA8 surface with an explicit
stride:

```rust
use hashavatar::prelude::*;

let identity = AvatarIdentity::new("user-123")?;
let prepared = AvatarRequest::builder(identity).size(256, 256).prepare()?;
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

The 1.3 adapter still renders through one internal `RgbaImage`; it does not
claim zero-allocation rendering. `ResourceBudget` reports the known caller and
temporary RGBA bytes, but excludes format-dependent codec allocations.

## Exact 1.x Compatibility Decision

The repository freezes one complete request, style, asset key, RGBA digest,
and SVG digest for every 1.x family in
`tests/compatibility_corpus_v1.tsv`. Existing public helpers and the prepared
workflow must continue to match that corpus throughout 1.x.

Hashavatar 2.0 is intended to introduce a new renderer and may deliberately
change pixels. Applications that require exact 1.x output should pin the latest
1.x release rather than assuming 2.0 is visually identical:

```toml
[dependencies]
hashavatar = "=1.3.0"
```

A separate `compat-v1` package is not currently planned. It will be considered
only if downstream demand justifies maintaining and auditing two rendering
engines. This keeps the 2.0 default graph focused and avoids silently carrying
legacy internals into new deployments.

## Application Checklist

1. Derive tenant-isolated `AvatarIdentity` values at the application boundary.
2. Build strict `AvatarRequest` values for user-selected styles.
3. Use legacy mode only for requests whose current no-op behavior is required.
4. Read effective style, capabilities, and memory estimates from preparation.
5. Use typed prepared keys instead of assembling cache strings.
6. Own writer failures, partial output cleanup, concurrency, and rate limits.
7. Pin 1.3 if exact 1.x pixels remain an application requirement.
