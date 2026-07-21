# Migrating From 1.x To 2.0

Hashavatar 2.0 intentionally changes API and pixels. The final 1.x renderer is
preserved by `v1.3.0` and `release/1.3`; applications requiring exact 1.x
output should pin `hashavatar = "=1.3.0"` until they deliberately migrate.

Alpha.1 is not a full replacement for 1.3. It supports only a canonical Cat,
RGBA8, and SVG. It exists so applications can test the new ownership and
determinism boundary before catalog and format packages are ported.

## Alpha.1 Trial

Replace the 1.x builder or free function with one prepared request:

```rust
use hashavatar::CatRequest;

let prepared = CatRequest::with_namespace(
    256,
    256,
    0,
    b"tenant-a",
    b"visual-rollout-alpha1",
    b"user-123",
)?
.prepare()?;

let rgba = prepared.render_rgba()?;
let svg = prepared.render_svg()?;

# Ok::<(), hashavatar::CatError>(())
```

The request borrows identity bytes only during preparation. The prepared value
owns one validated private scene and can execute either output without
rehashing. Encoders, automatic family selection, accessories, expressions,
frames, and typed cache keys are not yet part of alpha.1.

## Rollout Rules

1. Keep production on pinned 1.3 while testing alpha source locally.
2. Use a new style-version namespace and cache namespace for 2.0 output.
3. Expect Cat pixels and SVG to differ from 1.x.
4. Enforce service concurrency from `SceneReport` and application memory
   policy.
5. Test each reviewed alpha tag in `hashavatar-website`.
6. Move production only after the required 2.0 package and compatibility
   contracts reach stable.

Do not reuse a 1.x cache key for 2.0 bytes. Alpha.1 does not yet expose the
final typed asset-key contract; keep trial output in an isolated cache.
