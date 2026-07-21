# Third-Party Notices

Hashavatar alpha.5 uses third-party Rust crates for sensitive-memory cleanup,
SHA-512 hashing, established image encoding, and development-only XML parsing.
The optional `image` dependency is isolated in `hashavatar-formats`, has
default features disabled, and admits only the selected WebP, PNG, JPEG, and
GIF codec paths. Hashavatar does not copy source from `imageproc`, external
avatar generators, or art packs.

The complete resolved dependency and license inventory is generated from
`Cargo.lock` by `cargo deny`, `cargo audit`, and the pinned SBOM tool. Package
licenses remain their upstream licenses; Hashavatar first-party source is
licensed under `MIT OR Apache-2.0`.
