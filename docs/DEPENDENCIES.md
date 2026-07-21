# Dependency Policy

Alpha.5 keeps canonical rendering dependencies limited to identity derivation
and cleanup, while codec dependencies live behind the formats boundary:

- `sanitization` `2.0.1` supplies bounded secret containers and drop cleanup.
- `sanitization-crypto-interop` `2.0.1` supplies SHA-512 with reviewed hasher
  cleanup through upstream `sha2` hooks.
- `roxmltree` `0.21.1` is dev-only in `hashavatar-core` for parser-backed SVG
  tests and is also used by the separate fuzz package.
- `image` `0.25.10` is optional and direct only in `hashavatar-formats`.
  Default features are disabled; WebP, PNG, JPEG, and GIF activate only their
  corresponding image-rs feature and transitive codec.

`hashavatar-core` and the no-default facade have no runtime dependency on
`image`, codecs, `rand`, Serde, JSON, web frameworks, async runtimes, network
clients, GPU libraries, or filesystem helpers. Raster storage and geometry
execution are first-party safe Rust.

## Admission Rules

- Prefer the latest compatible stable dependency release.
- Verify maintenance state, docs, advisories, default features, transitive
  growth, MSRV, and license compatibility before admission.
- Keep optional capabilities in the package that owns their policy boundary.
- Do not add web or service infrastructure to reusable crates.
- Run `cargo update`, `cargo audit`, `cargo deny check`, and `cargo outdated`
  when available before stable releases.
- Add tests for every newly enabled dependency path.

The `hashavatar-core` default feature set is empty. The `kani` feature reserves
verifier configuration and adds no runtime dependency. `hashavatar-formats`
and the facade default to WebP only; every other established format is opt-in.
Schema, heapless storage, AVIF, and GPU support follow later admission gates.

`scripts/validate-dependencies.sh` and `scripts/check_format_features.sh`
enforce the current package and per-format graphs.
