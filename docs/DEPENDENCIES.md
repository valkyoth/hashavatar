# Dependency Policy

Alpha.1 keeps the runtime graph limited to identity derivation and cleanup:

- `sanitization` `2.0.1` supplies bounded secret containers and drop cleanup.
- `sanitization-crypto-interop` `2.0.1` supplies SHA-512 with reviewed hasher
  cleanup through upstream `sha2` hooks.
- `roxmltree` `0.21.1` is dev-only in `hashavatar-core` for parser-backed SVG
  tests and is also used by the separate fuzz package.

There is no runtime dependency on `image`, codecs, `rand`, Serde, JSON, web
frameworks, async runtimes, network clients, GPU libraries, or filesystem
helpers. Raster storage and geometry execution are first-party safe Rust.

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
verifier configuration and adds no runtime dependency. Future codecs belong in
`hashavatar-formats`; schema, heapless storage, and GPU support follow their
separate roadmap admission gates.

`scripts/validate-dependencies.sh` enforces the current narrow graph.
