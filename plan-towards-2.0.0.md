# hashavatar Plan Towards 2.0.0

Status: planning document

Current stable line: `1.1.x`

Target: a security-oriented `2.0.0` workspace whose canonical safe-Rust CPU
renderer produces a validated fixed-point scene, deterministic raw RGBA pixels,
and SVG from one artistic source of truth.

This roadmap is intentionally granular. Every version is a review stop, not a
promise that several adjacent versions will be implemented in one batch. Split
a milestone again whenever its implementation, review, pentest, or migration
surface becomes too large for one release.

## Why 2.0 Exists

The `1.x` renderer has a strong bounded-input and security foundation, but its
raster and SVG implementations are separate artistic implementations. It also
uses floating-point geometry, `StdRng`, direct `image::RgbaImage` coupling, and
automatic selection based on mutable variant-list lengths.

The major release is justified by five linked changes:

1. A validated scene becomes the only source of family geometry.
2. Fixed-point math and a canonical CPU rasterizer define the pixel contract.
3. Trait and catalog derivation become explicitly versioned and frozen.
4. Unsupported style combinations become strict errors in the primary API.
5. The crate becomes a focused workspace so codecs, GPU dependencies, schemas,
   and test infrastructure do not weaken or enlarge the canonical core.

`2.0` does not promise that its output matches `1.x`. It promises a clearer,
stronger, versioned contract from `2.0` onward.

## Non-Negotiable Engineering Rules

- Keep first-party production crates on stable Rust, edition 2024, with MSRV
  checked independently from the pinned development toolchain.
- Use the latest MSRV-compatible stable crate and tool versions unless a
  documented security, license, regression, or maintenance reason requires an
  older version.
- Keep GitHub CodeQL on default setup unless a documented repository decision
  explicitly replaces it.
- First-party production crates use `#![forbid(unsafe_code)]`.
- A future need for direct unsafe code must be isolated in a dedicated crate,
  documented per block, tested with Miri or sanitizers where applicable, and
  independently security-reviewed before admission.
- Third-party unsafe code is not treated as first-party unsafe code, but it is
  still a trust boundary. Record why each such dependency is needed and which
  crate isolates it.
- Normal non-generated Rust source files, including tests, examples, benches,
  and fuzz targets, must stay at or below 500 lines.
- Review every Rust file approaching 300 lines for a coherent split before it
  reaches the hard limit.
- Keep `lib.rs` files focused on crate documentation, module wiring, and public
  re-exports.
- New 2.0 crates inherit strict workspace lints: denied missing documentation
  and unused results, forbidden panic/unwrap/expect in production, and strict
  review of indexing, arithmetic side effects, truncating casts, and sign-loss
  casts. Apply these to legacy 1.x code incrementally instead of hiding a large
  lint waiver in the workspace root.
- No HTTP server, async runtime, filesystem-writing API, CLI, rate limiter,
  query-string parser, or network client belongs in the core or facade.
- Hash modes remain explicit. A non-cryptographic hash must never be selected
  accidentally through runtime distribution or Cargo feature unification.
- Every parser and public resource boundary is bounded, rejects malformed input
  with typed errors, and has panic-policy tests.
- Rendering changes require deterministic fixtures and explicit release notes.
- Kani, fuzzing, differential tests, golden tests, pentests, RustSec, dependency
  policy, SBOMs, and CodeQL are complementary evidence. None replaces another.
- Temporary root `PENTEST.md` files are review input and must be removed after
  findings are handled.
- No version is tagged until its exact implementation has passed local gates,
  GitHub CI, CodeQL default setup, integration testing, and the requested
  pentest/retest cycle.

## Practices Adopted From `eth`

This roadmap intentionally adopts the parts of the `eth` workflow that scale to
an image-generation project:

- a facade over focused inward dependencies rather than one implementation
  crate that accumulates every optional backend;
- independently versioned workspace crates and a machine-readable release
  matrix instead of lockstep publication;
- enforced 500-line Rust modules and early review at 300 lines;
- scripts that validate dependency direction and minimal/default feature
  graphs, not only manifest text;
- release-only network checks for current stable Rust, cargo tools, crate
  versions, and immutable GitHub Action pins;
- semantic committed SBOM comparison rather than checking only that generation
  succeeds;
- release notes and permanent sanitized pentest evidence tied to the exact
  reviewed implementation commit;
- temporary detailed `PENTEST.md` findings removed after remediation;
- one small implementation stop at a time with a named verification command
  and no automatic tag.

Ethereum-specific source/spec synchronization, fork matrices, node fixtures,
and consensus policies do not belong here. Hashavatar replaces them with a
versioned scene/pixel corpus, art-contract inventory, and cross-backend output
evidence.

## Modularity Policy

The workspace must have a one-way dependency graph:

```text
hashavatar (facade)
  |-- hashavatar-core
  |-- hashavatar-formats  --> hashavatar-core
  |-- hashavatar-gpu      --> hashavatar-core
  `-- hashavatar-schema   --> hashavatar-core

hashavatar-testkit --> every crate, dev/test use only
```

No lower-level crate may depend on the facade. `hashavatar-core` may not depend
on formats, GPU, schema, `image`, `wgpu`, web frameworks, or JSON libraries.
The primary package remains named `hashavatar` on crates.io; the workspace split
does not rename the crate existing users install. Companion crates start on
independent `0.x` lines and are published only when their own finish line is
met. Facade features may forward to them, but direct use remains possible.

### `hashavatar-core`

Published, `no_std + alloc` capable, and the owner of:

- bounded identity and namespace derivation;
- optional keyed pseudonymization primitives;
- stable catalog and render-contract IDs;
- request, style, family, rig, and capability models;
- fixed-point arithmetic and integer color/compositing rules;
- scene authoring, validation, canonical serialization, and scene digest;
- canonical safe-Rust CPU rasterization into caller-provided RGBA memory;
- `SceneSink`, `RasterSurfaceMut`, and bounded scratch/capacity contracts;
- SVG writing through `core::fmt::Write` or a crate-local sink;
- resource budgets and typed errors.

The earlier `hashavatar-core` split was correctly rejected because the old
architecture still fundamentally depended on image rendering and codecs. The
2.0 fixed-point scene and raw-surface renderer provide a concrete, useful core
boundary that did not exist then.

### `hashavatar-formats`

Published, `std`-oriented, default-empty, and the owner of:

- `image` compatibility adapters;
- WebP, PNG, JPEG, GIF, and any later admitted raster encoders;
- writer and caller-slice adapters where the underlying codec supports them;
- a bounded `ByteSink` abstraction where `std::io::Write` is not the right
  public boundary;
- encoder capability metadata and codec-specific resource estimates;
- decode-and-compare tests against canonical core pixels;
- documentation for codec-owned scratch buffers and zeroization limits.

The facade may enable WebP by default for compatibility. Extra formats remain
explicit opt-ins. `all-formats` must include only fully admitted stable formats;
experimental codecs do not join it.

### `hashavatar-gpu`

Published independently and never enabled by default. It owns:

- GPU device/queue integration and backend-specific resources;
- translation from validated core scenes into GPU commands;
- explicit GPU resource ceilings;
- buffer zero-fill and completion-fence policy before reuse when sensitive
  cleanup is requested;
- differential tests against the canonical CPU pixel corpus.

GPU output starts as **visually conforming and noncanonical**. It may only gain
a bit-identical claim after integer execution and multi-vendor evidence prove
zero raw-pixel differences. The CPU renderer remains normative for `2.0.0`.

### `hashavatar-schema`

Published independently and never enabled by default. It owns a versioned,
transport-neutral `AvatarRequestDocument` plus optional Serde and JSON Schema
support without adding those dependencies to core. It does not contain Axum
extractors or parse HTTP query strings. Query parsing and service policy remain
in `hashavatar-website` or another caller application.

### `hashavatar-testkit`

Initially `publish = false`. It owns deterministic request vectors, raw pixel
fixtures, scene fixtures, SVG structural fixtures, cross-backend comparisons,
allocation counters, and downstream integration helpers. Production crates may
not depend on it.

### Conditional `hashavatar-compat-v1`

Create this only if real downstream migration demand requires exact `1.x`
pixels after `2.0`. It would contain the frozen legacy renderer, receive no new
art features, and have a documented support end. Do not burden the normal 2.0
facade with two permanent rendering engines by default.

## Contract Vocabulary

- `CatalogVersion`: freezes family/background/style IDs, weights, and automatic
  selection mappings.
- `RenderContractId`: freezes trait derivation, scene semantics, fixed-point
  rules, compositing, and canonical raster behavior.
- `SceneDigest`: hashes a canonical internal representation of one validated
  scene. The serialization need not become a public interchange format.
- `PixelDigest`: hashes canonical premultiplied RGBA8 output from the reference
  CPU rasterizer.
- `IdentityCacheKey`: identifies one pseudonymous subject within an identity
  protocol and domain.
- `AvatarAssetKey`: additionally binds catalog, render contract, dimensions,
  seed, resolved style, and pixel backend contract.
- `EncodedAssetKey`: additionally binds output format, encoder contract/version,
  and encoding settings.

## Website-Driven Reusable Building Blocks

`hashavatar-website` is the hosted `hashavatar.app` reference implementation,
not the reusable API layer. Its current request path identifies several pieces
that should become reusable without pulling web concerns into the workspace.

### Core Request Preparation

`hashavatar-core` should provide:

- a transport-neutral `AvatarRequest` containing only rendering inputs;
- an `AvatarRequestBuilder` for ergonomic Rust construction;
- a validated `PreparedAvatar` that owns or borrows the derived identity,
  validated spec, catalog/render IDs, resolved style, and resource budget;
- `ResolvedStyle` and `LayoutReport` so callers do not independently normalize
  unsupported family/accessory/expression combinations;
- canonical `IdentityCacheKey` and `AvatarAssetKey` derivation;
- strict and documented automatic-fallback policies;
- redacted `Debug` and cleanup behavior for identity-bearing request stages.

The prepared form prevents applications from validating one request tuple and
accidentally rendering or caching a different tuple.

Illustrative API shape:

```rust,ignore
let prepared = AvatarRequest::builder(identity)
    .namespace(namespace)
    .spec(spec)
    .style(style)
    .catalog(CatalogVersion::V2)
    .prepare()?;

let budget = prepared.resource_budget();
let key = prepared.asset_key();
prepared.render_into(&mut surface)?;
```

### Encoded Asset Preparation

`hashavatar-formats` should accept `PreparedAvatar` or canonical pixels and
return an `EncodedAvatar` containing:

- encoded bytes or writer completion metadata;
- media type and conventional extension;
- alpha support and encoder capability metadata;
- `EncodedAssetKey` derived from the core asset key plus encoder contract and
  settings;
- documented scratch-memory and cleanup limitations.

The crate may expose stable key bytes and hex formatting. HTTP ETag quoting,
cache-control durations, CDN headers, and object-storage path layout remain
caller policy.

### Versioned Wire Document

`hashavatar-schema` should provide a strict, bounded
`AvatarRequestDocumentV1` that converts into core `AvatarRequest`. It includes
only rendering fields. Website-only fields such as `persist`, `redirect`, S3
configuration, locale, telemetry, and rate-limit policy are excluded.

The adapter must reject unknown or duplicate fields where the selected parser
can express that policy, bound strings before excessive allocation where
possible, and surface typed conversion errors. Its schema records supported
catalog/render IDs and style capabilities. OpenAPI documents remain the
responsibility of the hosting application, which can embed this schema.

### Reference Service Recipe

The main documentation should show a short framework-neutral service recipe:

1. Parse transport input under application limits.
2. Convert it into `AvatarRequestDocumentV1` or build `AvatarRequest` directly.
3. Call `prepare()` before entering expensive work.
4. Use the reported resource budget to acquire an application-owned permit.
5. Run CPU rendering away from async runtime worker threads.
6. Encode with `hashavatar-formats`.
7. Build HTTP caching and storage metadata from canonical asset keys.

`hashavatar-website` remains the full production example for Axum, semaphore
limits, timeouts, proxy trust, security headers, telemetry, object storage,
localization, CDN policy, and deployment.

The reusable workspace must not absorb:

- Axum request/response types;
- Tokio tasks, semaphores, or timeout policy;
- service-specific limits stricter than the crate's hard safety bounds;
- rate limiting, trusted-proxy handling, or browser security headers;
- S3 clients, object-key prefixes, redirects, or signed URLs;
- OpenTelemetry instruments, website pages, translations, or deployment code.

Encoded WebP, PNG, JPEG, GIF, AVIF, or JPEG XL bytes are deterministic only when
an explicitly versioned encoder contract says so. Decoding to identical pixels
does not imply identical encoded bytes across dependency upgrades.

SVG is a lossless serialization of scene semantics where supported. Browser
rasterization is not the canonical pixel renderer and is not promised to match
the CPU rasterizer byte-for-byte.

## Numeric And Scene Rules

- Use signed Q16.16 scene coordinates backed by `i32`, with proven `i64`
  intermediates for transforms and multiplication.
- Do not model normalized `1.0` as an impossible Q0.16 `u16` value. Choose and
  freeze either a checked `u32` range `0..=65536` or an explicitly named UNORM16
  range `0..=65535` with denominator `65535`.
- Define one signed division, remainder, tie-breaking, and rounding policy.
- Reject invalid construction at trust boundaries; saturation is reserved for
  explicitly documented resource estimates or clipping behavior.
- Use exact premultiplied RGBA8 Porter-Duff source-over arithmetic.
- Lower curves with bounded integer de Casteljau subdivision or another proven
  deterministic method with an explicit stack and maximum depth.
- Validate command ranges, path ranges, point ranges, stack balance, transform
  depth, clip depth, coordinate bounds, command count, path complexity, and
  estimated raster cost before execution.
- Do not admit arbitrary SVG fragments, external images, shaders, filters, or
  unbounded recursion into canonical scenes.
- `OwnedScene` uses `alloc`; `BorrowedScene` and caller-provided scratch storage
  are admitted only with typed capacity errors and without introducing
  first-party unsafe code.

## Family Rig And Style Rules

- Use a hierarchy of optional semantic anchors rather than assuming every
  family has human anatomy.
- Provide focused `FaceRig`, `EyeRig`, `HeadRig`, or `BodyRig` capabilities only
  where a family actually supports them.
- Calibrate per-family slot transforms and exclusion zones before relying on a
  generic collision solver.
- Typed slots include back accessory, aura, headwear, earwear, facewear,
  eyewear, neckwear, left/right handheld, and foreground effect.
- Canonicalize accessory stacks by slot and stable accessory ID, never caller
  insertion order.
- Manual styles return errors for unsupported combinations in 2.0.
- Automatic styles use a frozen, documented fallback policy.
- Resolution returns a `LayoutReport` describing accepted, adjusted, rejected,
  and automatically substituted layers.
- Prefer a strict dynamic request/builder API. Do not add one marker type per
  family unless compile-time capability APIs demonstrate clear downstream value
  during alpha testing.

## Security And Privacy Rules

- Use "pseudonymization," never "anonymization." Stable avatars and cache keys
  remain correlators.
- Keyed identity support must use a reviewed standard PRF/KDF construction with
  fixed protocol labels, algorithm IDs, versions, domains, and length prefixes.
  Do not invent a custom cryptographic protocol from ad hoc hash calls.
- Keys have redacted `Debug`, no serialization, no accidental `Clone`, bounded
  input, and sanitization on drop.
- Identifier normalization remains caller policy.
- Construct sanitizing scene, canvas, scratch, and partial-output owners before
  sensitive derived data is written, not after rendering succeeds.
- Cleanup claims cover hashavatar-owned allocations on normal return, errors,
  and unwinding. Process abort, registers, compiler copies, paging, crash dumps,
  driver memory, and inaccessible codec scratch remain explicit residuals.
- Caller-owned final images and encoded output remain caller responsibility;
  optional sanitizing output guards may be provided.
- Rendering remains variable-time. High-assurance service timing, concurrency,
  caching, and rate limiting remain application responsibilities.

## Dependency And License Admission

Every new dependency or feature path requires:

- latest stable and latest MSRV-compatible version review;
- complete feature and transitive graph review;
- license review through `cargo-deny`;
- RustSec and maintenance-history review;
- default-feature and platform review;
- unsafe/FFI/native-toolchain inventory;
- resource and zeroization-boundary documentation;
- focused tests and package-size evidence;
- a decision whether it is runtime, optional backend, reference-only, fuzz-only,
  or test-only.

As reviewed on 2026-07-20 (versions must be rechecked before implementation):

- `image 0.25.10`'s pure-Rust AVIF encoder path uses `ravif`; current
  `ravif 0.13.0` and `rav1e 0.8.1` use BSD-3-Clause and BSD-2-Clause. Those
  licenses are admitted by the existing policy, so AVIF is not currently
  blocked by license. It is still blocked pending graph, assembly/threading,
  compile-time, MSRV, resource, determinism, and security review in
  `hashavatar-formats`.
- `jpegxl-rs 0.15.0+libjxl-0.12.0`, the current reference-wrapper release, is
  GPL-3.0-or-later and is not admissible under the permissive dependency policy.
  It also declares Rust `1.92.0`, newer than the present MSRV.
- `jxl-oxide 0.12.6` is permissively licensed but is a decoder, not the required
  output encoder.
- `zune-jpegxl 0.5.2` advertises a permissively licensed Rust encoder, but
  license compatibility alone is insufficient. Conformance, feature
  completeness, maintenance, security, output quality, resource bounds, and
  MSRV must be evaluated before admission.
- `wgpu 30.0.0` is permissively licensed and currently MSRV-compatible, but its
  large, platform-specific dependency and unsafe boundary is exactly why it
  belongs in `hashavatar-gpu`, never `hashavatar-core` or the default facade
  graph.

These observations are time-sensitive and must be rechecked at implementation.

## Release Discipline

Every milestone below must include:

- `Status`, `Goal`, `Deliverables`, `Verification`, and `Exit criteria` in its
  release notes;
- current dependency/tool checks and immutable GitHub Action pins;
- formatting, clippy, tests, docs, panic/unsafe/modularity policies;
- MSRV and pinned-stable checks for every supported feature profile;
- `cargo deny`, `cargo audit`, lockfile review, and SBOM evidence;
- package-content and reproducibility checks;
- release-specific fixtures, fuzz builds, proofs, or differential evidence;
- updated limitations, threat model, dependency policy, migration notes, and
  crate-version matrix where relevant;
- temporary pentest findings removed after remediation;
- a sanitized permanent `security/pentest/vX.Y.Z.md` PASS report that identifies
  the exact reviewed implementation commit without publishing exploit details;
- GitHub CI and CodeQL default setup green before tagging.

Once the workspace exists, crates use independent versions. A
`release-crates.toml` file records whether each crate changed because of code,
dependency metadata, documentation/metadata, or not at all. Do not republish
every companion crate for every facade release.

At each implementation stop, report exactly:

```text
vX.Y.Z implementation stop reached. Run pentest for this exact commit.
```

## 1.x Preparation Releases

All `1.x` work is additive and preserves existing explicit render output unless
a correctness or security fix is separately documented. Private prototypes are
not public stability commitments.

At the time this plan was written, the files over the future hard limit were
`src/tests.rs` (2516 lines), `src/layers.rs` (1336), `src/model.rs` (894),
`src/api.rs` (864), `src/core.rs` (686), and `src/primitives.rs` (584).
`src/cat_support.rs` was already near the limit at 497 lines. The modularity
releases below must split by ownership, not merely move arbitrary line ranges.

### v1.1.3 - Policy Corrections

**Status:** Planned.

**Goal:** Make current documentation match current code before planning new API.

**Deliverables:** Correct the Kani count from four to five; state that current
public enums are exhaustive and adding variants is breaking until 2.0; fix the
duplicate encoding example line in the idea document or supersede that document
with this roadmap.

**Verification:** Documentation links, Kani harness inventory, and semver review.

**Exit criteria:** No behavior or fingerprint change; policy contradictions are
removed. `v1.1.3 implementation stop reached. Run pentest for this exact commit.`

### v1.2.0 - Release Assurance Foundation

**Status:** Planned.

**Goal:** Adopt the useful release discipline from `eth` before architecture
work begins.

**Deliverables:** Add threat-model, modularity, unsafe, supply-chain, and release
readiness documents; add networked latest-tool checks for release gates; add
semantic SBOM drift checking; define exact pentest handoff and tag readiness.

**Verification:** Script self-tests, clean/offline behavior tests, deliberate
stale-tool fixtures, SBOM drift fixtures, and the existing stable gate.

**Exit criteria:** Release gates fail closed on stale or missing required
evidence without changing avatar output. `v1.2.0 implementation stop reached.
Run pentest for this exact commit.`

### v1.3.0 - Source Modularity, Part I

**Status:** Planned.

**Goal:** Split the largest production modules without changing behavior.

**Deliverables:** Split `api.rs`, `model.rs`, and `core.rs` by ownership; keep
public re-exports stable; add a line-count report and a 300-line review warning.

**Verification:** Public API diff, compile-tested examples, existing goldens,
panic policy, and module-boundary tests.

**Exit criteria:** The named production files are at or below 500 lines and all
fingerprints remain unchanged. `v1.3.0 implementation stop reached. Run pentest
for this exact commit.`

### v1.4.0 - Source Modularity, Part II

**Status:** Planned.

**Goal:** Complete and enforce the 500-line policy.

**Deliverables:** Split `layers.rs`, `primitives.rs`, `tests.rs`,
`cat_support.rs`, and any other remaining first-party Rust file at or near 500
lines; enforce the limit in CI while excluding generated/vendor output only.

**Verification:** Deliberate over-limit fixture, module test ownership review,
goldens, fuzz harness build, and full release gate.

**Exit criteria:** Every non-generated first-party `.rs` file is at or below 500
lines. `v1.4.0 implementation stop reached. Run pentest for this exact commit.`

### v1.5.0 - Frozen Catalog And Contract IDs

**Status:** Planned.

**Goal:** Stop future art additions from silently reshuffling established
automatic identities.

**Deliverables:** Add explicit legacy `CatalogVersion`, `RenderContractId`,
stable built-in IDs and weights, `IdentityCacheKey`, `AvatarAssetKey`, and
`EncodedAssetKey`; preserve the existing automatic mapping as the legacy
catalog.

**Verification:** Frozen mapping vectors for all byte values, cache-key domain
separation tests, and unchanged 1.x visual fingerprints.

**Exit criteria:** New catalogs can be added later without mutating legacy
mappings. `v1.5.0 implementation stop reached. Run pentest for this exact
commit.`

### v1.6.0 - Strict Style Validation Preview

**Status:** Planned.

**Goal:** Expose compatibility errors without changing legacy skip behavior.

**Deliverables:** Add family capability manifests, `validate_style`, a strict
opt-in builder path, stable compatibility errors, and a preview `LayoutReport`.

**Verification:** Every family/accessory/expression combination is classified;
manual invalid combinations fail only on the strict path; legacy output stays
unchanged.

**Exit criteria:** Downstream applications can migrate away from silent skips
before 2.0. `v1.6.0 implementation stop reached. Run pentest for this exact
commit.`

### v1.7.0 - Keyed Pseudonymization

**Status:** Planned.

**Goal:** Make dictionary-resistant identity preparation available without
requiring every caller to design its own protocol.

**Deliverables:** Add an optional keyed-identity feature using a reviewed
standard construction; define identity domain, purpose, protocol version, and
key ID; add non-clone key ownership and sanitization behavior.

**Verification:** Published known-answer vectors, domain/tenant/purpose
separation, key rotation, redacted logging, clone-policy, cleanup, malformed
input, and independent cryptographic design review.

**Exit criteria:** The feature is opt-in, default outputs are unchanged, and the
security documentation does not claim anonymity. `v1.7.0 implementation stop
reached. Run pentest for this exact commit.`

### v1.8.0 - Compatibility Output Sinks

**Status:** Planned.

**Goal:** Let applications adopt writer and caller-buffer APIs before 2.0.

**Deliverables:** Add `write_svg`, `encode_to_writer`, a validated raster surface
adapter, and an additive prepared-request facade around the 1.x renderer.
Expose effective style, resource budget, and canonical key material from one
validated preparation step. Document honestly where the legacy engine or codec
still allocates internally; do not claim zero allocation merely because the
final `Vec` is avoided.

**Verification:** Short writes, writer failures, stride errors, undersized
buffers, unwind cleanup, decoded-pixel parity, and allocation-count evidence.

**Exit criteria:** New APIs are additive and output-equivalent to existing
helpers. `v1.8.0 implementation stop reached. Run pentest for this exact
commit.`

### v1.9.0 - Fixed-Point Arithmetic Shadow

**Status:** Planned.

**Goal:** Implement and prove the numeric contract without switching production
rendering.

**Deliverables:** Add private fixed coordinate, normalized value, transform,
coverage, interpolation, and compositing modules with explicit rounding rules.

**Verification:** Boundary/property tests and Kani proofs for construction,
overflow, multiplication, division, transform composition, channel bounds, and
source-over arithmetic.

**Exit criteria:** Numeric rules are documented and proven for admitted bounds;
1.x output is unchanged. `v1.9.0 implementation stop reached. Run pentest for
this exact commit.`

### v1.10.0 - Stateless Trait Derivation Shadow

**Status:** Planned.

**Goal:** Replace mutable RNG coupling in the future contract without changing
the active 1.x renderer.

**Deliverables:** Add private domain-separated stateless `derive_u16`/range
selection under an explicit derivation version; freeze labels and counter
rules; prevent modulo/catalog drift.

**Verification:** Known-answer vectors on x86_64, AArch64, and WASM; domain
collision tests; distribution sanity tests; no `StdRng` use in the shadow path.

**Exit criteria:** The future trait genome is reproducible without mutable RNG
state. `v1.10.0 implementation stop reached. Run pentest for this exact
commit.`

### v1.11.0 - Private Scene Prototype

**Status:** Planned.

**Goal:** Prove the scene architecture before exposing it publicly.

**Deliverables:** Add private scene commands, paints, paths, transforms, clips,
opacity groups, semantic IDs, validation, budgets, and canonical digest
serialization; support `OwnedScene` first.

**Verification:** Validator fuzz target, malformed range/stack/capacity corpus,
canonical serialization tests, bounded-cost tests, and Kani range proofs.

**Exit criteria:** A private scene can be built, validated, serialized, and
rejected safely without becoming 1.x public API. `v1.11.0 implementation stop
reached. Run pentest for this exact commit.`

### v1.12.0 - Final 1.x Compatibility Corpus

**Status:** Planned.

**Goal:** Freeze complete migration evidence before the major-version branch.

**Deliverables:** Publish request vectors, resolved styles, raw RGBA hashes,
decoded codec hashes, SVG fixtures, cache-key fixtures, allocation/resource
baselines, and deprecation guidance for APIs that will not survive 2.0.

**Verification:** Reproduce the corpus on supported Rust versions and at least
x86_64, AArch64, and WASM where applicable; verify signed/checksummed evidence.

**Exit criteria:** The 1.x contract can be tested independently during the 2.0
rewrite. `v1.12.0 implementation stop reached. Run pentest for this exact
commit.`

## 2.0 Alpha Releases

Alpha releases may change API and output. Every alpha still receives normal
security, dependency, documentation, and pentest treatment.

### v2.0.0-alpha.1 - Workspace Boundary

**Status:** Planned.

**Goal:** Convert the repository to the planned workspace without implementing
new rendering behavior.

**Deliverables:** Create facade, core, formats, GPU, optional schema, and
nonpublished testkit crate skeletons; establish inward dependency rules,
independent versions, release matrix, workspace lints, and package checks.

**Verification:** Dependency-direction tests, default graph assertions,
individual package dry runs, and 500-line enforcement.

**Exit criteria:** Empty or compatibility-backed crates compile with no cycles or
accidental heavy default dependencies. `v2.0.0-alpha.1 implementation stop
reached. Run pentest for this exact commit.`

### v2.0.0-alpha.2 - Fixed Numeric Core

**Status:** Planned.

**Goal:** Promote the shadow fixed-point contract into `hashavatar-core`.

**Deliverables:** Public checked numeric types, transforms, colors, coverage,
premultiplied compositing, exact rounding documentation, and proof contracts.

**Verification:** Unit/property tests and mandatory Kani proofs for every public
operation and valid boundary.

**Exit criteria:** Core fixed math is float-free and proof-clean.
`v2.0.0-alpha.2 implementation stop reached. Run pentest for this exact commit.`

### v2.0.0-alpha.3 - Scene Storage And Validation

**Status:** Planned.

**Goal:** Establish bounded trusted scenes before any backend executes them.

**Deliverables:** `OwnedScene`, validated display lists, arena/range types,
command/path/paint/transform/clip limits, stack validation, resource estimates,
and `SceneDigest`.

**Verification:** Validator fuzzing, canonical digest vectors, malformed scenes,
capacity exhaustion, stack imbalance, and Kani bounds.

**Exit criteria:** No backend can execute an unvalidated scene.
`v2.0.0-alpha.3 implementation stop reached. Run pentest for this exact commit.`

### v2.0.0-alpha.4 - Canonical Raster Primitives

**Status:** Planned.

**Goal:** Build the safe-Rust reference raster foundation.

**Deliverables:** Caller-provided RGBA surface, tiles/scanlines, fill rules,
rectangles, ellipses, lines, integer coverage, clipping, and exact pixel-index
validation.

**Verification:** Primitive goldens, arbitrary stride/dimension tests, fuzzing,
Kani pixel-index proofs, debug/release/LTO equality, and resource ceilings.

**Exit criteria:** Primitive raw pixels are canonical on the first supported
platform matrix. `v2.0.0-alpha.4 implementation stop reached. Run pentest for
this exact commit.`

### v2.0.0-alpha.5 - Paths, Curves, Strokes, And Groups

**Status:** Planned.

**Goal:** Complete the canonical scene operations required by current artwork.

**Deliverables:** Bounded paths, integer curve flattening, stroke lowering,
nested clips, opacity groups, gradients, and complete source-over compositing.

**Verification:** Termination/depth proofs, endpoint preservation, fill-rule
fixtures, adversarial paths, clip-stack fuzzing, and compositing vectors.

**Exit criteria:** Every existing family can be represented without adding
unbounded or backend-specific scene commands. `v2.0.0-alpha.5 implementation
stop reached. Run pentest for this exact commit.`

### v2.0.0-alpha.6 - Active Catalog And Trait Derivation

**Status:** Planned.

**Goal:** Activate the frozen stateless identity-to-trait contract.

**Deliverables:** Versioned trait labels, catalog IDs, frozen weighted mapping,
explicit seed handling, stable style resolution, and asset-key integration.

**Verification:** Cross-architecture known-answer vectors, catalog permutation
tests, domain separation, key uniqueness tests, and no `StdRng` in core.

**Exit criteria:** Identical requests resolve to identical traits without
mutable RNG state. `v2.0.0-alpha.6 implementation stop reached. Run pentest for
this exact commit.`

### v2.0.0-alpha.7 - Cat Vertical Slice

**Status:** Planned.

**Goal:** Prove the complete architecture with one production-quality family.

**Deliverables:** Cat rig and scene compiler, themed background, canonical CPU
pixels, SVG from the same scene, shape clipping, and scene/pixel/SVG fixtures.

**Verification:** Visual review, raw pixel corpus across x86_64/AArch64/WASM,
SVG parsing/structural parity, and bounded resource evidence.

**Exit criteria:** Cat has no independent raster/SVG geometry path.
`v2.0.0-alpha.7 implementation stop reached. Run pentest for this exact commit.`

### v2.0.0-alpha.8 - Animal And Face Family Port

**Status:** Planned.

**Goal:** Port dog, fox, bear, panda, frog, penguin, and bird.

**Deliverables:** Calibrated rigs, safe zones, semantic anchors, scene compilers,
and canonical fixtures for the named families.

**Verification:** Per-family visual review, minimum/maximum dimensions, all
frame shapes, SVG parsing, and cross-platform pixel digests.

**Exit criteria:** Every named family uses the scene path only.
`v2.0.0-alpha.8 implementation stop reached. Run pentest for this exact commit.`

### v2.0.0-alpha.9 - Fantasy And Constructed Family Port

**Status:** Planned.

**Goal:** Port robot, alien, monster, ghost, wizard, skull, knight, dragon,
ninja, and astronaut.

**Deliverables:** Family-specific rigs/capabilities, scene compilers, and
canonical fixtures.

**Verification:** The same family, dimension, frame, SVG, and platform gates as
alpha.8, plus complex-path resource checks.

**Exit criteria:** Every named family uses the scene path only.
`v2.0.0-alpha.9 implementation stop reached. Run pentest for this exact commit.`

### v2.0.0-alpha.10 - Object, Food, And Remaining Family Port

**Status:** Planned.

**Goal:** Port slime, paws, planet, rocket, mushroom, cactus, cupcake, pizza,
icecream, octopus, diamond, coffee cup, and shield.

**Deliverables:** Non-face capability manifests, scene compilers, and canonical
fixtures without fake eye/head anchors.

**Verification:** The same family, dimension, frame, SVG, and platform gates as
earlier ports; verify unsupported face capabilities fail explicitly.

**Exit criteria:** Every 1.x family is represented by the 2.0 scene system.
`v2.0.0-alpha.10 implementation stop reached. Run pentest for this exact
commit.`

### v2.0.0-alpha.11 - Backgrounds And Frames

**Status:** Planned.

**Goal:** Move every background and frame into canonical scene geometry.

**Deliverables:** Themed, fixed-color, transparent, pattern, gradient, and
starry backgrounds plus circle, square, squircle, hexagon, and octagon frames.

**Verification:** Raster/SVG scene parity, clipping boundaries, contrast review,
identity-dependent pattern vectors, and integer geometry proofs.

**Exit criteria:** No backend-specific background or frame implementation
remains. `v2.0.0-alpha.11 implementation stop reached. Run pentest for this
exact commit.`

### v2.0.0-alpha.12 - SVG Backend Completion

**Status:** Planned.

**Goal:** Finish streaming SVG emission and delete independent family SVG code.

**Deliverables:** Stable element/attribute ordering, exact fixed-point
formatting, clips/groups/gradients, `core::fmt::Write` support, and SVG
capability reporting.

**Verification:** Parse every request fixture, compare scene semantics, inject
writer failures, fuzz scene-to-SVG, and verify caller input never becomes raw
markup.

**Exit criteria:** SVG is generated only from validated scenes.
`v2.0.0-alpha.12 implementation stop reached. Run pentest for this exact
commit.`

### v2.0.0-alpha.13 - Typed Accessory Slots

**Status:** Planned.

**Goal:** Replace the single accessory enum with a slot-aware model.

**Deliverables:** Stable accessory IDs, typed slots, z-bands, required anchors,
family capability checks, explicit exclusion groups, and one calibrated item per
slot where current art supports it.

**Verification:** Valid/invalid slot matrix, insertion-order canonicalization,
family support tests, and raster/SVG parity.

**Exit criteria:** One accessory per slot resolves deterministically without
silent skipping. `v2.0.0-alpha.13 implementation stop reached. Run pentest for
this exact commit.`

### v2.0.0-alpha.14 - Multi-Accessory Layout

**Status:** Planned.

**Goal:** Support deterministic accessory stacks safely.

**Deliverables:** Bounded stack length, priority ordering, calibrated placement
candidates, scaling limits, frame-safe zones, limited collision hulls,
fallback/rejection policy, and complete `LayoutReport`.

**Verification:** Permutation invariance, capacity exhaustion, collision/fallback
fuzzing, every-family stress fixtures, and maximum-cost bounds.

**Exit criteria:** Multiple compatible slots work together and incompatible
stacks fail predictably. `v2.0.0-alpha.14 implementation stop reached. Run
pentest for this exact commit.`

### v2.0.0-alpha.15 - Expressions And Color Layers

**Status:** Planned.

**Goal:** Complete layered style behavior on the scene model.

**Deliverables:** Expression capabilities, integer palettes, primary/secondary
color roles, sunglasses/expression conflict policy, and deterministic automatic
fallbacks.

**Verification:** Every expression/palette/family classification, integer color
proofs, contrast checks, and raster/SVG parity.

**Exit criteria:** The complete style tuple is scene-native and bounded.
`v2.0.0-alpha.15 implementation stop reached. Run pentest for this exact
commit.`

### v2.0.0-alpha.16 - no_std And Caller Storage

**Status:** Planned.

**Goal:** Complete the portable raw-rendering profiles.

**Deliverables:** `no_std + alloc` core, caller-provided RGBA surfaces,
`OwnedScene`, optional preinitialized-slice `BorrowedScene`, reusable scratch
arenas, `SceneBudget`, and typed capacity failures.

**Verification:** `--no-default-features`, alloc profile, WASM, AArch64, and at
least one embedded target; allocation-count tests; no hidden filesystem, clock,
entropy, thread, or OS-I/O dependency.

**Exit criteria:** Documented portable profiles compile and render canonical
fixtures. `v2.0.0-alpha.16 implementation stop reached. Run pentest for this
exact commit.`

### v2.0.0-alpha.17 - Formats Crate Baseline

**Status:** Planned.

**Goal:** Isolate all standard raster codec and `image` integration costs.

**Deliverables:** `hashavatar-formats` WebP, PNG, JPEG, and GIF features;
`RgbaImage` adapters; writer APIs; capability metadata; scratch/resource and
cleanup documentation.

**Verification:** Per-feature dependency trees, deny/audit, MSRV, decoded-pixel
parity, alpha handling, malformed writer behavior, package size, and no codec in
the core graph.

**Exit criteria:** Stable legacy formats work through the optional formats
boundary. `v2.0.0-alpha.17 implementation stop reached. Run pentest for this
exact commit.`

### v2.0.0-alpha.18 - AVIF Admission Decision

**Status:** Planned.

**Goal:** Decide AVIF on current evidence without weakening defaults.

**Deliverables:** Review the latest pure-Rust encoder graph, licenses, MSRV,
assembly/threading defaults, unsafe boundaries, resource usage, output quality,
determinism, WASM behavior, and codec scratch cleanup; admit `avif` only if all
required gates pass.

**Verification:** Dependency report, representative encode/decode corpus,
resource benchmark, security review, and explicit accept/reject record.

**Exit criteria:** AVIF is either an isolated non-default formats feature with
evidence or a documented rejection; it does not block 2.0.
`v2.0.0-alpha.18 implementation stop reached. Run pentest for this exact commit.`

### v2.0.0-alpha.19 - JPEG XL Admission Decision

**Status:** Planned.

**Goal:** Re-evaluate JPEG XL providers without accepting GPL or immature code
silently.

**Deliverables:** Recheck `jpegxl-rs`, permissive Rust encoders, conformance,
maintenance, MSRV, resource limits, native dependencies, and security posture.
Do not admit GPL-3.0-or-later into the permissive workspace policy.

**Verification:** Provider comparison, license evidence, corpus results for any
candidate, deny/audit, and explicit accept/reject record.

**Exit criteria:** JPEG XL is either isolated behind an admitted provider or
remains deferred with a precise reason; it does not block 2.0.
`v2.0.0-alpha.19 implementation stop reached. Run pentest for this exact commit.`

### v2.0.0-alpha.20 - GPU Boundary Scaffold

**Status:** Planned.

**Goal:** Publish the optional GPU contract without claiming canonical output.

**Deliverables:** `hashavatar-gpu` device-independent API, scene capability
query, resource budget, unsupported-command errors, backend status, and a
deterministic CPU fallback decision that never happens silently.

**Verification:** Default graph exclusion, dependency/unsafe inventory,
headless/no-device behavior, resource-limit tests, and package review.

**Exit criteria:** Applications can opt into GPU integration explicitly while
core/facade defaults remain unchanged. `v2.0.0-alpha.20 implementation stop
reached. Run pentest for this exact commit.`

### v2.0.0-alpha.21 - GPU Scene Execution

**Status:** Planned.

**Goal:** Render complete validated scenes through the selected GPU backend.

**Deliverables:** Scene translation, integer-capable shader path where practical,
bounded buffers, deterministic command order, explicit zero-fill/fence cleanup,
and output-status metadata.

**Verification:** Differential corpus across available vendors/drivers, device
loss, timeout, allocation failure, malformed capability, cleanup, and stress
tests.

**Exit criteria:** GPU output is production-usable as explicitly noncanonical;
any mismatch is measured and documented. `v2.0.0-alpha.21 implementation stop
reached. Run pentest for this exact commit.`

### v2.0.0-alpha.22 - Request Wire And Schema Adapter

**Status:** Planned.

**Goal:** Make it easier to build services without making HTTP or JSON part of
rendering core.

**Deliverables:** Add independently versioned `hashavatar-schema` with bounded
`AvatarRequestDocumentV1`, strict conversion into core `AvatarRequest`, optional
Serde/JSON Schema support, catalog/render IDs, compatibility metadata, and
strict unknown/duplicate-value policy where supported.

**Verification:** Schema snapshots, malformed/unknown/duplicate field tests in
the chosen parser integration, feature-tree isolation, and MSRV review.

**Exit criteria:** Schema dependencies never enter core or default facade; HTTP
query parsing remains in `hashavatar-website` or another caller application.
`v2.0.0-alpha.22 implementation stop reached. Run pentest for this exact
commit.`

### v2.0.0-alpha.23 - Facade And Migration API

**Status:** Planned.

**Goal:** Assemble the intended user-facing 2.0 API.

**Deliverables:** `#[non_exhaustive]` built-in enums or stable IDs where
appropriate, strict dynamic builder, `PreparedAvatar`, resolved style/layout,
surface/SVG/encode helpers, canonical asset keys, optional formats/GPU/schema
forwarding, and complete 1.x migration mapping.

**Verification:** Public API review, compile-pass/fail examples, feature powerset
for valid combinations, downstream `hashavatar-website` migration, and no
accidental heavy dependencies in minimal/default graphs.

**Exit criteria:** All intended 2.0 workflows are possible through the facade.
`v2.0.0-alpha.23 implementation stop reached. Run pentest for this exact
commit.`

### v2.0.0-alpha.24 - Testkit And Compatibility Decision

**Status:** Planned.

**Goal:** Centralize assurance evidence and decide whether legacy compatibility
is justified.

**Deliverables:** `hashavatar-testkit`, shared corpus runner, CPU/GPU/SVG/codec
differential helpers, allocation/resource fixtures, and a documented decision
on conditional `hashavatar-compat-v1` based on downstream demand.

**Verification:** Corpus reproducibility, tamper detection, cross-crate use,
package exclusion, and compatibility cost review.

**Exit criteria:** Every backend consumes one evidence corpus; legacy renderer
code is either isolated with an end-of-support policy or omitted.
`v2.0.0-alpha.24 implementation stop reached. Run pentest for this exact commit.`

## 2.0 Beta Releases

No new architecture enters after beta. Betas close APIs, performance,
portability, security, and documentation gaps.

### v2.0.0-beta.1 - Public API Freeze

**Status:** Planned.

**Goal:** Freeze public types, IDs, errors, builders, scenes, sinks, and crate
ownership.

**Deliverables:** Semver audit, rustdoc completion, sealed/internal boundaries,
and explicit stability policy for catalog, scene, pixels, SVG, and encoders.

**Verification:** API snapshot/diff tooling, downstream compile matrix, and
manual ownership review.

**Exit criteria:** Later betas require compatibility-preserving changes or a
documented reset to another alpha. `v2.0.0-beta.1 implementation stop reached.
Run pentest for this exact commit.`

### v2.0.0-beta.2 - Feature And Dependency Freeze

**Status:** Planned.

**Goal:** Freeze default/minimal/optional feature topology and crate graphs.

**Deliverables:** Final facade defaults, mutually exclusive hash policy,
per-crate dependency allowlists, optional backend classifications, duplicate
exceptions, and independent crate version matrix.

**Verification:** Valid feature powerset, forbidden combinations, cargo trees,
deny/audit, MSRV, package size, and default graph assertions.

**Exit criteria:** No optional codec, GPU, schema, or test dependency leaks into
core or minimal facade builds. `v2.0.0-beta.2 implementation stop reached. Run
pentest for this exact commit.`

### v2.0.0-beta.3 - Resource And Performance Contract

**Status:** Planned.

**Goal:** Make memory, CPU, scene, scratch, and backend costs measurable and
bounded.

**Deliverables:** Per-family scene budgets, raster tile/scratch bounds, encoder
capabilities, GPU resource bounds, benchmark baselines, and regression
thresholds.

**Verification:** Minimum/maximum dimensions, worst-case style stacks,
concurrency math, allocation counts, wall-time smoke ceilings, and denial-of-
service review.

**Exit criteria:** Documentation and measured limits agree for every supported
backend. `v2.0.0-beta.3 implementation stop reached. Run pentest for this exact
commit.`

### v2.0.0-beta.4 - Cleanup And Secret-Lifetime Audit

**Status:** Planned.

**Goal:** Close hashavatar-owned cleanup gaps and document external residuals.

**Deliverables:** Allocation-time sanitizing owners, error/unwind cleanup,
optional caller output guards, codec scratch boundaries, GPU zero-fill/fence
policy, and updated threat model.

**Verification:** Normal/error/unwind tests, deliberate writer/device failures,
memory-lifetime review, and security pentest focused on identity and derived
buffers.

**Exit criteria:** Every owned sensitive allocation has an explicit cleanup
owner and honest limits. `v2.0.0-beta.4 implementation stop reached. Run pentest
for this exact commit.`

### v2.0.0-beta.5 - Cross-Platform Determinism

**Status:** Planned.

**Goal:** Prove the advertised scene and pixel contracts across supported
platforms.

**Deliverables:** Native x86_64 and AArch64 runs, WASM execution, multiple
optimization/LTO profiles, optional big-endian supplemental evidence, and
published digest matrices.

**Verification:** Every canonical `SceneDigest` and `PixelDigest` matches; SVG
structural fixtures match; encoded outputs are compared only under admitted
encoder contracts.

**Exit criteria:** No unresolved canonical scene/pixel mismatch remains.
`v2.0.0-beta.5 implementation stop reached. Run pentest for this exact commit.`

### v2.0.0-beta.6 - Downstream Migration Trials

**Status:** Planned.

**Goal:** Validate 2.0 in real consumers before RC.

**Deliverables:** Migrate `hashavatar-website`, at least one minimal no_std/alloc
fixture, one formats consumer, and one GPU example; complete 1.x-to-2.0 guide
and a concise service-construction recipe.

**Verification:** Integration tests, cache migration, style-version rollout,
concurrency/resource behavior, and API ergonomics review.

**Exit criteria:** Real consumers need no undocumented workaround.
`v2.0.0-beta.6 implementation stop reached. Run pentest for this exact commit.`

### v2.0.0-beta.7 - Documentation And Evidence Freeze

**Status:** Planned.

**Goal:** Finish user, author, security, and release documentation before RC.

**Deliverables:** README, crate docs, family-authoring guide, scene contract,
no_std profiles, format/GPU status, security controls, dependency inventory,
SBOM, and limitations.

**Verification:** All examples compile in their documented feature profiles;
links, package contents, and generated evidence are reproducible.

**Exit criteria:** RC work is verification and remediation, not missing
documentation. `v2.0.0-beta.7 implementation stop reached. Run pentest for this
exact commit.`

## 2.0 Release Candidates

### v2.0.0-rc.1 - Formal And Adversarial Verification

**Status:** Planned.

**Goal:** Run the complete assurance program against the frozen API and output
contracts.

**Deliverables:** Mandatory pinned Kani job, defined fuzz CPU-hour campaign,
Miri where meaningful, sanitizer builds for adapters, differential CPU/GPU
tests, codec decode comparisons, dependency review, and full pentest.

**Verification:** Kani cannot silently skip; fuzz targets have no unresolved
crash/hang; security findings are classified with concrete reproductions.

**Exit criteria:** All critical/high findings and actionable correctness gaps are
resolved before promotion. `v2.0.0-rc.1 implementation stop reached. Run
pentest for this exact commit.`

### v2.0.0-rc.2 - Remediation Candidate

**Status:** Planned.

**Goal:** Retest every RC1 change without adding unrelated features.

**Deliverables:** Focused fixes, regression tests, refreshed corpus/SBOM, updated
security controls, and exact finding dispositions.

**Verification:** Full release gate, complete pentest retest, GitHub CI, CodeQL,
downstream integrations, and artifact reproducibility.

**Exit criteria:** No unresolved release-blocking finding remains.
`v2.0.0-rc.2 implementation stop reached. Run pentest for this exact commit.`

### v2.0.0-rc.3 - Exact Release Artifact Candidate

**Status:** Planned.

**Goal:** Produce the exact packages and evidence intended for GA.

**Deliverables:** Final independently versioned crate set, checksummed package
archives, committed semantic SBOMs, provenance, corpus/proof/fuzz summary,
migration guide, and final release notes.

**Verification:** Install packages from local archives in clean downstream
fixtures; tags are absent; package checksums and evidence are reproducible.

**Exit criteria:** GA can be a same-commit tag promotion. Any code, dependency,
metadata, or artifact change requires another RC. `v2.0.0-rc.3 implementation
stop reached. Run pentest for this exact commit.`

Additional `rc.N` releases are added whenever remediation changes the candidate.
Do not force GA because the roadmap happened to name three candidates.

## v2.0.0 - Stable Release

**Status:** Planned.

**Goal:** Publish the first stable canonical-scene release.

**Deliverables:** Publish only workspace crates marked for release in dependency
order; tag the exact approved RC commit; publish immutable release evidence and
the supported-target/backend matrix.

**Verification:** Final readiness gate verifies tag target, package checksums,
SBOMs, provenance, release notes, pentest evidence, GitHub status, and crate
publish order.

**Exit criteria:**

- `hashavatar-core` is the canonical scene/CPU/SVG implementation.
- `hashavatar-formats` isolates codec dependencies.
- `hashavatar-gpu` is optional and clearly canonical or noncanonical per its
  evidence; CPU remains normative for 2.0.0.
- `hashavatar-schema`, if published, remains optional and separate.
- Every first-party Rust file satisfies the 500-line policy.
- Supported feature profiles pass MSRV and pinned stable Rust.
- Scene and raw-pixel contracts match the published corpus.
- No unresolved critical/high security or correctness finding remains.
- Documentation makes no ambiguous bit-identical, zero-allocation,
  zeroization, anonymity, codec, or GPU claim.

`v2.0.0 implementation stop reached. Run final release readiness for this exact
commit before tagging.`

## Explicitly Deferred Beyond 2.0

- Promoting GPU output to canonical without multi-vendor zero-difference proof.
- A general external plugin or third-party art execution system. Stable string
  IDs alone do not create a safe extension mechanism.
- HTTP query parsing or service middleware in this workspace.
- Custom hand-written SIMD in first-party crates.
- Async rendering APIs in core.
- Unbounded scene graphs, arbitrary SVG, user-provided shaders, or external
  image loading.
- New avatar families added merely to fill a release; catalog additions can
  resume after the 2.0 contract is stable.

## Critical Path

The critical path is:

```text
policy and modularity
  -> frozen catalog/contracts
  -> fixed numeric core
  -> validated scene
  -> canonical CPU rasterizer
  -> Cat vertical slice
  -> all family/background ports
  -> SVG parity
  -> layered rig/layout
  -> portable core and formats isolation
  -> facade freeze
  -> cross-platform assurance
  -> RC remediation
  -> 2.0.0
```

AVIF, JPEG XL, GPU, schema generation, and a legacy compatibility crate each
have explicit decision releases, but rejection or deferral of any one of them
must not weaken or block the canonical CPU 2.0 release.
