# Failure Contract

Public request, namespace, dimensions, scene structure, SVG options, surface
stride, buffer length, and request/surface dimension equality are validated
before rendering begins. These failures do not modify caller output.

Alpha.4 style preparation returns typed errors for accessory-stack capacity,
unsupported family layers, duplicate accessory slots, exclusion collisions,
unsupported expressions, and expression/face-layer collisions. These failures
occur before canonical scene execution. `AutomaticFallback` converts only the
documented compatibility cases into immutable report decisions; allocation,
numeric, surface, scene, and writer errors are never converted into fallback.

Alpha.5 format selection fails with `FormatDisabled` before rendering or
writer modification when its feature is absent. Canonical render/allocation
failure also occurs before codec output. Once an encoder writes, a writer or
codec error may leave an arbitrary partial image prefix; retry with a fresh
destination. `encode_to_writer_with_scratch` may leave caller-owned RGBA
scratch containing complete or partially modified public pixels. Codec-owned
temporary state is outside Hashavatar's cleanup control.

Once canonical raster execution begins, a later arithmetic or internal scene
error may leave visible bytes partially modified. Row padding is never part of
the renderer's write range. Retry with a fresh or intentionally reset surface.
The owned-image path discards its private allocation on failure.

`PreparedAvatar::write_svg` and the transitional `PreparedCat::write_svg`
validate scene and options before the first write. A
destination `fmt::Write` failure returns `AvatarError::SvgWrite` and may leave an
arbitrary valid prefix in the destination. Retry with a fresh destination.
Owned SVG output is capped at 64 KiB and uses fallible initial reservation;
capacity exhaustion returns an error.

The crate promises deterministic typed errors for admitted checks, not strong
exception safety after execution starts. A global allocator may abort rather
than report allocation failure depending on application configuration.
