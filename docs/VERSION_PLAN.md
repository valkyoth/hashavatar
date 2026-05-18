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
- Prefer latest compatible stable crate releases; check crates.io, docs.rs,
  upstream repositories, and RustSec advisories before adding or upgrading
  dependencies.
- Keep web/API concerns in `hashavatar-api`; keep this crate focused on
  reusable avatar generation.
- Treat self-testing as part of implementation, not a follow-up task.
- Keep GitHub CodeQL on the repository default setup unless there is a
  documented reason to move to advanced setup.
- Treat visual-output changes as compatibility events and document them in
  release notes.
- Derive enum-based choices from a single variant list instead of hard-coded
  modulo counts.

## 0.7.0: Pluggable Identity Hashing

Status: implemented in `0.7.0`.

Goal: make identity hashing explicit and extensible while keeping SHA-512 as
the stable default.

### Scope

- Keep SHA-512 as the default crate-wide identity hash mode.
- Add optional `BLAKE3` support behind an explicit Cargo feature.
- Add optional `XXH3-128` support behind an explicit Cargo feature.
- Keep optional hash modes mutually exclusive so feature unification cannot
  silently mix identity derivation algorithms.
- Include the active non-default hash mode in the length-prefixed hash domain
  input.
- Preserve existing default SHA-512 output unless the crate is built with a
  different hash-mode feature.
- Document that `XXH3-128` is non-cryptographic and intended only for
  non-adversarial identity distribution.

### API Shape

Hash choice is not a runtime public API. `AvatarIdentityOptions` carries
namespace configuration only; the active identity hash is selected for the
whole crate by Cargo features.

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
- Oversized identity and namespace inputs are rejected for the active hash mode.
- Optional hash-mode tests prove non-default modes add an explicit algorithm
  domain component.
- README, release notes, dependency docs, and security controls document the
  default and optional crate-wide hash modes.
- `scripts/stable_release_gate.sh check` passes.
- crates.io publish dry run passes.

## 0.8.0: Core Boundary Preparation

Status: implemented in `0.8.0`.

Goal: separate deterministic avatar decisions from encoding/integration
concerns internally so later visual-layer work can stay organized.

### Scope

- Identify the minimum deterministic core:
  - validated avatar specs
  - identity and namespace validation
  - crate-wide hash mode
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
- Keep raster encoding in the main crate.

### Non-Goals

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
- The README explains that `hashavatar` remains a focused image-generation
  crate.
- `docs/DEPENDENCIES.md` explains the single-crate dependency boundary.
- `scripts/stable_release_gate.sh check` passes.
- crates.io publish dry run passes.

### Implementation Notes

- Public enum variant lists use `ALL` slices rather than array constants with
  duplicated lengths.
- `from_byte` helpers derive variants from `ALL.len()` for avatar kinds,
  backgrounds, and output formats.
- Tests cover parser/display drift and byte-to-variant behavior for public
  enums.
- Raster and SVG rendering now share an internal render plan before output
  encoding.
- `0.9.0` later rejected a public core crate split because the project goal is
  avatar image generation.

## 0.9.0: Single-Crate Boundary Decision

Status: implemented in `0.9.0`.

Goal: keep `hashavatar` focused as one image-generation crate and avoid a
separate public core crate until there is a concrete image-generation use case
for that split.

### Scope

- Keep deterministic identity hashing, public options, raster rendering, SVG
  rendering, and encoders in the `hashavatar` crate.
- Keep lower-level planning helpers internal.
- Remove `hashavatar-core` / `no_std + alloc` from the near-term release plan.
- Preserve all 0.8.0 rendering output and security posture.

### Finish Line

`0.9.0` is done when:

- The published package is still only `hashavatar`.
- README and dependency docs describe the single-crate boundary.
- Release notes explain why the separate core crate was not adopted.
- Golden fingerprints prove the release did not accidentally change default
  output.
- `scripts/stable_release_gate.sh check` passes.

## 0.10.0: Visual Layer Model

Status: implemented in `0.10.0`.

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
- Reserve distinct identity digest offsets for top-level automatic choices so
  kind, background, accessory, palette, expression, and frame shape do not all
  depend on the same byte.
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

### Automatic Derivation Schedule

Automatic layer choices must use an explicit schedule of identity digest bytes
or domain-separated sub-derivations. The simple baseline is one distinct digest
byte per top-level choice:

```rust
let kind = AvatarKind::from_byte(identity.byte(0));
let background = AvatarBackground::from_byte(identity.byte(1));
let accessory = AvatarAccessory::from_byte(identity.byte(2));
let color = AvatarColor::from_byte(identity.byte(3));
let expression = AvatarExpression::from_byte(identity.byte(4));
let shape = AvatarShape::from_byte(identity.byte(5));
```

The exact offsets can change during implementation, but the schedule must be
documented and covered by tests. Reusing a byte for unrelated top-level traits
is avoided because it creates visible correlation, such as one background being
overrepresented with one accessory.

### Finish Line

`0.10.0` is done when:

- [x] Every visual layer enum has `ALL`, `as_str`, `Display`, and `FromStr`.
- [x] Every automatic public enum choice uses the enum's variant list instead of a
  duplicated literal count.
- [x] Top-level automatic traits use distinct identity digest offsets or explicit
  domain-separated derivations.
- [x] Tests prove changing the digest byte reserved for one top-level trait does
  not alter the other automatic trait selections.
- [x] Automatic layer derivation is deterministic and covered by tests.
- [x] Manual layer selection is covered by tests.
- [x] Raster and SVG renderers support all selected baseline layers.
- [x] Unsupported accessory/expression combinations skip deterministically
  instead of drawing in arbitrary positions.
- [x] Golden fingerprints cover representative combinations of kind, background,
  accessory, color palette, expression, and frame shape.
- [x] README includes examples for automatic and manual visual layers.
- [x] Release notes clearly state whether default visuals changed.
- [x] `scripts/stable_release_gate.sh check` passes.
- [x] crates.io publish dry run passes.

## 0.11.0: Visual Layer Coverage And Polish

Status: implemented in `0.11.0`.

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

## 0.12.0: Variant Expansion

Status: implemented in `0.12.0` for the accepted avatar-family set.
Candidate background variants remain deferred.

Goal: broaden the built-in avatar and background catalog once the visual layer
model is in place and tested.

### Scope

- Add new `AvatarKind` families where they can be rendered with the same
  deterministic, asset-free approach as the existing variants.
- Candidate animal variants:
  - `Bear` (accepted)
  - `Monkey` (deferred)
  - `Penguin` (accepted)
  - `Dinosaur`
  - `Unicorn`
  - `Bat`
  - `Turtle`
- Candidate fantasy and sci-fi variants:
  - `Dragon` (accepted)
  - `Ninja` (accepted)
  - `Vampire`
  - `Cyborg`
  - `Astronaut` (accepted)
- Candidate object variants:
  - `Diamond` (accepted)
  - `Gemstone`
  - `CoffeeCup` (accepted)
  - `Sword`
  - `Shield` (accepted)
  - `Crown`
- Add new `AvatarBackground` modes where they do not require external assets
  or unbounded procedural work.
- Candidate pattern backgrounds:
  - `PolkaDot`
  - `Striped`
  - `Checkerboard`
  - `Grid`
  - `Wavy`
- Candidate gradient backgrounds:
  - `Sunrise`
  - `Synthwave`
  - `Ocean`
- Candidate environment backgrounds:
  - `Starry`
  - `Pixel`
  - `Cloudy`

Accepted `0.12.0` families are `Bear`, `Penguin`, `Dragon`, `Ninja`,
`Astronaut`, `Diamond`, `CoffeeCup`, and `Shield`. Pattern, gradient, and
environment backgrounds are deferred because they need a separate bounded
raster/SVG texture implementation and contrast review before they are safe
public API.

### Admission Policy

- Each new variant must be visually distinct at small sizes such as 64x64 and
  128x128.
- Each new variant must work in raster and SVG output.
- New backgrounds must be bounded, deterministic, and safe for untrusted
  identities.
- Pattern and environment backgrounds must not make foreground avatars hard to
  read; use contrast tests or reviewed golden outputs where practical.
- Variant expansion must update enum `ALL` lists, parser/display tests,
  README examples, and release notes together.
- Adding variants may change automatic distribution only in a documented
  visual-output compatibility release. Services that need stable old output
  should pin the older `style_version` until they intentionally migrate.

### Finish Line

`0.12.0` is done when:

- The accepted `AvatarKind` and `AvatarBackground` variants are implemented in
  both raster and SVG paths.
- Every new public variant has `as_str`, `Display`, `FromStr`, and `ALL`
  coverage.
- Automatic selection uses the enum variant lists, not duplicated literal
  counts.
- Golden fingerprints cover representative new avatar families and
  backgrounds.
- Small-size visual review confirms each accepted variant is recognizable.
- README and docs describe the expanded catalog without promising variants
  that were rejected or deferred.
- Release notes clearly state any deterministic output impact.
- `scripts/stable_release_gate.sh check` passes.
- crates.io publish dry run passes.

## 0.13.0: Background Expansion And Determinism Hardening

Status: implemented in `0.13.0`.

Goal: improve the background catalog without adding new avatar families, and
reduce platform-sensitive geometry in frame-shape masking before `1.0`.

### Scope

- Add a small accepted background set:
  - `PolkaDot`
  - `Striped`
  - `Checkerboard`
  - `Grid`
  - `Sunrise`
  - `Ocean`
  - `Starry`
- Keep background rendering deterministic, bounded, asset-free, and represented
  in both raster and SVG output.
- Avoid admitting busier backgrounds such as `Wavy`, `Synthwave`, `Pixel`, and
  `Cloudy` until they have a clearer contrast and readability story.
- Replace selected float-based frame-shape raster hit-testing with integer
  arithmetic.
- Fix stale documentation around feature-combination testing and fuzz encoder
  coverage.

### Finish Line

`0.13.0` is done when:

- Every accepted background has `as_str`, `Display`, `FromStr`, and `ALL`
  coverage.
- Raster tests prove the new background canvases are distinct and bounded.
- SVG tests parse every new background as well-formed XML.
- Golden fingerprints document the automatic-output impact from expanding
  `AvatarBackground::ALL`.
- README, dependency docs, changelog, and release notes describe the new
  background catalog and the feature-testing policy.
- `scripts/stable_release_gate.sh check` passes.
- crates.io publish dry run passes.

## 1.0.0: Stability Contract

Status: release candidate.

Goal: freeze a professional, security-oriented public API and rendering
contract.

### Scope

- Freeze default hash algorithm and default visual style version.
- Freeze the visual layer option model.
- Freeze the baseline avatar kind and background catalog intended for `1.0`.
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
