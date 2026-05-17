# hashavatar 0.9.0

`hashavatar` 0.9.0 keeps the project as a single image-generation crate.

## Highlights

- Bumped the crate to `0.9.0`.
- Kept the public package as `hashavatar` only.
- Removed the near-term separate-core-crate direction from the roadmap.
- Kept deterministic identity hashing, public options, raster rendering, SVG
  rendering, and encoders together in the main crate.

## Compatibility

- Raster/SVG output is intended to stay stable from `0.8.0`.
- Existing render, SVG, encode, identity, and optional hash APIs are unchanged.
- There is no `hashavatar-core` crate to publish for this release.

## Rationale

The project goal is avatar image generation. A separate public `no_std + alloc`
core crate would add release and API surface without serving that primary use
case. Lower-level planning boundaries can remain internal unless a future
image-generation feature needs them.
