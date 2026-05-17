# hashavatar-core

`hashavatar-core` is the `no_std + alloc` deterministic core used by
`hashavatar`.

It provides bounded avatar specs, bounded identity and namespace validation,
domain-separated identity hashing, public option enums, and a structured
render-plan type. It does not provide raster buffers, image encoders, SVG
strings, filesystem APIs, HTTP APIs, random-number generators, or service
controls.

The main `hashavatar` crate layers raster and SVG rendering on top of this
core.
