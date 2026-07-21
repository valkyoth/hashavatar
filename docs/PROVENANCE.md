# Provenance

Hashavatar visuals are generated from first-party Rust geometry. The repository
does not bundle character illustrations, sprite sheets, icon packs, fonts,
templates, or external media used in avatar output.

Alpha.1 Cat geometry, colors, raster containment rules, and SVG serialization
are implemented in `hashavatar-core`. The repository image under
`.github/images/` is documentation artwork only and is excluded from package
archives.

Runtime dependencies provide secret-container cleanup and SHA-512 hashing;
they do not provide avatar art. `roxmltree` is test/fuzz-only. Dependency
licenses and notices are reviewed through `cargo-deny` and documented in
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

This is a technical provenance statement, not legal advice.
