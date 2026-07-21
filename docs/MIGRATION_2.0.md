# Migrating From 1.x To 2.0

Hashavatar 2.0 intentionally changes API and pixels. The final 1.x renderer is
preserved by `v1.3.0` and `release/1.3`; applications requiring exact 1.x
output should pin `hashavatar = "=1.3.0"` until they deliberately migrate.

Alpha.5 ports the complete 1.3 style catalog to typed canonical composition and
adds the intended prepared-request, key, reusable raster, and established-
format boundaries. Pixels intentionally differ from 1.x, while family-aware
colors and defining silhouettes preserve the recognizable subject and visual
intent of each 1.3 family.

## Alpha.5 Trial

Replace the 1.x builder or free function with one prepared request:

```rust
use hashavatar::{
    AvatarBackground, AvatarIdentity, AvatarKind, AvatarRequest, AvatarShape,
    AvatarStyle,
};

let style = AvatarStyle::new(
    AvatarKind::Cat,
    AvatarBackground::Themed,
    AvatarShape::Square,
);
let identity = AvatarIdentity::with_namespace(
    b"tenant-a",
    b"visual-rollout-alpha5",
    b"user-123",
)?;
let prepared = AvatarRequest::builder(identity)
    .size(256, 256)
    .style(style)
    .prepare()?;

let rgba = prepared.render_rgba()?;
let svg = prepared.render_svg()?;

# Ok::<(), hashavatar::AvatarError>(())
```

The identity constructor hashes and releases raw input before request building.
The prepared value owns one validated private scene and can execute canonical
RGBA, SVG, or enabled format output without rehashing. It also exposes
`ResourceBudget`, `IdentityCacheKey`, and `AvatarAssetKey` from the same frozen
tuple. Explicit and automatic layered styles remain available through
`AvatarStyle`, `AccessoryStack`, `ResolvedStyle`, and `LayoutReport`.

The 1.x default build exposed WebP. Alpha.5 restores that user-facing default
through the separate formats crate. PNG/JPEG/GIF remain opt-in with the same
feature names; format and encoded-key types are new and are not 1.x key
compatible. `image::RgbaImage` is no longer part of Hashavatar's public API.

## Rollout Rules

1. Keep production on pinned 1.3 while testing alpha source locally.
2. Use a new style-version namespace and cache namespace for 2.0 output.
3. Expect all family pixels and SVG to differ from 1.x.
4. Enforce service concurrency from `SceneReport` and application memory
   policy.
5. Test each reviewed alpha implementation-stop commit in `hashavatar-website`.
6. Review complete 1.3-versus-2.0 raster and SVG contact sheets; pixel KATs
   prevent unreviewed drift after that visual acceptance point.
7. Move production only after the required 2.0 package and compatibility
   contracts reach stable.

Do not reuse a 1.x cache key for 2.0 bytes. Alpha.5 typed keys use new domains
and remain prerelease; keep trial output in an isolated cache. A semantic
encoded key binds settings but not exact dependency builds. Bind an
`EncoderBuildId` or hash encoded bytes when cross-deployment byte identity is
required.
