 Its architectural core is right. Its practical problem is that optional work—GPU, AVIF, JPEG XL, schemas, heapless scenes, multiple companion crates—has returned to the
  numbered alpha path. Even though the document says these must not block 2.0, sequential milestones alpha.18 through alpha.22 make them effective blockers before the
  facade is assembled at alpha.23.

  My recommendation is:

  - Build a polished public Rust library API.
  - Do not build an HTTP/REST API into this crate or workspace.
  - Keep facade + canonical CPU core + established formats on the 2.0 critical path.
  - Move GPU, AVIF/JXL, schema/wire documents, and heapless storage to independent companion roadmaps.
  - Bring the Cat vertical slice much earlier.
  - Add several missing low-level contracts before implementation.

  ## Should hashavatar include an API?

  ### Yes: a public Rust rendering API

  The plan’s transport-neutral API is appropriate and should be part of 2.0:

  - AvatarIdentity and keyed identity construction.
  - AvatarRequest.
  - AvatarRequestBuilder.
  - PreparedAvatar.
  - ResolvedStyle.
  - LayoutReport.
  - ResourceBudget.
  - IdentityCacheKey, AvatarAssetKey, SemanticEncodedAssetKey, and
    BuildEncodedAssetKey.
  - RasterSurfaceMut.
  - render_into.
  - write_svg.
  - Writer-based and convenience encoding APIs.
  - Typed capability, validation, capacity, rendering, writer, and encoding errors.

  The proposed preparation flow in plan-towards-2.0.0.md:222 is particularly good. PreparedAvatar prevents an application from validating one tuple, deriving a key from
  another, and rendering a third.

  A suitable facade might look like:

  let prepared = AvatarRequest::builder(identity)
      .spec(spec)
      .style(style)
      .prepare()?;

  let key = prepared.asset_key();
  let layout = prepared.layout_report();
  let budget = prepared.resource_budget();

  prepared.render_into(&mut surface)?;
  prepared.write_svg(&mut svg_writer)?;

  The formats boundary can add:

  formats::encode_to_writer(
      &prepared,
      AvatarOutputFormat::WebP,
      &mut output,
  )?;

  ### No: an HTTP or hosted service API

  HTTP should remain the responsibility of the application using the crate. The plan already makes the correct decision at plan-towards-2.0.0.md:62 and provides a good
  service recipe at plan-towards-2.0.0.md:283.

  The crate should not own:

  - Axum or another web framework.
  - Tokio/runtime integration.
  - Query-string parsing.
  - Authentication or authorization.
  - Rate limiting.
  - Request concurrency permits.
  - Timeouts and cancellation policy.
  - Proxy trust.
  - CORS/CSP/security headers.
  - S3/object storage.
  - CDN/cache-control policy.
  - Persistence.
  - Observability and telemetry.
  - Identity normalization rules.

  Those decisions vary materially by deployment. Putting them in hashavatar would enlarge the dependency graph, make no_std harder, and imply a security posture the
  rendering crate cannot enforce.

  If an officially supported hosted API is desired, keep it as hashavatar-website, hashavatar-api, or a separate service repository. It can serve as the reference
  integration without becoming a dependency of the library.

  ### Schema adapter

  hashavatar-schema is defensible as a companion crate, but it should not block 2.0. A versioned JSON document is a long-term wire-compatibility promise. Build it only
  after the Rust AvatarRequest API is frozen and preferably after two independent consumers demonstrate demand.

  OpenAPI remains application-owned because HTTP status codes, headers, authentication, and endpoint structure are outside the rendering contract.

  ## Release-critical changes to the plan

  ### 1. Remove optional projects from the sequential alpha path

  The largest contradiction is:

  - AVIF decision at plan-towards-2.0.0.md:957.
  - JPEG XL decision at alpha.19.
  - GPU scaffold and execution at plan-towards-2.0.0.md:993.
  - Schema adapter at plan-towards-2.0.0.md:1028.
  - User-facing facade only at alpha.23.

  If releases are sequential, those tasks block facade stabilization regardless of the text saying otherwise.

  Move them into separate documents:

  roadmaps/gpu.md
  roadmaps/avif.md
  roadmaps/jpeg-xl.md
  roadmaps/schema.md
  roadmaps/heapless.md

  They may proceed in parallel, but the main roadmap must be capable of advancing directly from established formats to facade freeze.

  The dependency observations themselves are sensible and current: wgpu 30.0.0 declares MSRV 1.87 and enables a broad default backend graph; jpegxl-rs
  0.15.0+libjxl-0.12.0 declares GPL-3.0-or-later and Rust 1.92; zune-jpegxl 0.5.2 describes itself as a limited/POC-style pure-Rust encoder; and jxl-oxide 0.12.6 is a
  decoder. These facts reinforce deferral rather than justify placing codec research on the main path. wgpu manifest
  (https://docs.rs/crate/wgpu/latest/source/Cargo.toml), jpegxl-rs manifest (https://docs.rs/crate/jpegxl-rs/latest/source/Cargo.toml), zune-jpegxl documentation
  (https://docs.rs/zune-jpegxl), jxl-oxide documentation (https://docs.rs/crate/jxl-oxide/latest).

  ### 2. Bring the vertical slice forward

  The Cat vertical slice currently arrives at alpha.7, after:

  - Workspace design.
  - Public numeric types.
  - Scene storage.
  - Raster primitives.
  - Complete path/group semantics.
  - Active trait derivation.

  That is too waterfall-heavy. Several abstractions could be made public before a real avatar demonstrates that they are correct.

  The first meaningful alpha should prove:

  fixed traits
    → fixed geometry
    → minimal scene
    → Cat
    → canonical raster
    → SVG from the same scene

  Only then generalize paths, groups, gradients, and public APIs.

  I would also avoid making fixed numeric types public unless callers genuinely need to author scenes. If external family/plugin execution is deferred, Q16.16 types can
  remain internal implementation details.

  ### 3. Keep the scene AST private or opaque

  The plan says external plugins are deferred, but hashavatar-core owns scene authoring and beta freezes “scenes.” That risks stabilizing a general graphics API without a
  consumer need.

  For 2.0:

  - Keep command/path/paint enum layouts private.
  - Expose an opaque ValidatedScene or SceneRef only if GPU/inspection consumers require it.
  - Expose SceneDigest, bounds, budgets, and capability metadata.
  - Keep canonical serialization internal and one-way.
  - Do not provide public scene deserialization.
  - Do not promise external scene authoring.

  This gives the project freedom to improve arena layout and internal commands during 2.x while preserving rendered behavior.

  ### 4. Shorten the 1.x runway

  The plan goes from 1.1.3 through 1.12.0 before the first alpha. That creates ten stable releases, several containing private shadow implementations, source refactoring,
  and full pentest handoffs.

  I would consolidate:

  - 1.1.3: Correct enum/Kani documentation.
  - 1.2.0: Contract/catalog IDs, complete asset keys, opt-in strict validation, optionally keyed pseudonymization.
  - 1.3.0: Useful writer APIs, prepared-request preview, compatibility corpus, source splits needed to support those APIs, and migration/deprecation guidance.
  - Then branch to 2.0 alpha.

  Private fixed-point, trait derivation, and scene prototypes belong on the 2.0 branch. Shipping inactive private prototypes in several stable 1.x releases adds package
  weight and maintenance exposure without helping 1.x users.

  ### 5. Reconsider the 500-line hard rule

  Splitting the current 2,516-line test module and 1,336-line layer module is sensible. Treating exactly 500 lines as a universal architectural invariant is less
  convincing.

  A line limit can encourage:

  - Artificially fragmented tests.
  - Excessively shallow modules.
  - Navigation overhead.
  - Splits by size rather than responsibility.

  Prefer:

  - A warning around 500 lines for production modules.
  - A hard limit only with an explicit, reviewed exception mechanism.
  - More generous limits for tests, fixtures, and generated exhaustive tables.
  - Enforcement of dependency direction, ownership, cyclomatic complexity, and panic/unsafe policy as stronger architectural signals.

  Do not spend multiple stable releases splitting legacy modules that will soon be deleted by the scene rewrite.

  ### 6. Decide compat-v1 earlier

  The compatibility decision currently occurs at alpha.24, after the old family implementations have been replaced.

  Determine downstream demand during the 1.x migration period. If exact legacy rendering will be required, isolate it at the workspace transition before deleting or
  restructuring the old implementation. Otherwise explicitly reject it and rely on pinned 1.x for callers requiring old pixels.

  ## Missing technical contracts

  ### 1. External pixel ABI

  The plan defines premultiplied compositing but does not fully specify what callers receive. Add a dedicated PIXEL_CONTRACT.md covering:

  - sRGB transfer/color space.
  - Straight versus premultiplied external RGBA.
  - Alpha interpretation.
  - Channel order.
  - Top-left row origin.
  - Pixel-center and sample-grid definitions.
  - Transparent-pixel RGB canonicalization—preferably [0, 0, 0, 0].
  - Stride validation.
  - Whether stride padding is preserved, cleared, or included in PixelDigest.
  - Empty/zero-sized behavior.
  - Exact conversion from internal compositing representation to codec input.

  Most codecs expect straight-alpha RGBA, so either make that the canonical external surface or expose clearly distinct Rgba8Srgb and Rgba8PremulSrgb formats.

  ### 2. Digest specifications

  Define separately for every digest/key:

  - Hash/PRF algorithm.
  - Protocol label.
  - Version.
  - Field order.
  - Length prefix width.
  - Integer endianness.
  - Inclusion/exclusion rules.
  - Truncation length.
  - Hex/base encoding.
  - Whether padding, metadata, accessibility text, or nonsemantic scene IDs participate.

  A canonical internal scene serialization need not be public, but its hashing algorithm must be frozen precisely.

  ### 3. Stateless derivation algorithm

  derive_u16 is currently conceptual. Before freezing it, specify:

  - Underlying PRF/XOF.
  - Label encoding.
  - Counter encoding and endianness.
  - Seed handling.
  - Range selection.
  - Rejection sampling or accepted modulo bias.
  - Trait independence.
  - Catalog weighting algorithm.
  - Exact behavior when new catalogs are added.

  The selected identity hash Cargo feature must not silently change the visual derivation protocol unless its algorithm ID is also part of the contract.

  ### 4. Failure and partial-output semantics

  For each sink/surface API, document:

  - Whether the output surface may be partially modified on failure.
  - Whether writer failures leave a valid prefix.
  - Whether retrying is allowed.
  - Whether OutputBufferTooSmall reports the required size.
  - Whether scene preparation is transactional.
  - Cleanup behavior after capacity, writer, codec, or validation errors.
  - Which errors are deterministic for the same request.

  Strong exception safety for a full raster may require a second buffer, so partial modification is acceptable—but it must be explicit.

  ### 5. SVG embedding contract

  Add requirements for:

  - Collision-free clip/gradient IDs when several avatars are embedded in one SVG document.
  - Deterministic caller-supplied or scene-derived ID prefixes.
  - SVG document versus fragment output.
  - XML escaping.
  - Accessibility title/description policy.
  - Whether accessibility metadata participates in canonical SVG output.
  - Stable numeric formatting independent of locale.
  - Unsupported scene capability reporting.

  ### 6. Canonical execution order

  State explicitly that canonical compositing order is independent of:

  - Tile scheduling.
  - Thread count.
  - Iteration order of maps.
  - Parallel execution.
  - CPU feature detection.

  The initial canonical backend should be single-threaded or use provably independent tiles. Parallelism can be added later without changing pixel order.

  ### 7. 32-bit portability

  The plan includes an embedded target, but resource and index math should also be tested on a 32-bit usize target. Maximum pixel and scene budgets that are safe on 64-
  bit may behave differently on embedded/WASM32 systems.

  ### 8. Default migration and cache busting

  Define what happens to:

  - AVATAR_STYLE_VERSION.
  - Default namespace style version.
  - Existing AvatarIdentity::cache_key.
  - Default family/catalog selection.
  - CDN/object cache paths.
  - Callers that omit an explicit CatalogVersion or RenderContractId.

  A 2.0 request must not accidentally reuse a 1.x asset key for different pixels.

  ### 9. Public API compatibility tooling

  Add:

  - cargo-semver-checks or an equivalent API diff after beta.
  - A checked public API snapshot.
  - Compile-fail/UI tests for invalid style and surface construction.
  - Downstream tests using only published package archives—not workspace paths.
  - Duplicate-version tests for public companion-crate types re-exported by the facade.

  ### 10. Testkit dependency direction

  The diagram says hashavatar-testkit depends on every crate. Be careful not to also make those crates dev-depend on testkit, which can create dependency cycles or
  awkward duplicate compilation.

  Prefer:

  - Testkit depends on core fixtures and generic traits.
  - Top-level integration tests depend on facade/backends and testkit.
  - Production crates do not depend back on a testkit that already imports them.

  ## Release-process refinements

  Two policies are excessive:

  - Full pentest/retest after every documentation or source-splitting release.
  - Release gates that require being on the latest network-visible tool version.

  Use continuous internal security review, fuzzing, and policy checks at every stop. Reserve independent pentests for:

  - Keyed identity protocol.
  - First complete canonical renderer.
  - Cleanup/security beta.
  - RC and remediation RCs.

  For dependencies and tools, freeze reviewed versions for reproducibility and generate a freshness report. A new upstream version appearing moments before release should
  not automatically invalidate an otherwise reviewed artifact. The Cargo documentation also distinguishes reproducible locked builds from testing newer compatible
  dependency resolution; both are useful, but they serve different purposes. Cargo reproducibility guidance (https://doc.rust-lang.org/cargo/faq.html).

  ## Recommended reduced roadmap

   Release    Scope
  ━━━━━━━━━  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   1.1.3      Correct enum semver and Kani documentation.
  ─────────  ────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
   1.2.0      Catalog/render IDs, complete keys, opt-in validation, keyed identity decision.
  ─────────  ────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
   1.3.0      Prepared-request preview, writer APIs that genuinely help, compatibility corpus, migration/deprecation guide, necessary source splits.
  ─────────  ────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
   alpha.1    Minimal workspace boundary plus private fixed math/scene and complete Cat vertical slice.
  ─────────  ────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
   alpha.2    Canonical primitives, paths, compositing, scene validation, SVG completion.
  ─────────  ────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
   alpha.3    Port all families, backgrounds, and frames.
  ─────────  ────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
   alpha.4    Typed accessory slots, calibrated multi-layer layout, expressions, palettes.
  ─────────  ────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
   alpha.5    Final no_std + alloc, caller surfaces/scratch, formats isolation, facade assembly.
  ─────────  ────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
   beta.1     Public Rust API, IDs, errors, feature graph, opaque scene boundary freeze.
  ─────────  ────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
   beta.2     Resource/performance, cleanup, cross-platform determinism, downstream migration, documentation.
  ─────────  ────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
   rc.1       Mandatory Kani, fuzz campaign, dependency review, full pentest.
  ─────────  ────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
   rc.2+      Remediation and exact artifact candidates as needed.
  ─────────  ────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
   2.0.0      Canonical CPU + SVG + established formats only.

  GPU, schema, AVIF, JPEG XL, heapless scenes, and external plugin APIs should have independent readiness decisions and version lines.
