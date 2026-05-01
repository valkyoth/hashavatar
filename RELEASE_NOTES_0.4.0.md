# hashavatar 0.4.0

`hashavatar` 0.4.0 expands the public avatar set, adds more export and background options, and keeps the crate asset-free and dependency-conscious.

## Added

- New avatar families: `planet`, `rocket`, `mushroom`, `cactus`, `frog`, `panda`, `cupcake`, `pizza`, `icecream`, `octopus`, and `knight`
- New background modes: `transparent`, `black`, `dark`, and `light`
- JPEG/JPG and GIF raster export formats
- CLI and demo support for the new families and background modes
- Golden visual regression coverage for the expanded avatar set

## Changed

- Improved identity-driven variation for `ghost`, `slime`, `wizard`, and `skull`
- Updated README examples and demo identities for the current public API
- Updated crate documentation metadata to point to docs.rs

## Security And Quality

- Removed a vulnerable transitive dependency path while keeping drawing code asset-free
- Added stricter input and dimension validation for public avatar endpoints
- Verified with `cargo check`, `cargo test`, `cargo clippy`, `cargo audit`, and `cargo deny check licenses`
