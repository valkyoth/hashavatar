# Third-Party Notices

Hashavatar alpha.4 uses third-party Rust crates for sensitive-memory cleanup,
SHA-512 hashing, and development-only XML parsing. It does not copy source from
`imageproc`, external avatar generators, or art packs.

The complete resolved dependency and license inventory is generated from
`Cargo.lock` by `cargo deny`, `cargo audit`, and the pinned SBOM tool. Package
licenses remain their upstream licenses; Hashavatar first-party source is
licensed under `MIT OR Apache-2.0`.
