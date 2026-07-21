# hashavatar Plan Towards 2.0.0

Status: accepted direction, 1.x preparation complete

Current stable line: `1.3.x` on the `release/1.3` maintenance branch

Target: a polished, security-oriented Rust rendering workspace with a canonical
safe-Rust CPU renderer, SVG generated from the same private fixed-point scene,
established raster formats plus AVIF, heapless rendering, a bounded schema
adapter, and an optional GPU backend behind focused crate boundaries.

This plan incorporates the architecture, contracts, and public API recommended
in `docs/archive/GAP_ANALYSIS_2.0.md`, but supersedes its recommendation to
defer GPU, AVIF, schema, and heapless storage. Those four components are now
explicit 2.0 requirements. It also supersedes the broader sequence in
`docs/archive/2.0-idea.md`; both analyses remain useful background, while this
document defines release scope and finish lines.

The versions below are not a maximum or a demand to combine work. Add a patch
release or split an alpha/beta before implementation whenever one milestone is
no longer small enough to review, test, pentest, and stop cleanly.

## Product Decision

Hashavatar 2.0 is a Rust library API, not an HTTP service framework.

The 2.0 critical path contains:

1. The `hashavatar` facade crate.
2. The canonical CPU and SVG implementation in `hashavatar-core`.
3. Established formats and admitted AVIF support in `hashavatar-formats`.
4. A safe caller-storage profile through `hashavatar-heapless`.
5. A bounded transport-neutral request document in `hashavatar-schema`.
6. An explicit optional GPU backend in `hashavatar-gpu`.
7. The contracts, tests, migration work, and security evidence required to
   support every published crate.

The companion crates remain optional for downstream users and absent from the
default dependency graph. They are nevertheless required release deliverables:
Hashavatar `2.0.0` is not complete until all four pass their finish lines.

External scene authoring and third-party avatar plugins remain deferred.

## Why 2.0 Exists

The `1.x` renderer has strong bounded-input and security controls, but raster
and SVG are separate artistic implementations. It also relies on floating-point
geometry, mutable RNG state, direct `image::RgbaImage` coupling, and mappings
whose meaning can drift when enum lists change.

The major release is justified by these linked changes:

1. One validated private scene becomes the source of all family geometry.
2. Fixed-point math and a canonical CPU rasterizer define raw output pixels.
3. SVG is emitted from the same scene rather than separate family renderers.
4. Trait derivation, catalogs, contracts, and asset keys become versioned.
5. Unsupported explicit style combinations return typed errors.
6. Request preparation binds validation, resolved style, resource estimates,
   rendering, and cache-key derivation to one immutable request tuple.
7. Codecs move out of the portable rendering core.
8. Heapless, schema, GPU, and advanced codec support gain explicit boundaries
   instead of leaking optional dependencies into the canonical core.

`2.0` does not promise pixel compatibility with `1.x`. It establishes a
stronger contract that later `2.x` releases must preserve or version explicitly.

## Supported Public Rust API

The facade must make normal applications easy to write without exposing web or
renderer internals. The intended public vocabulary is:

- `AvatarIdentity` and bounded namespace/keyed identity construction;
- `AvatarRequest` and `AvatarRequestBuilder`;
- `PreparedAvatar`;
- `ResolvedStyle` and `LayoutReport`;
- `ResourceBudget`;
- `CatalogVersion` and `RenderContractId`;
- `IdentityCacheKey`, `AvatarAssetKey`, `SemanticEncodedAssetKey`, and
  `BuildEncodedAssetKey`;
- `RasterSurfaceMut` and an explicit external pixel format;
- `render_into` and convenience owned-image rendering;
- `write_svg` with document/fragment options;
- writer-based and convenience encoding through the formats boundary;
- a caller-storage profile that reports exact capacity failures without hidden
  allocation;
- strict conversion from the optional versioned schema document;
- explicit GPU capability/status reporting and caller-selected CPU fallback;
- typed identity, request, capability, capacity, surface, render, writer, and
  encoding/backend errors.

The primary workflow should be concise:

```rust,ignore
let prepared = AvatarRequest::builder(identity)
    .spec(spec)
    .style(style)
    .prepare()?;

let key = prepared.asset_key();
let layout = prepared.layout_report();
let budget = prepared.resource_budget();

prepared.render_into(&mut surface)?;
prepared.write_svg(&mut svg_writer, SvgOptions::document())?;
```

Encoding is supplied by the formats boundary:

```rust,ignore
hashavatar::formats::encode_to_writer(
    &prepared,
    AvatarOutputFormat::WebP,
    &mut output,
)?;
```

Convenience APIs may allocate an owned image or `Vec<u8>`, but writer and
caller-surface APIs are first-class. Documentation must state where an
underlying codec still allocates internally.

### Prepared Request Invariant

`PreparedAvatar` is the main safety and correctness boundary. Preparation must:

- validate identity, namespace, dimensions, seed, contract, catalog, and style;
- resolve automatic traits and all compatibility/fallback decisions;
- produce immutable `ResolvedStyle` and `LayoutReport` values;
- compute conservative CPU, scene, scratch, and output resource estimates;
- derive canonical identity and avatar asset keys from the same validated tuple;
- retain no raw identity input longer than the documented identity protocol
  requires;
- redact identity-bearing fields in `Debug` output;
- complete transactionally without publishing a partially prepared value.

A prepared value must not permit callers to change fields that affect pixels or
keys. This prevents applications from validating one tuple, caching another,
and rendering a third.

### Format Result

The formats crate may return an `EncodedAvatar` for allocating APIs and
completion metadata for writer APIs. It should expose:

- media type and conventional extension;
- output format and encoder contract/version;
- alpha support and relevant capability metadata;
- a typed semantic or build-bound encoded asset key derived from the prepared
  asset key and exact encoder
  settings;
- conservative scratch-memory information where the codec permits it;
- documented cleanup limits for codec-owned buffers.

HTTP ETag quoting, cache-control policy, CDN paths, and object-store key layout
remain application policy. The library supplies stable key bytes and optional
text formatting, not HTTP behavior.

## Explicitly Excluded Service Concerns

`hashavatar-website` is the hosted `hashavatar.app` reference implementation.
It demonstrates the library in production but is not part of the reusable API.

Hashavatar crates must not own:

- Axum or another HTTP framework;
- Tokio or another async runtime;
- query-string or route parsing;
- authentication or authorization;
- rate limiting, concurrency permits, timeouts, or cancellation policy;
- trusted-proxy, CORS, CSP, or other browser/HTTP security policy;
- S3, object storage, persistence, redirects, or signed URLs;
- CDN/cache-control policy;
- observability and telemetry;
- locale, website pages, or deployment configuration;
- caller-specific identity normalization.

The reference service recipe is:

1. Parse and bound transport input under application-specific limits.
2. Construct `AvatarIdentity` and `AvatarRequest`.
3. Call `prepare()` before starting expensive work.
4. Use `ResourceBudget` to acquire an application-owned concurrency permit.
5. Render away from asynchronous runtime worker threads.
6. Encode through `hashavatar-formats`.
7. Build HTTP and storage metadata from canonical asset keys.

## Workspace Boundary

The release-critical workspace has a one-way dependency graph:

```text
hashavatar (facade)
  |-- hashavatar-core
  |-- hashavatar-formats  --> hashavatar-core
  |-- hashavatar-heapless --> hashavatar-core
  |-- hashavatar-schema   --> hashavatar-core
  `-- hashavatar-gpu      --> hashavatar-core

integration tests/testkit --> facade and every companion crate
```

The primary package remains `hashavatar` on crates.io.

### `hashavatar-core`

Published, `no_std` capable with `alloc` and caller-storage profiles, and
responsible for:

- identity and namespace derivation;
- optional keyed pseudonymization if admitted during 1.x;
- catalog, render-contract, trait-derivation, and key protocols;
- request preparation, style resolution, family rigs, and capabilities;
- private fixed-point arithmetic, scene construction, and validation;
- canonical safe-Rust CPU rendering into caller-provided memory;
- streaming SVG document/fragment writing;
- resource budgets and typed errors.

It must not depend on `image`, codecs, GPU libraries, Serde, JSON, web
frameworks, async runtimes, filesystems, clocks, OS entropy, or network clients.

### `hashavatar-formats`

Published, `std`-oriented, and responsible for:

- `image` compatibility adapters;
- WebP, PNG, JPEG, GIF, and admitted AVIF format features;
- writer and allocating convenience APIs;
- encoder contracts and encoded asset keys;
- codec capability and resource metadata;
- decode-and-compare tests against canonical core pixels;
- accurate documentation of codec-owned scratch and cleanup limitations.

WebP may remain the facade default for compatibility. Other formats stay
explicit opt-ins. Once admitted, AVIF joins `all-formats` but never the default
feature set. JPEG XL is not planned; it may be reconsidered from current
evidence in a future minor release.

### `hashavatar-heapless`

Published, `no_std` and no-allocator capable, and responsible for ergonomic
safe caller-owned scene/scratch storage adapters, capacity types, and
compile-time or runtime storage sizing. It must not expose scene authoring,
silently allocate, or require first-party unsafe code.

### `hashavatar-schema`

Published, optional, and responsible for a versioned bounded request document,
strict conversion into core `AvatarRequest`, and optional Serde/JSON Schema
integration. It does not contain HTTP routes, query parsing, OpenAPI endpoint
policy, authentication, persistence, telemetry, or service configuration.

### `hashavatar-gpu`

Published, optional, and never enabled by default. It consumes a narrow,
versioned, read-only backend protocol from validated core scenes. It owns GPU
device/queue integration, capability negotiation, bounded resources, device
failure handling, and differential evidence. CPU fallback is caller-selected
and never silent.

### Test Infrastructure

Shared test utilities are non-production workspace members or integration-test
support. Dependency direction must avoid cycles:

- generic fixture/testkit code may depend on core contracts;
- top-level integration tests may depend on the facade, every companion crate,
  and testkit;
- production crates must not dev-depend on a testkit that imports them back;
- no test-only crate is re-exported or published accidentally.

### Legacy Compatibility Decision

Demand for exact 1.x rendering must be measured by `v1.3.0`, before the legacy
implementation is removed. Choose one outcome explicitly:

1. Users requiring old pixels pin the supported 1.x crate.
2. A separately maintained `hashavatar-compat-v1` crate freezes the old engine
   with a documented support end.

The normal 2.0 facade must not carry both engines indefinitely.

## Private Scene Boundary

The scene is an implementation mechanism, not a general graphics API.

For 2.0:

- command, path, paint, arena, and transform layouts remain private;
- fixed-point numeric types remain private unless a demonstrated public use
  requires one;
- no public scene deserialization or arbitrary scene authoring is provided;
- no arbitrary SVG fragments, external images, filters, shaders, or unbounded
  recursion enter the scene;
- `hashavatar-gpu` receives only a narrow, versioned, read-only backend protocol
  over an opaque validated scene;
- the backend protocol exposes semantic commands and bounded iteration needed
  for execution, but not arena layouts, mutable authoring, or deserialization;
- public inspection is limited to stable bounds, budgets, capability metadata,
  `SceneDigest`, and the documented backend protocol;
- canonical scene serialization remains internal and one-way.

This preserves freedom to improve internal arena layouts throughout 2.x while
keeping visible output, documented digests, and the narrow GPU protocol stable.

## Required Contracts

The following contracts must be written and tested before beta. Their values
cannot remain implicit in implementation code.

### Pixel Contract

`PIXEL_CONTRACT.md` must freeze:

- external output as straight-alpha RGBA8 in sRGB channel order;
- private premultiplied representation used during compositing;
- top-left row origin and the exact pixel-center/sample grid;
- exact conversion and rounding from internal compositing to external RGBA;
- canonical transparent pixels as `[0, 0, 0, 0]`;
- checked stride and buffer-length validation;
- preservation of caller-owned stride padding;
- `PixelDigest` over dimensions, pixel contract ID, and tightly packed visible
  rows only, excluding stride padding;
- rejection semantics for empty, zero-sized, or unsupported surfaces.

If another pixel representation is ever exposed, it receives a distinct type
and contract ID rather than an ambiguous boolean flag.

### Digest And Key Contracts

Each identity, scene, pixel, avatar-asset, and encoded-asset digest/key must
freeze independently:

- algorithm or PRF and protocol label;
- protocol version and algorithm ID;
- ordered fields and inclusion/exclusion rules;
- length-prefix width and integer endianness;
- output/truncation length and text encoding;
- handling of dimensions, seed, catalog, render contract, resolved style,
  backend contract, format, encoder version, and settings;
- whether metadata, accessibility text, scene IDs, and stride padding
  participate.

A 2.0 request must never reuse a 1.x cache key for different pixels. Raw
identity digests must not become cache keys merely for convenience.

### Stateless Trait Derivation

The derivation contract must freeze:

- the PRF/XOF or hash construction;
- domain and trait-label encoding;
- counter encoding and endianness;
- seed handling;
- range selection and rejection sampling or explicitly accepted modulo bias;
- independence between traits;
- catalog weighting and fallback behavior;
- behavior when later catalogs are introduced.

Selecting SHA-512, BLAKE3, or XXH3 at build time must not silently claim the
same identity/visual protocol. The active algorithm ID is bound into the
identity and derivation contracts. XXH3 remains explicit, non-cryptographic,
and unsuitable for adversarial or sensitive identifiers.

### Failure And Partial-Output Contract

Every sink and surface API must state:

- which validation completes before output is touched;
- whether a runtime failure may leave a partially modified raster surface;
- that writer failures may leave a prefix that is not a complete SVG/image;
- whether retry requires a fresh destination;
- whether a capacity error reports a required size or only a safe lower bound;
- cleanup behavior after validation, capacity, render, writer, and codec errors;
- which errors are deterministic for the same prepared request.

Strong exception safety must not be claimed where it would require an
undocumented second full-size image allocation.

### SVG Contract

The SVG contract must define:

- complete-document and embeddable-fragment modes;
- collision-free deterministic IDs for clips and gradients;
- validated caller-supplied or scene-derived ID prefixes;
- XML escaping for all caller-provided accessibility text;
- title/description policy and whether metadata affects encoded asset keys;
- locale-independent fixed-point numeric formatting;
- stable element/attribute ordering only where explicitly promised;
- unsupported scene-capability errors;
- the distinction between scene-semantic parity and browser pixel parity.

SVG derives from the canonical scene, but browser rasterization is not promised
to match the reference CPU renderer byte-for-byte.

### Canonical Execution Contract

Canonical raw pixels must be independent of:

- thread count and tile scheduling;
- iteration order of hash maps or other unordered containers;
- CPU features and runtime dispatch;
- optimization level, LTO, and debug/release mode;
- supported target architecture.

The initial reference backend is single-threaded unless tiles are proven fully
independent and produce the same compositing order.

### Migration And Default Contract

The migration guide must define changes to:

- `AVATAR_STYLE_VERSION` and namespace defaults;
- default catalog and render-contract selection;
- default family/style mapping;
- identity, avatar, and encoded cache keys;
- CDN/object cache migration;
- callers that omit explicit contract/catalog IDs;
- 1.x output retention or deliberate non-retention.

Defaults must be named constants with tests. A new default may select a new
versioned contract; it may not mutate an old contract in place.

### Heapless Storage Contract

The heapless profile must define:

- all caller-provided scene, command, point, paint, transform, clip, raster, and
  scratch capacities;
- exact or conservative required-capacity reporting for every failure;
- whether failure occurs before a surface or sink is modified;
- per-family and worst-case style storage budgets;
- absence of hidden allocation, filesystem, clock, entropy, thread, and OS-I/O
  dependencies;
- safe preinitialized storage without first-party unsafe code;
- behavior on reuse after success, error, and unwinding.

### Schema Contract

The schema crate must freeze:

- a versioned `AvatarRequestDocumentV1` containing rendering fields only;
- per-field length/range bounds and strict enum/ID handling;
- unknown, duplicate, missing, and conflicting field behavior;
- conversion into core `AvatarRequest` without a second interpretation path;
- Serde/JSON Schema feature and MSRV boundaries;
- compatibility rules for adding fields or document versions.

The schema is transport-neutral. HTTP status codes, routes, authentication,
query syntax, OpenAPI operations, and service policy remain application-owned.

### GPU Backend Contract

The GPU crate and core backend protocol must define:

- supported scene capability negotiation and typed unsupported errors;
- explicit device, queue, adapter, format, and backend selection;
- resource ceilings and checked buffer/dispatch arithmetic;
- deterministic command order and no silent CPU fallback;
- device-loss, submission, mapping, timeout, and partial-output behavior;
- buffer initialization, completion-fence reuse, and cleanup limitations;
- whether output is canonical or visually conforming only.

GPU output may ship as noncanonical in 2.0. The canonical CPU `PixelDigest`
remains normative unless a declared multi-vendor matrix proves exact equality.

### AVIF Contract

AVIF support must define encoder provider/version, quality and speed settings,
alpha/color handling, metadata policy, resource estimates, writer behavior,
lossy comparison tolerances, and codec-owned cleanup limits. Encoded bytes are
stable only under an explicitly frozen encoder contract; decoding to acceptable
pixels does not make bytes stable across dependency upgrades.

## Numeric And Rendering Rules

- Use a documented signed fixed-point coordinate representation with wider
  checked intermediates.
- Freeze division, remainder, tie-breaking, conversion, and rounding behavior.
- Reject invalid construction at trust boundaries. Reserve saturation for
  explicitly documented clipping and resource estimates.
- Use exact integer Porter-Duff source-over compositing.
- Lower curves with a bounded deterministic algorithm and explicit maximum
  work/depth.
- Validate command/path ranges, stack balance, transform and clip depth,
  coordinate bounds, command count, path complexity, and estimated raster cost
  before execution.
- Use checked `usize` resource and index math, including on 32-bit targets.
- Never execute an unvalidated scene.

## Family And Style Rules

- Use optional semantic anchors rather than assuming human anatomy.
- Model focused face, eye, head, and body capabilities only where supported.
- Calibrate family-specific slot transforms and exclusion zones.
- Use typed slots for back, aura, headwear, earwear, facewear, eyewear,
  neckwear, handheld, and foreground layers where admitted.
- Canonicalize stacks by slot and stable accessory ID, never insertion order.
- Explicit unsupported styles return typed errors.
- Automatic styles use a frozen fallback policy.
- `LayoutReport` records accepted, adjusted, rejected, and substituted layers.
- Do not add new avatar families merely to fill the 2.0 schedule. Port and
  stabilize the existing catalog first; new art can resume after 2.0.

## Security And Privacy Rules

- First-party production crates use `#![forbid(unsafe_code)]`.
- Use pseudonymization, never anonymity, in security claims.
- Keyed identity support uses a reviewed standard construction with protocol
  labels, versions, domains, and key IDs.
- Secret/key types have redacted `Debug`, bounded input, deliberate clone
  policy, no accidental serialization, and sanitization on drop.
- Identifier normalization remains caller policy.
- Hashavatar-owned sensitive buffers receive cleanup owners before data is
  written, covering normal return, errors, and unwinding where Rust permits.
- Process abort, registers, compiler copies, paging, crash dumps, allocator and
  codec internals, and hardware/driver memory remain documented residuals.
- Caller-owned final images and encoded buffers remain caller responsibility.
- Rendering remains variable-time. Service concurrency, latency padding,
  caching, and rate limiting remain application responsibilities.
- Every public untrusted-input path has bounded allocation/work, typed errors,
  panic-policy tests, and focused fuzz targets.

## Modularity And Engineering Policy

- Use Cargo resolver `3`, workspace package metadata, centralized dependency
  versions, and `default-members` containing only the facade once the workspace
  split exists.
- Apply workspace Rust lints equivalent to `eth`: forbid unsafe code, deny
  unused results and missing documentation, and apply strict Clippy policy for
  panic, unwrap, expect, undocumented unsafe, indexing/slicing, arithmetic side
  effects, truncating casts, and sign-loss casts.
- Keep release overflow checks enabled. Use `panic = "abort"` for project-owned
  release artifacts while documenting that final profile selection for a Rust
  library dependency belongs to the consuming binary.
- Keep `lib.rs` focused on crate docs, module wiring, and public re-exports.
- Split by responsibility and dependency direction, not arbitrary line ranges.
- Every non-generated first-party Rust source file, including tests, examples,
  benches, and fuzz targets, must stay under 500 lines; CI fails closed on an
  over-limit file.
- Review modules approaching 300 lines during new development.
- Enforce architecture with dependency-direction tests, strict lints,
  complexity review, panic/unsafe policy, and public API snapshots rather than
  relying on line count alone.
- Do not spend 1.x releases splitting code that the 2.0 scene port will delete.

## Dependency And Release Policy

- Use reviewed, MSRV-compatible crate and tool versions. Every version-specific
  release gate runs a mandatory networked freshness check for stable Rust,
  required Cargo tools, dependencies under review, and GitHub Actions.
- Normal commit CI may avoid live network checks. A release gate fails closed
  when required version metadata is unavailable or a reviewed pin is stale.
- Pin every GitHub Action to a full immutable commit SHA with its release tag in
  a comment, while keeping CodeQL on GitHub default setup.
- Review complete feature graphs, licenses, advisories, maintenance history,
  unsafe/FFI boundaries, platform defaults, package size, and resource behavior.
- Run formatting, Clippy, tests, docs, MSRV, pinned stable, deny, audit, SBOM,
  package, reproducibility, fuzz-build, and policy gates at every release stop.
- Commit a semantic SPDX SBOM and compare stable package, version, license,
  checksum, reference, and relationship fields against fresh generation.
- Give every version a version-specific local verification command, release
  notes under `release-notes/`, explicit known limitations, dependency
  evidence, and updated threat/security documentation where behavior changes.
- Use cargo-semver-checks or equivalent API snapshots after beta.1.
- Test downstream consumers from packaged archives, not only workspace paths.
- Test valid feature combinations and duplicate public companion-crate versions.
- Preserve temporary detailed `PENTEST.md` only while triaging findings; remove
  it after disposition and retain sanitized exact-commit evidence where needed.
- Track independently versioned workspace packages in `release-crates.toml` and
  `docs/CRATE_VERSION_MATRIX.md` as `code`, `dependency`, `metadata`, or
  `unchanged`; publish only changed crates in dependency order.
- Validate the release matrix against Cargo metadata and reject accidental
  lockstep publication or duplicate public dependency identities.
- Validate this roadmap mechanically for release order, required milestone
  sections, and exact pentest stop sentences. Give release, SBOM, dependency,
  and policy scripts deliberate failing-fixture self-tests rather than testing
  only their successful path.

Every prerelease milestone and stable release requires exact-commit pentest
evidence. Record both the previous implementation stop and candidate stop so
review covers `<base>..<candidate>` plus the candidate's complete tree. The
permanent `security/pentest/vX.Y.Z.md` report records `Status: PASS`, the full
`Reviewed-Range`, `Reviewed-Commit`, tester, scope, and date. A report-only
commit may change only that report. Root `PENTEST.md` remains temporary and
must never be committed.

Each release stop must end with:

```text
vX.Y.Z implementation stop reached. Run pentest for this exact commit.
```

No prerelease tag is created. After a clean retest, GitHub CI and CodeQL default
setup must be green for the report commit. A milestone readiness gate verifies
metadata, release notes, semantic SBOM, packages, checksums, pentest evidence,
and publish order before work begins on the next milestone. Only the approved
stable release receives a signed tag.

## Completeness Review Register

Review this table whenever the roadmap or a pentest implies new work. A gap on
the 2.0 critical path must be assigned to a release before implementation moves
past its dependency point.

| Gap | Assigned resolution |
| --- | --- |
| Raster and SVG use separate family geometry. | Prove one-scene Cat in `alpha.1`, complete the shared renderer in `alpha.2`, and port the catalog in `alpha.3`. |
| Floating-point geometry prevents a clear cross-platform contract. | Private fixed math begins in `alpha.1`; numeric, compositing, and execution contracts finish in `alpha.2`. |
| Mutable RNG and enum-list growth can change visual traits silently. | Freeze legacy IDs in `v1.2.0` and activate versioned stateless derivation in `alpha.1`. |
| Unsupported style combinations can be silently skipped. | Add opt-in strict validation in `v1.2.0` and freeze typed layered resolution in `alpha.4`. |
| Website integrations duplicate request preparation, resource math, and cache keys. | Preview `PreparedAvatar` in `v1.3.0`; complete the facade and packaged website trial in `alpha.5` and `beta.2`. |
| External RGBA, digest, derivation, SVG, and partial-output behavior are underspecified. | Write and test the contracts in `alpha.2`; freeze them in `beta.1`. |
| Codecs enlarge and constrain the rendering core. | Isolate established formats in `hashavatar-formats` at `alpha.5` and admit AVIF there at `alpha.8`. |
| Existing 1.x cache keys could collide with different 2.0 pixels. | Introduce complete keys in `v1.2.0`; freeze migration/default vectors in `beta.1`. |
| Resource/index behavior needs 32-bit evidence. | Exercise 32-bit targets from `alpha.1` through heapless `alpha.6` and publish the final matrix in `beta.2`. |
| Public API drift and workspace duplicate types need automated detection. | Add API snapshots, semver checks, feature powersets, and packaged downstream tests in `beta.1`. |
| Exact 1.x pixels may be required after the rewrite. | Decide pinning versus a separate compatibility crate by `v1.3.0`, before deleting legacy code. |
| Heapless support could freeze unnecessary scene internals or hide allocation. | Add a safe caller-storage adapter and no-allocation evidence in `alpha.6` without exposing scene authoring. |
| Service consumers need a shared request document without importing HTTP policy. | Add the bounded transport-neutral schema crate in `alpha.7`. |
| AVIF adds a large codec/provider boundary. | Admit it fail-closed in `alpha.8`; no provider is accepted without license, MSRV, resource, conformance, and security evidence. |
| GPU support requires scene access without making the scene a public graphics API. | Freeze a narrow read-only backend protocol in `alpha.9` and complete optional noncanonical execution in `alpha.10`. |
| JPEG XL currently adds unnecessary license/provider uncertainty. | Leave it unplanned; reconsider only in a future minor release from current evidence. |
| Release evidence could drift from the reviewed commit or package graph. | Use exact commit ranges, report-only pentest commits, semantic SBOM comparison, package-archive tests, release matrices, and milestone readiness gates. |

## 1.x Preparation Releases

All 1.x work is additive. Existing explicit outputs remain unchanged unless a
separate security/correctness fix requires a documented fingerprint change.
Private fixed-point, scene, and new renderer prototypes belong on the 2.0 alpha
branch, not in published 1.x packages.

### v1.1.3 - Policy Corrections

**Status:** Released as `v1.1.3`.

**Goal:** Make current documentation and checks accurately describe 1.1.x.

**Deliverables:** Correct enum semver guidance and the Kani harness count;
remove stale or duplicated roadmap claims; identify this document as the
accepted 2.0 direction; harden encoded-output cleanup, custom-renderer bounds,
SVG definition isolation, and fail-closed release evidence.

**Verification:** Documentation links, Kani inventory, package contents,
formatting, and release metadata.

**Exit criteria:** No code or visual fingerprint changes; current API and
security claims are internally consistent.

`v1.1.3 implementation stop reached. Run pentest for this exact commit.`

### v1.2.0 - Contracts And Strict Preparation

**Status:** Released as `v1.2.0`.

**Goal:** Give 1.x consumers the identifiers and validation path needed for a
safe 2.0 migration.

**Deliverables:** Add explicit legacy `CatalogVersion` and `RenderContractId`,
stable built-in IDs and weights, complete identity/avatar/encoded asset keys,
family capability manifests, opt-in strict style validation, and a keyed
identity admission decision. If keyed identity is accepted, implement it behind
an explicit feature using a reviewed standard construction.

**Verification:** Frozen mapping vectors, key domain-separation and cache-bust
tests, complete style compatibility matrix, unchanged legacy fingerprints, and
known-answer/cleanup tests for keyed identity if implemented.

**Exit criteria:** New catalogs cannot reshuffle the legacy mapping; downstreams
can detect unsupported styles without changing legacy skip behavior. A keyed
identity implementation also receives focused cryptographic review.
`v1.2.0 implementation stop reached. Run pentest for this exact commit.`

### v1.3.0 - Migration API And Corpus

**Status:** Released as `v1.3.0`. The supported maintenance line lives on
`release/1.3`; only serious security and correctness fixes are backported.

**Goal:** Let real applications exercise the future workflow before the 2.0
engine replaces rendering internals.

**Deliverables:** Add an additive `AvatarRequest`/builder and
`PreparedAvatar` preview, `ResolvedStyle`, `LayoutReport`, `ResourceBudget`,
writer-based SVG/encoding APIs where they genuinely help, validated raster
surface adaptation, complete 1.x compatibility corpus, migration/deprecation
guide, compat-v1 demand decision, and only the source splits necessary to own
these APIs coherently.

**Verification:** Short-write and writer-failure tests, stride/capacity tests,
public API diff, packaged downstream trial with `hashavatar-website`, complete
request/style/pixel/SVG/key fixtures, and unchanged existing output helpers.

**Exit criteria:** A downstream can migrate request construction and cache-key
logic before accepting 2.0 pixel changes; exact 1.x compatibility has an
explicit keep-or-pin decision.

**Compatibility decision:** Exact 1.x requests, styles, keys, RGBA output, and
SVG output are frozen in the 1.3 corpus. Applications requiring those pixels
after 2.0 should pin 1.3. A separate compatibility crate is deferred unless
real downstream demand justifies maintaining and auditing two renderers.

`v1.3.0 implementation stop reached. Run pentest for this exact commit.`

## 2.0 Alpha Releases

Alpha APIs and pixels may change. Each alpha must compile examples, document
its current limitations, and pass the repository's local and GitHub gates. The
sequence contains release-critical work only. Alpha, beta, and release-
candidate milestones use named implementation-stop commits for local downstream
testing; they are not tagged or published to crates.io. Tags and crates.io
publication begin with the approved stable `2.0.0` workspace.

### v2.0.0-alpha.1 - Cat Vertical Slice And Workspace

**Status:** Implementation complete; pending exact-commit pentest, downstream
website validation, and green GitHub checks before beginning alpha.2.

**Goal:** Prove the complete architecture with real artwork before generalizing
the rendering abstraction.

**Deliverables:** Create the minimal facade/core workspace boundary; implement
private fixed-point math, stateless trait derivation, a minimal validated scene,
Cat rig/compiler, themed background, canonical CPU raster output, and SVG from
that same scene. Keep scene and numeric layouts private.

**Verification:** Cat visual review; known-answer trait vectors; malformed scene
tests; raw pixel equality on x86_64, AArch64, WASM, debug/release, and a 32-bit
`usize` target where CI permits; parsed SVG semantic checks; bounded work and
allocation evidence.

**Exit criteria:** One request produces Cat pixels and SVG from one scene with
no independent family geometry path. The abstractions remain changeable.

`v2.0.0-alpha.1 implementation stop reached. Run pentest for this exact commit.`

### v2.0.0-alpha.2 - Complete Canonical Renderer

**Status:** Planned.

**Goal:** Generalize only the scene operations demonstrated by real avatar
needs and finish the canonical CPU/SVG contracts.

**Deliverables:** Add required rectangles, ellipses, lines, bounded paths,
integer curve lowering, fill/stroke rules, clips, opacity groups, gradients,
exact compositing, scene validation and resource estimates, caller-provided
surface execution, SVG document/fragment emission, and the pixel, digest,
derivation, failure, SVG, and canonical execution specifications.

**Verification:** Mandatory Kani for fixed math/index/compositing bounds;
primitive and path goldens; validator/raster/SVG fuzz targets; adversarial stack,
range, curve, stride, and writer inputs; locale-independent output; platform and
optimization digest matrix.

**Exit criteria:** Every operation needed by current artwork has a bounded,
validated, float-free canonical implementation. This first complete canonical
renderer receives focused adversarial review.

`v2.0.0-alpha.2 implementation stop reached. Run pentest for this exact commit.`

### v2.0.0-alpha.3 - Existing Catalog Port

**Status:** Planned.

**Goal:** Port all existing 1.x families, backgrounds, and frames without
expanding the art catalog.

**Deliverables:** Family-specific semantic rigs and scene compilers for every
current family; all themed, fixed, transparent, patterned, gradient, and starry
backgrounds; all current frame shapes; capability manifests; canonical corpus.

**Verification:** Every family at minimum/default/maximum dimensions, all
background/frame combinations, raster/SVG scene parity, cross-platform pixel
digests, unsupported-capability tests, resource ceilings, and visual review.

**Exit criteria:** Every existing family/background/frame uses only the canonical
scene; old family raster and SVG implementations can be removed or isolated by
the prior compatibility decision.

`v2.0.0-alpha.3 implementation stop reached. Run pentest for this exact commit.`

### v2.0.0-alpha.4 - Layered Style And Layout

**Status:** Planned.

**Goal:** Complete secure, deterministic style composition.

**Deliverables:** Typed accessory slots, bounded multi-accessory stacks,
per-family anchors/transforms/exclusion zones, deterministic ordering,
collision/fallback policy, expressions, integer palettes and color roles, and
complete `ResolvedStyle`/`LayoutReport` behavior.

**Verification:** Full capability matrix, permutation invariance, invalid and
capacity-exhaustion tests, collision/fallback fuzzing, every-family stress
fixtures, color/compositing proofs, raster/SVG parity, and visual review.

**Exit criteria:** Supported layers compose predictably; explicit unsupported
requests fail with typed errors; automatic substitutions are frozen and
reported.

`v2.0.0-alpha.4 implementation stop reached. Run pentest for this exact commit.`

### v2.0.0-alpha.5 - Portable Core, Formats Baseline, And Facade

**Status:** Planned.

**Goal:** Assemble the portable core, established formats baseline, and facade
before adding the remaining isolated 2.0 components.

**Deliverables:** Finalize the `no_std + alloc` core, caller RGBA surfaces and
reusable scratch, owned convenience output, `hashavatar-formats` with admitted
WebP/PNG/JPEG/GIF features, the `hashavatar` facade, full recommended request
API, encoded asset metadata/keys, typed error ownership, and 1.x migration map.

**Verification:** Minimal/default/all-established-format feature matrices;
MSRV, WASM, AArch64, and 32-bit checks; per-feature dependency trees; package
contents; codec decode-to-canonical-pixel comparisons; writer/error/cleanup
tests; packaged downstream use from `hashavatar-website`.

**Exit criteria:** Canonical CPU, SVG, and established-format workflows are
possible through the facade; core has no codec/std leakage; the remaining
companions have stable boundaries to build on without changing canonical
pixels.

`v2.0.0-alpha.5 implementation stop reached. Run pentest for this exact commit.`

### v2.0.0-alpha.6 - Heapless Caller Storage

**Status:** Planned.

**Goal:** Support deterministic rendering without a global allocator while
keeping scene authoring and storage layouts controlled by core.

**Deliverables:** Add the `hashavatar-heapless` crate, make core's `alloc`
profile optional where required, provide safe preinitialized caller storage for
scene/scratch/raster work, publish per-family and worst-case capacity budgets,
and return typed capacity errors without hidden allocation or first-party unsafe
code.

**Verification:** `--no-default-features` and no-allocator builds; representative
embedded and WASM32 targets; allocation instrumentation; every family and
maximum style stack at exact/undersized/oversized capacities; reuse after
success/error/unwind; fuzzed capacity descriptors; Kani range/index proofs.

**Exit criteria:** Every canonical CPU fixture can render through documented
caller-owned storage with zero allocator calls, deterministic capacity failure,
and unchanged visible pixels.

`v2.0.0-alpha.6 implementation stop reached. Run pentest for this exact commit.`

### v2.0.0-alpha.7 - Versioned Request Schema

**Status:** Planned.

**Goal:** Provide reusable service-building data contracts without importing
HTTP or deployment policy into Hashavatar.

**Deliverables:** Add `hashavatar-schema` with bounded
`AvatarRequestDocumentV1`, strict conversion into core `AvatarRequest`, optional
Serde and JSON Schema features, stable catalog/render/style IDs, compatibility
rules, and examples for direct Rust and website integration.

**Verification:** Schema snapshots; unknown, duplicate, missing, conflicting,
oversized, and malformed field tests; bounded deserialization visitors where
supported; feature/dependency isolation; MSRV; packaged trials in
`hashavatar-website` and a minimal independent service fixture.

**Exit criteria:** Consumers share one versioned rendering document while HTTP
routes, query syntax, OpenAPI operations, authentication, storage, and service
policy remain application-owned.

`v2.0.0-alpha.7 implementation stop reached. Run pentest for this exact commit.`

### v2.0.0-alpha.8 - AVIF Format Admission

**Status:** Planned.

**Goal:** Ship AVIF as a securely admitted, explicit output feature.

**Deliverables:** Re-evaluate current AVIF providers and select one compatible
with license/MSRV/security policy; add the optional `avif` feature, writer and
convenience APIs, encoder settings contract, encoded asset keys, capability and
resource metadata, and codec-owned cleanup documentation. Add AVIF to
`all-formats` only after admission; never enable it by default.

**Verification:** Complete transitive/default-feature/unsafe/assembly/threading
review; deny/audit/SBOM; representative alpha and color corpus; encode/decode
and lossy-tolerance tests; malformed writer/error behavior; peak memory and CPU
benchmarks; WASM/platform matrix; package and compile-cost evidence.

**Exit criteria:** An admitted AVIF provider passes every gate and works through
the same prepared request and encoded-key APIs. If no provider qualifies, 2.0
waits rather than weakening policy or shipping a misleading feature.

`v2.0.0-alpha.8 implementation stop reached. Run pentest for this exact commit.`

### v2.0.0-alpha.9 - GPU Backend Boundary

**Status:** Planned.

**Goal:** Establish an optional GPU crate without exposing mutable scene
authoring or contaminating default/core dependency graphs.

**Deliverables:** Add `hashavatar-gpu`; define the narrow versioned read-only
core backend protocol, scene capability negotiation, device/backend status,
checked resource budgets, explicit CPU fallback decision, typed unsupported and
device errors, and reviewed GPU dependency/unsafe/driver boundaries.

**Verification:** Dependency-direction and default-graph tests; public API and
semver review of the backend protocol; malformed capability/resource tests;
no-adapter/headless behavior; package/MSRV/platform review; deliberate proof
that enabling unrelated facade features cannot enable GPU transitively.

**Exit criteria:** Applications can opt into and inspect GPU capability without
executing an unvalidated scene, silently falling back, or adding GPU code to
minimal/default builds.

`v2.0.0-alpha.9 implementation stop reached. Run pentest for this exact commit.`

### v2.0.0-alpha.10 - GPU Scene Execution

**Status:** Planned.

**Goal:** Render the complete supported scene through the optional GPU backend
with explicit conformance and cleanup limits.

**Deliverables:** Implement bounded scene translation/execution, deterministic
command order, output readback, buffer initialization and completion-fence
reuse policy, device-loss/submission/mapping failure handling, backend metadata,
and caller-selected fallback. Declare GPU output visually conforming and
noncanonical unless exact equality is proven.

**Verification:** Differential corpus against CPU across the declared
vendor/driver/software-adapter matrix; all families/styles/dimensions; device
loss and allocation/dispatch bounds; repeated reuse and cleanup tests;
headless/no-device fixtures; performance and resource baselines; packaged
downstream example.

**Exit criteria:** Every supported scene either renders within the declared GPU
contract or returns a typed error; mismatches are measured and documented; CPU
remains the canonical pixel authority unless zero-difference evidence supports
a stronger backend-specific claim.

`v2.0.0-alpha.10 implementation stop reached. Run pentest for this exact commit.`

## 2.0 Beta Releases

No new architecture or art enters after beta. A breaking correction resets the
line to another alpha.

### v2.0.0-beta.1 - Public API And Contract Freeze

**Status:** Planned.

**Goal:** Freeze the Rust API and all identifiers that downstream code or
caches depend on.

**Deliverables:** Complete rustdoc; freeze builders, prepared values, style and
layout reports, budgets, surfaces, writer APIs, errors, IDs, keys, feature graph,
defaults, pixel/digest/derivation/SVG contracts, and the opaque/private scene
boundary; freeze heapless capacity/storage APIs, schema V1, AVIF settings, and
the narrow GPU backend protocol. Add checked per-crate public API snapshots and
cargo-semver-checks.

**Verification:** API snapshot/diff for every published crate,
compile-pass/fail UI tests, valid feature powerset, duplicate public dependency
tests, packaged downstream builds, default/cache migration vectors, schema
compatibility snapshots, backend protocol review, and manual API/security
review.

**Exit criteria:** Later changes are compatibility-preserving or explicitly
return to alpha. No internal numeric or scene representation is accidentally a
public stability commitment.

`v2.0.0-beta.1 implementation stop reached. Run pentest for this exact commit.`

### v2.0.0-beta.2 - Assurance And Migration Freeze

**Status:** Planned.

**Goal:** Close resource, cleanup, portability, integration, performance, and
documentation gaps against the frozen API.

**Deliverables:** Final CPU/heapless/codec/GPU resource budgets and performance
baselines; cleanup and secret-lifetime audit; x86_64/AArch64/WASM/32-bit CPU and
heapless determinism matrix; declared GPU conformance matrix;
debug/release/LTO evidence; migrated `hashavatar-website`; packaged minimal,
formats, heapless, schema, and GPU consumers; README, migration guide, service
recipe, threat model, limitations, dependency inventory, SBOM, and release
evidence drafts.

**Verification:** Worst-case dimensions/styles, allocation and concurrency
math, normal/error/unwind cleanup, cross-platform scene/pixel/SVG fixtures,
zero-allocation heapless evidence, AVIF corpus, schema malformed-input corpus,
GPU differential/device-failure tests, benchmark thresholds, examples under
documented feature profiles, reproducible package tests, CI, and CodeQL default
setup.

**Exit criteria:** Real consumers need no undocumented workaround; documentation
and measured behavior agree; no unresolved canonical mismatch remains. This
cleanup/security boundary receives focused adversarial review.

`v2.0.0-beta.2 implementation stop reached. Run pentest for this exact commit.`

## 2.0 Release Candidates

### v2.0.0-rc.1 - Exact Candidate Verification

**Status:** Planned.

**Goal:** Test the frozen packages as an adversarial release candidate.

**Deliverables:** Mandatory pinned Kani job, defined fuzz campaign, Miri or
sanitizers where meaningful, dependency/license/unsafe review, semantic SBOM,
reproducibility and provenance evidence, final package archives, migration
trials, and a full independent pentest against the exact commit.

**Verification:** Kani cannot silently skip; fuzzing has no unresolved
crash/hang; local archives install in clean downstream fixtures; package hashes
and evidence reproduce; GitHub and CodeQL are green; findings have concrete
dispositions.

**Exit criteria:** No critical/high finding or release-blocking correctness,
contract, portability, packaging, or documentation gap remains.

`v2.0.0-rc.1 implementation stop reached. Run pentest for this exact commit.`

### v2.0.0-rc.2 And Later - Remediation Candidates

**Status:** Planned as needed.

**Goal:** Retest fixes without adding unrelated features.

**Deliverables:** Focused remediation, regression tests, refreshed
corpus/SBOM/packages/provenance, updated finding dispositions, and release notes.

**Verification:** Repeat the full RC gate and independent retest for every code
change. Install and test the exact package archives intended for publication.

**Exit criteria:** The latest RC commit and package bytes are the exact approved
GA candidate. Any further code, dependency, metadata, or artifact change creates
another `rc.N`; there is no fixed maximum RC count.

`v2.0.0-rc.2 implementation stop reached. Run pentest for this exact commit.`

Every later candidate uses the same exact sentence with its concrete `rc.N`
version rather than a placeholder.

## v2.0.0 - Stable Release

**Status:** Planned.

**Goal:** Publish the first stable canonical-scene Hashavatar library.

**Deliverables:** Promote the exact approved RC commit; publish
`hashavatar-core`, `hashavatar-formats`, `hashavatar-heapless`,
`hashavatar-schema`, `hashavatar-gpu`, and `hashavatar` in dependency order; tag
immutable source and release evidence; publish final migration and security
documentation.

**Verification:** Final readiness confirms tag target, package checksums,
provenance, SBOMs, release notes, pentest evidence, supported-target matrix,
GitHub status, CodeQL status, and crates.io publish order.

**Exit criteria:** All of the following are true:

- the public request/preparation/render/encode workflow is stable and documented;
- canonical CPU pixels and SVG derive from one validated private scene;
- external RGBA, digest, derivation, SVG, failure, and cache contracts are frozen;
- all existing 1.x families, styles, backgrounds, and frames are represented;
- the portable core supports documented `no_std + alloc` and no-allocator
  caller-storage profiles while remaining codec-free;
- WebP, PNG, JPEG, GIF, and admitted AVIF are isolated and feature-controlled;
- schema V1 is bounded, versioned, transport-neutral, and HTTP-free;
- the optional GPU backend passes its declared device/conformance matrix and
  reports noncanonical status honestly;
- supported targets match the published scene/pixel corpus;
- the reference website works from published package archives;
- no unresolved critical/high security or correctness finding remains;
- no documentation claims anonymity, universal constant time, guaranteed total
  zeroization, browser pixel parity, zero allocation, or encoded-byte stability
  beyond the contracts actually proven;
- every required companion crate or advanced feature is published and absent
  from the default dependency graph unless explicitly selected;
- external scene authoring and plugin execution remain out of scope.

`v2.0.0 implementation stop reached. Run pentest for this exact commit.` The
stable tag may reuse the exact approved RC evidence only when commit, package
archives, checksums, SBOM, provenance, and metadata are unchanged; otherwise
another RC is required.

## Required 2.0 Component Roadmaps

The following documents provide deeper admission criteria for components with
explicit alpha milestones in the main sequence:

- `docs/roadmaps/gpu.md`
- `docs/roadmaps/avif.md`
- `docs/roadmaps/schema.md`
- `docs/roadmaps/heapless.md`

No component may change the canonical CPU pixel contract silently or add its
heavy dependencies to `hashavatar-core`. Each companion crate uses an
independent version line, remains optional for downstream users, and must meet
its own roadmap plus assigned alpha finish line before Hashavatar 2.0 can reach
beta.

JPEG XL has no active roadmap. A future minor release may reconsider it only
from then-current implementation, license, conformance, security, maintenance,
resource, and MSRV evidence.

## Critical Path

```text
1.x contract and migration preparation
  -> Cat vertical slice from one private scene
  -> complete canonical CPU and SVG renderer
  -> existing catalog port
  -> layered style/layout
  -> portable core and established formats isolation
  -> heapless caller storage
  -> bounded request schema
  -> AVIF admission
  -> GPU boundary and execution
  -> public API and contract freeze
  -> assurance and downstream migration
  -> RC verification/remediation
  -> 2.0.0
```

JPEG XL and external plugins remain outside this path. GPU, AVIF, schema, and
heapless storage are explicit 2.0 requirements even though downstream users
must opt into their Cargo features or companion crates.
