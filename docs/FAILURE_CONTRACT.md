# Failure Contract

Public request, namespace, dimensions, scene structure, SVG options, surface
stride, buffer length, and request/surface dimension equality are validated
before rendering begins. These failures do not modify caller output.

Once canonical raster execution begins, a later arithmetic or internal scene
error may leave visible bytes partially modified. Row padding is never part of
the renderer's write range. Retry with a fresh or intentionally reset surface.
The owned-image path discards its private allocation on failure.

`PreparedCat::write_svg` validates scene and options before its first write. A
destination `fmt::Write` failure returns `CatError::SvgWrite` and may leave an
arbitrary valid prefix in the destination. Retry with a fresh destination.
Owned SVG output is capped at 64 KiB and uses fallible initial reservation;
capacity exhaustion returns an error.

The crate promises deterministic typed errors for admitted checks, not strong
exception safety after execution starts. A global allocator may abort rather
than report allocation failure depending on application configuration.
