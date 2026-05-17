# Version Plan

This plan describes the intended post-`0.6.0` direction for `hashavatar`.
Each version has a concrete finish line so the project can stop, review, and
release without letting exploratory work blur into the release criteria.

## Guiding Principles

- Keep the default path conservative and deterministic.
- Prefer explicit opt-in features over broad default dependency growth.
- Do not add custom unsafe SIMD code to this crate.
- Use vetted dependencies for optimized algorithms when they have a clear
  security and maintenance story.
- Keep web/API concerns in `hashavatar-api`; keep this crate focused on
  reusable avatar generation.
- Treat visual-output changes as compatibility events and document them in
  release notes.

## 0.7.0: Pluggable Identity Hashing

Goal: make identity hashing explicit and extensible while keeping SHA-512 as
the stable default.

### Scope

- Add an `AvatarHashAlgorithm` enum.
- Add an `AvatarIdentityOptions` or equivalent options type.
- Keep `Sha512` as the default hash algorithm.
- Add optional `BLAKE3` support behind an explicit Cargo feature.
- Add optional `XXH3-128` support behind an explicit Cargo feature.
- Include the selected algorithm in the length-prefixed hash domain input.
- Preserve existing default SHA-512 output unless the caller opts into another
  algorithm.
- Document that `XXH3-128` is non-cryptographic and intended only for
  non-adversarial identity distribution.

### Candidate API Shape

```rust
pub enum AvatarHashAlgorithm {
    Sha512,
    Blake3,
    Xxh3_128,
}

pub struct AvatarIdentityOptions {
    pub namespace: AvatarNamespace<'static>,
    pub algorithm: AvatarHashAlgorithm,
}
```

The exact lifetime/API shape can change during implementation. The important
contract is that algorithm choice is explicit, typed, and included in the
deterministic identity input.

### Dependency Policy

- `sha2` remains the default dependency.
- `blake3` is optional.
- `xxhash-rust` or another reviewed XXH3 provider is optional.
- Optional hash dependencies must pass `cargo deny`, `cargo audit`, license
  checks, and package-size review.
- Do not enable optional hash dependencies by default unless there is a clear
  project decision and release-note justification.

### SIMD Policy

- SIMD may be used only through admitted dependency implementations such as
  `blake3`.
- No custom AVX2, AVX-512, NEON, or wasm SIMD implementation is added to
  `hashavatar`.
- README/docs must explain that hardware acceleration, when available, comes
  from the selected hash dependency and platform support.

### Finish Line

`0.7.0` is done when:

- Default SHA-512 fingerprints are unchanged from `0.6.0`.
- BLAKE3 and XXH3 identity modes have dedicated tests.
- Oversized identity and namespace inputs are rejected for every algorithm.
- Algorithm separation tests prove the same identity does not collide across
  algorithm domains.
- README, release notes, dependency docs, and security controls document the
  default and optional hash algorithms.
- `scripts/stable_release_gate.sh check` passes.
- crates.io publish dry run passes.

## 0.8.0: Core Boundary Preparation

Goal: separate deterministic avatar decisions from encoding/integration
concerns so a future `no_std + alloc` core is realistic.

### Scope

- Identify the minimum deterministic core:
  - validated avatar specs
  - identity and namespace validation
  - hash algorithm selection
  - avatar genome derivation
  - geometry/layout primitives
  - color values
- Reduce direct coupling between that core and `image::RgbaImage`.
- Introduce an internal drawing target trait if it makes the core boundary
  cleaner.
- Keep the public crate behavior stable unless a breaking change is explicitly
  worth making before `1.0`.
- Keep raster encoding in the main crate, not in the future core.

### Non-Goals

- Do not promise `no_std` in `0.8.0`.
- Do not split the crate unless the internal boundary is already clean enough
  to maintain.
- Do not add custom SIMD code.

### Finish Line

`0.8.0` is done when:

- The code has a clear internal boundary between deterministic avatar planning
  and output encoding.
- Raster and SVG outputs still pass golden fingerprint and safety tests.
- The dependency graph is no larger than `0.7.0` unless explicitly justified.
- The README explains that `no_std` is planned but not yet a public contract.
- `docs/DEPENDENCIES.md` explains which dependencies block or belong outside
  a future core crate.
- `scripts/stable_release_gate.sh check` passes.
- crates.io publish dry run passes.

## 0.9.0: `hashavatar-core` Experiment

Goal: publish or prepare a constrained core API that can work in
`no_std + alloc` environments without image encoders.

### Scope

- Create one of:
  - a separate `hashavatar-core` crate, or
  - a feature-isolated internal module that can later become a crate.
- Target `no_std + alloc` for the core boundary.
- Keep SVG string generation only if `alloc` is available and the API remains
  clean.
- Keep raster encoding, `image`, and format-specific encoders in the main
  `hashavatar` crate.
- Provide structured output or drawing commands if that is cleaner than
  exposing image buffers from the core.

### Likely Dependency Changes

- Replace or isolate `palette` usage if it blocks `no_std`.
- Replace `StdRng` usage with a deterministic, small, reviewable internal
  generator or an admitted `no_std` RNG dependency.
- Keep cryptographic hashing behind dependency features that support the core
  target.

### Finish Line

`0.9.0` is done when:

- The core boundary builds without `std`.
- The main crate still provides ergonomic raster/SVG APIs on top of the core.
- Golden fingerprints prove the refactor did not accidentally change default
  output, or release notes clearly document intentional changes.
- `no_std + alloc` support is documented precisely, including what is not
  available.
- Fuzz harnesses cover the core identity and rendering-plan boundary.
- `scripts/stable_release_gate.sh check` passes.
- crates.io publish dry run passes for every crate intended to be published.

## 1.0.0: Stability Contract

Goal: freeze a professional, security-oriented public API and rendering
contract.

### Scope

- Decide whether `hashavatar-core` is public and versioned independently.
- Freeze default hash algorithm and default visual style version.
- Freeze error types and constructor behavior.
- Define a stable policy for when visual output may change.
- Define semver rules for adding avatar families, hash algorithms, and output
  formats.

### Finish Line

`1.0.0` is done when:

- The public API has no known avoidable breaking changes left.
- Security controls, dependency policy, panic policy, and release process are
  complete and current.
- README and docs.rs examples compile and reflect the final API.
- Release notes clearly state the stability contract.
- `scripts/stable_release_gate.sh release` passes.
- crates.io publish dry run passes.

## Deferred Until Evidence Exists

These are intentionally not committed to a specific version:

- Custom SIMD inside `hashavatar`.
- Async APIs in this crate.
- HTTP server or demo code in this crate.
- AVIF/JPEG XL output.
- Enabling BLAKE3 or XXH3 by default.
- Raising identity length limits.

Each item needs concrete use cases, dependency review, tests, and release-note
justification before admission.
