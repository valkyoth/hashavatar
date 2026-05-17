# hashavatar 0.6.0

`hashavatar` 0.6.0 narrows the crate back to its core purpose: reusable deterministic avatar rendering.

## Highlights

- Removed the bundled Axum demo web server from the crate package
- Removed mandatory `axum` and `tokio` dependencies
- Removed the bundled `hashavatar-cli` binary so the package is a pure library crate
- Pointed web/API usage to the separate `hashavatar-api` project
- Added crate-focused security policy checks and release gates
- Added a fuzz harness for arbitrary avatar identities, families, backgrounds, SVG rendering, and PNG encoding
- Changed `AvatarSpec::new` to validate dimensions at construction and made spec fields private
- Added enforced identity and namespace byte-length limits with typed errors
- Changed public render APIs to return `Result<_, AvatarSpecError>` for invalid dimensions instead of panicking
- Removed public path-writing export helpers; callers should write encoded bytes or SVG strings through their own storage boundary
- Changed namespace identity hashing to length-prefix components, preventing separator ambiguity from embedded NUL bytes
- Hardened internal polygon and ellipse rasterization against edge-case panics and large-radius precision loss
- Added post-0.6 version planning for pluggable hashing, no-std preparation, visual layers, variant expansion, and 1.0 stabilization
- Documented maintenance rules for dependency freshness, security review, GitHub CodeQL default setup, and self-testing expectations

## Why This Changed

The public HTTP API and demo website already live in `hashavatar-api`. Keeping a second demo server inside the library crate made the package heavier and pulled web-server dependencies into users that only need avatar rendering.

## Compatibility

- This is a breaking API release for callers constructing `AvatarSpec`, constructing `AvatarIdentity`/`AvatarNamespace`, using direct render functions, using custom `AvatarRenderer` implementations, or relying on the removed path-writing export helpers.
- `AvatarSpec::new(...)` now returns `Result<AvatarSpec, AvatarSpecError>`.
- `AvatarIdentity::new(...)`, `AvatarIdentity::new_with_namespace(...)`, and `AvatarNamespace::new(...)` now return `Result`.
- Namespace-based identities intentionally produce new deterministic fingerprints because the hash input format was hardened.
- Existing deterministic fingerprints remain covered by updated golden regression tests.
- Users embedding the library should only see a smaller dependency graph.
- Users relying on `cargo run` for the bundled demo should use `hashavatar-api` instead.
- Users relying on `cargo run --bin hashavatar-cli` should call the library API directly or build a separate CLI wrapper.

## Security And Quality

- `src/lib.rs` now forbids unsafe code.
- `AvatarSpec` dimensions are validated before a spec value can be constructed through the public API.
- Identity inputs are capped at 1024 bytes, and namespace tenant/style-version components are capped at 128 bytes.
- The crate no longer writes to caller-provided filesystem paths.
- Public render APIs reject invalid dimensions without panicking.
- Namespace identity hashing is no longer delimiter-ambiguous when tenant or style version strings contain embedded NUL bytes.
- Rectangle helpers use saturating and clamping arithmetic.
- Polygon scanline rasterization skips incomplete intersection pairs instead of indexing blindly.
- Ellipse rasterization now uses `f64` intermediates for high-magnitude geometry calculations.
- `scripts/checks.sh` now validates release metadata, package contents, dependency scope, unsafe boundaries, reviewed panic-like sites, docs, fuzz harness compilation, dependency licenses, and RustSec advisories.
- `scripts/stable_release_gate.sh` adds publish dry-run, reproducibility, and optional SBOM generation for release validation.
