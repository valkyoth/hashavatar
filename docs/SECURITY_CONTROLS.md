# Security Controls

`hashavatar` is a deterministic avatar rendering crate. Its primary security concerns are resource exhaustion, panic safety for untrusted parameters, dependency hygiene, and safe SVG/raster output.

## Current Controls

- The library uses `#![forbid(unsafe_code)]`.
- `AvatarSpec` has private fields and validates dimensions at construction.
- `AvatarSpec::default()` is fixed at `256x256` with seed `1` and is covered
  by a regression test. It is a deterministic convenience value, not a random
  or production policy default.
- Public raster/SVG dimensions are bounded by `MIN_AVATAR_DIMENSION` and `MAX_AVATAR_DIMENSION`.
- Raw RGBA raster memory is bounded by `MAX_AVATAR_RGBA_BYTES` per render before
  encoder overhead.
- Identity inputs are bounded by `MAX_AVATAR_ID_BYTES`.
- Namespace tenant and style-version components are bounded by `MAX_AVATAR_NAMESPACE_COMPONENT_BYTES`.
- Image-generation APIs return typed errors for unsupported dimensions before allocating or encoding raster output.
- The crate exposes in-memory encoding and rendering APIs, but no public filesystem path-writing helpers.
- Namespace identity hashing length-prefixes every component, so tenant and style-version values cannot collide through embedded separator bytes.
- SHA-512 remains the default identity hash. Optional hash modes are crate-wide
  feature choices, not runtime API choices, and are domain-separated from the
  default path.
- BLAKE3 support is available only through the explicit `blake3` feature.
- XXH3-128 support is available only through the explicit `xxh3` feature. It is
  non-cryptographic; do not use XXH3-128 for adversarial or user-controlled
  identifiers unless the application first maps those identifiers through its
  own cryptographic boundary.
- The `blake3` and `xxh3` features are mutually exclusive. Enabling both is a
  compile-time error so feature unification cannot silently mix hash modes.
- PNG and JPEG export are available only through the explicit `png` and `jpeg`
  features. The default build exposes WebP as the only raster encoder.
- GIF export is available only through the explicit `gif` feature. The `image`
  crate's GIF encoder performs internal 256-color quantization, and
  `hashavatar` cannot sanitize those codec-owned buffers. High-assurance
  deployments should prefer WebP or PNG.
- Procedural RNG seeding uses 256 bits from the second half of the identity
  digest. Most direct visual parameter lookups use lower digest bytes, but some
  established renderers also use selected upper digest bytes for visible
  geometry. This is accepted for `1.x` visual stability: avatars are
  digest-derived public artifacts, and removing those lookups would change
  golden output. Callers that treat identifiers as sensitive should prefer
  SHA-512 or BLAKE3 over XXH3 and follow the timing/output-size guidance below.
- The temporary 256-bit RNG seed copy is stored in `sanitization::Secret`, so
  the digest-derived seed copy is scrubbed on scope exit. The final value
  passed to `StdRng::from_seed` is also held in a `Secret` guard before the
  copy into `StdRng`. `StdRng::from_seed` still takes the seed by value, so a
  transient unguarded argument copy is part of the crate's documented
  by-value-copy sanitization caveat.
- The procedural RNG itself is `rand::rngs::StdRng`. Its expanded internal
  state is not sanitized on drop because `StdRng` does not currently expose a
  sanitization hook. In the default SHA-512 mode, recovering the original
  identity from that expanded state would require reversing SHA-512 output,
  which is computationally infeasible. High-assurance callers should still
  treat this as a known residual and prefer SHA-512 or BLAKE3 over XXH3 for
  sensitive identifiers.
- `AvatarIdentity` equality uses constant-time digest comparison.
- `AvatarIdentity` has a redacted `Debug` implementation so accidental
  `{:?}` logging does not print the raw digest.
- `AvatarBuilder` has a redacted `Debug` implementation so accidental builder
  logging does not print the raw identity input, tenant, or style-version
  namespace values.
- `AvatarIdentity::cache_key()` derives an opaque display key by hashing the
  internal identity digest under a cache-key domain instead of returning raw
  digest bytes. Cache keys are still stable correlators for the same identity
  and should be treated as public cache identifiers, not authentication
  secrets.
- `AvatarIdentity` implements `Clone`; every clone is sanitized independently
  on drop. High-assurance integrations should keep identity clones
  short-lived so digest bytes do not remain live in multiple memory locations
  longer than necessary.
- Derived identity digests and temporary hash preimage buffers are sanitized
  when dropped. The intermediate 64-byte digest returned by each hash backend is
  held in `sanitization::Secret` guards before being copied into
  `AvatarIdentity`, and temporary `Vec<u8>` preimages are cleared across full
  allocation capacity with `sanitization`'s volatile vector clear helper.
- SHA-512 hashing is routed through `sanitization-crypto-interop`, which enables
  upstream `sha2` hasher cleanup for SHA-512 state. BLAKE3 hashing is routed
  through the same interop crate when the `blake3` feature is enabled, so the
  BLAKE3 hasher and XOF reader are explicitly cleared after output extraction.
  The interop crate necessarily uses those crypto crates' own cleanup hooks;
  callers that audit dependency internals should keep
  `sanitization-crypto-interop`, `sha2`, and `blake3` in scope.
- Identity hash preimage allocation is sized from the tenant, style-version,
  and identity input lengths. Debug/test builds assert that preimage buffers do
  not reallocate before sanitization, so future component-size drift is caught
  by CI. The crate bounds and sanitizes those temporary buffers, but it does not
  hide input length from the allocator, OS-level heap profilers, or other
  same-process memory-observation tools. Very high-assurance callers that treat
  identifier length as sensitive should normalize or pad identifiers to a fixed
  length before calling this crate.
- The optional XXH3-128 mode derives the crate's 64-byte identity digest by
  hashing four domain-separated chunks. Each chunk temporarily copies the
  bounded preimage into a sanitized buffer, so peak preimage memory is higher
  than SHA-512 or BLAKE3. Keep XXH3 for low-stakes, non-adversarial workloads
  where its non-cryptographic collision profile and extra temporary preimage
  copy are acceptable.
- The crate seeds its own rendering RNG deterministically from identity digest
  bytes and does not use OS entropy for avatar rendering. The `rand` dependency
  may still bring a transitive `getrandom` dependency into the lockfile; WASM
  and embedded applications that use OS-backed randomness elsewhere in the same
  binary must configure and test `getrandom` for their target explicitly.
- Encode APIs wrap temporary owned raster buffers in RAII sanitization guards,
  so pixel data is cleared during normal returns, encoder errors, and unwinding
  panics. Encoded output is accumulated in a local `SanitizingVec` until
  successful return, so partially encoded bytes are scrubbed on encoder errors.
  JPEG export also wraps the temporary RGB flattening buffer in `SanitizingVec`.
  Returned encoded bytes and images returned by render APIs are caller-owned
  and must be cleared by the caller if their environment requires that. The
  README includes `sanitization` examples for returned `Vec<u8>` and
  `RgbaImage` buffers.
- Rendering time is intentionally not constant-time. Shape counts, geometry,
  raster encoding, and SVG length can vary with the identity digest, so callers
  should not treat rendered avatar timing or output size as secret-preserving
  side channels.
- High-assurance API wrappers can reduce render-time observability with stable
  cache keys, CDN caching, and a fixed-minimum-latency response wrapper. Apply
  that at the service boundary, and use async timers in async servers rather
  than blocking runtime worker threads.
- The crate bounds individual render sizes, but service-level memory exhaustion
  from many concurrent maximum-size renders must be controlled by callers with
  API rate limits, request concurrency limits, and caching. Use
  `MAX_AVATAR_RGBA_BYTES`, `AvatarSpec::rgba_buffer_len()`, and
  `AvatarRenderResourceBudget` to size service-level render semaphores against
  the application's memory budget. The crate intentionally does not ship a
  semaphore wrapper because concurrency primitives belong to the caller's
  async/runtime boundary.
- Internal rectangle helpers use saturating or clamping arithmetic for edge and
  intersection calculations. Rectangle size construction promotes zero
  dimensions to a one-pixel rectangle so rounded-down decorative features remain
  non-panicking; minimum-size rendering is covered by regression tests.
- Raster frame-shape hit-testing uses integer arithmetic for circle, squircle,
  hexagon, and octagon masks, reducing platform-specific floating-point
  rounding in clipping decisions.
- Decorative raster backgrounds are deterministic. Pattern and gradient modes
  are explicit by design, while the starry background now incorporates identity
  digest bytes in its local deterministic star-position generator.
- Polygon rasterization returns immediately for empty polygons and zero-sized
  images, widens scanline interpolation math before rounding, and is covered by
  a dedicated fuzz harness for arbitrary image dimensions, degenerate polygons,
  negative coordinates, and extreme `i32` points.
- The SVG renderer emits generated shape markup from structured numeric values rather than from caller-provided SVG fragments.
- SVG output is covered by parser-backed well-formedness tests across every
  avatar family, background mode, representative identity inputs, and every
  public visual-layer option. The fuzz harness also parses rendered SVG with
  `roxmltree` instead of only checking that SVG rendering does not panic.
- The hidden `fuzzing` feature exposes internal fuzz harness entry points only
  for test/fuzz builds. A compile-time guard rejects ordinary non-fuzzing
  release builds if that feature is accidentally enabled.
- Golden fingerprint tests protect deterministic rendering output.
- The crate package excludes fuzz harnesses and generated build output.
- `scripts/checks.sh` runs formatting, metadata, dependency, unsafe-boundary, panic-policy, tests, `cargo deny`, and `cargo audit`.
- Dependency additions and upgrades are expected to use current upstream
  documentation and latest compatible crate releases unless an older version is
  explicitly justified.
- GitHub CodeQL should use the repository's default setup. Do not add an
  advanced CodeQL workflow while default setup is active.

## Testing Standard

Every code change that alters behavior should include focused tests for the new
logic or a clear explanation for why existing tests cover it. Security changes
must include regression tests whenever the behavior can be tested locally.

## Service Boundary

The crate does not ship an HTTP server. Public web/API concerns such as request
concurrency, maximum simultaneous large renders, rate limiting, CDN caching,
security headers, observability, and abuse controls belong in `hashavatar-api`.
