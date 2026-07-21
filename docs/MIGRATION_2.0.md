# Migrating From 1.x To 2.0

Hashavatar 2.0 intentionally changes API and pixels. The final 1.x renderer is
preserved by `v1.3.0` and `release/1.3`; applications requiring exact 1.x
output should pin `hashavatar = "=1.3.0"` until they deliberately migrate.

Alpha.3 ports the complete 1.3 family, background, and frame catalog to the new
canonical renderer. Pixels intentionally differ from 1.x. Accessories,
expressions, palette layers, codecs, and final typed asset keys remain later
milestones.

## Alpha.3 Trial

Replace the 1.x builder or free function with one prepared request:

```rust
use hashavatar::{
    AvatarBackground, AvatarKind, AvatarRequest, AvatarShape, AvatarStyle,
};

let style = AvatarStyle::new(
    AvatarKind::Cat,
    AvatarBackground::Themed,
    AvatarShape::Square,
);
let prepared = AvatarRequest::with_namespace(
    256,
    256,
    0,
    b"tenant-a",
    b"visual-rollout-alpha3",
    b"user-123",
    style,
)?
.prepare()?;

let rgba = prepared.render_rgba()?;
let svg = prepared.render_svg()?;

# Ok::<(), hashavatar::CatError>(())
```

The request borrows identity bytes only during preparation. The prepared value
owns one validated private scene and can execute either output without
rehashing. Explicit families, backgrounds, and frames are available. Encoders,
automatic style selection, accessories, expressions, palette layers, and typed
asset keys are not yet part of alpha.3. Versioned pixel digests are available
for canonical output comparison and isolated trial caching.

## Rollout Rules

1. Keep production on pinned 1.3 while testing alpha source locally.
2. Use a new style-version namespace and cache namespace for 2.0 output.
3. Expect all family pixels and SVG to differ from 1.x.
4. Enforce service concurrency from `SceneReport` and application memory
   policy.
5. Test each reviewed alpha implementation-stop commit in `hashavatar-website`.
6. Move production only after the required 2.0 package and compatibility
   contracts reach stable.

Do not reuse a 1.x cache key for 2.0 bytes. Alpha.3 does not yet expose the
final typed asset-key contract; keep trial output in an isolated cache.
