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
- Derive enum-based choices from a single variant list instead of hard-coded
  modulo counts.

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
- Normalize public enum variant lists and byte-to-variant helpers before new
  visual layers are added.
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
- Public enum derivation does not rely on duplicated magic counts such as
  `value % 23`; it uses `ALL.len()` or an equivalent single source of truth.
- Tests fail if public enum variant lists drift from parser/display behavior.
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

## 0.10.0: Visual Layer Model

Goal: expand avatar variety with explicit, deterministic visual layers before
the public API is frozen.

### Scope

- Add new option enums for visual layers:
  - `AvatarAccessory`
  - `AvatarColor`
  - `AvatarExpression`
  - `AvatarShape`
- Add a higher-level options type, for example `AvatarStyleOptions`, that can
  carry kind, background, accessory, color palette, expression, and frame
  shape together.
- Keep existing `AvatarOptions` working during the transition if practical.
- Define a deterministic derivation path for each layer from identity bytes
  when the caller chooses an automatic/default mode.
- Make every new enum parseable and displayable like the existing public
  enums.
- Derive automatic layer choices from each enum's variant list rather than
  hard-coded modulo counts.
- Document how layer choices affect the deterministic output tuple.

### Candidate API Shape

```rust
pub enum AvatarAccessory {
    None,
    Glasses,
    Hat,
    Headphones,
    Crown,
    Bowtie,
    Eyepatch,
    Scarf,
    Halo,
    Horns,
}

pub enum AvatarColor {
    Default,
    NeonMint,
    PastelPink,
    Crimson,
    Gold,
    DeepSeaBlue,
}

pub enum AvatarExpression {
    Default,
    Happy,
    Grumpy,
    Surprised,
    Sleepy,
    Winking,
    Cool,
    Crying,
}

pub enum AvatarShape {
    Square,
    Circle,
    Squircle,
    Hexagon,
    Octagon,
}
```

The exact enum names can change during design review. The important contract is
that visual layers are typed, deterministic, documented, and covered by golden
tests.

### Compatibility Policy

- Existing callers should either keep the old default visual behavior or get a
  clearly documented migration path.
- If default output changes, release notes must state that `0.10.0` is a
  visual-output compatibility release.
- The `style_version` guidance in `VERSIONING.md` must be updated so services
  can roll out layered avatars deliberately.

### Enum Derivation Policy

- Prefer small hand-maintained `ALL` lists or `all()` accessors as the single
  source of truth for public enum variants.
- Avoid `strum` or other enum-reflection dependencies unless the boilerplate
  becomes large enough to justify the additional dependency and audit surface.
- Any byte-to-variant helper should index through the variant list, for
  example `Self::ALL[(value as usize) % Self::ALL.len()]`, so adding a variant
  cannot silently leave it unreachable.

### Finish Line

`0.10.0` is done when:

- Every visual layer enum has `ALL`, `as_str`, `Display`, and `FromStr`.
- Every automatic public enum choice uses the enum's variant list instead of a
  duplicated literal count.
- Automatic layer derivation is deterministic and covered by tests.
- Manual layer selection is covered by tests.
- Raster and SVG renderers support all selected baseline layers.
- Golden fingerprints cover representative combinations of kind, background,
  accessory, color palette, expression, and frame shape.
- README includes examples for automatic and manual visual layers.
- Release notes clearly state whether default visuals changed.
- `scripts/stable_release_gate.sh check` passes.
- crates.io publish dry run passes.

## 0.11.0: Visual Layer Coverage And Polish

Goal: make the visual layer system feel complete across avatar families rather
than merely supported by the API.

### Scope

- Expand accessory rendering across all avatar families where the accessory
  makes visual sense.
- Define fallback behavior for combinations that do not make sense, such as
  accessories that conflict with a particular family shape.
- Refine color palettes so they are accessible and readable on all supported
  backgrounds.
- Refine expression support so the common expressions are visually distinct
  across face-like families.
- Ensure frame shapes clip or mask raster and SVG output consistently.
- Add documentation for recommended layer combinations and public endpoint
  query mapping.

### Non-Goals

- Do not add user-supplied SVG fragments, arbitrary colors, arbitrary paths,
  or external assets.
- Do not make the crate responsible for HTTP query parsing.
- Do not add an unbounded combinatorial golden test matrix; use representative
  coverage plus targeted unit tests.

### Finish Line

`0.11.0` is done when:

- Every avatar family has reviewed behavior for every visual layer.
- Unsupported or awkward layer/family combinations have deterministic,
  documented fallback behavior.
- Raster and SVG outputs are visually consistent for frame shapes.
- Tests cover fallback behavior and representative cross-family combinations.
- README and docs explain the full layer model.
- `hashavatar-api` has enough documented guidance to expose the new layers
  safely when that project is ready.
- `scripts/stable_release_gate.sh check` passes.
- crates.io publish dry run passes.

## 1.0.0: Stability Contract

Goal: freeze a professional, security-oriented public API and rendering
contract.

### Scope

- Decide whether `hashavatar-core` is public and versioned independently.
- Freeze default hash algorithm and default visual style version.
- Freeze the visual layer option model.
- Freeze error types and constructor behavior.
- Define a stable policy for when visual output may change.
- Define semver rules for adding avatar families, visual layers, hash
  algorithms, and output formats.

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
