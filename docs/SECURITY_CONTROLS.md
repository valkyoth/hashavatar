# Security Controls

`hashavatar` is a deterministic avatar rendering crate. Its primary security concerns are resource exhaustion, panic safety for untrusted parameters, dependency hygiene, and safe SVG/raster output.

## Current Controls

- The library uses `#![forbid(unsafe_code)]`.
- `AvatarSpec` has private fields and validates dimensions at construction.
- Public raster/SVG dimensions are bounded by `MIN_AVATAR_DIMENSION` and `MAX_AVATAR_DIMENSION`.
- Raw RGBA raster memory is bounded by `MAX_AVATAR_RGBA_BYTES` per render before
  encoder overhead.
- Identity inputs are bounded by `MAX_AVATAR_ID_BYTES`.
- Namespace tenant and style-version components are bounded by `MAX_AVATAR_NAMESPACE_COMPONENT_BYTES`.
- Image-generation APIs return typed errors for unsupported dimensions before allocating or encoding raster output.
- The crate exposes in-memory encoding and rendering APIs, but no public filesystem path-writing helpers.
- Namespace identity hashing length-prefixes every component, so tenant and style-version values cannot collide through embedded separator bytes.
- SHA-512 remains the default identity hash. Optional hash algorithms are
  domain-separated from the default path.
- BLAKE3 support is available only through the explicit `blake3` feature.
- XXH3-128 support is available only through the explicit `xxh3` feature and
  is documented as non-cryptographic. Do not use XXH3-128 for adversarial or
  user-controlled identifiers unless the application first maps those
  identifiers through its own cryptographic boundary.
- Procedural RNG seeding uses 256 bits from the second half of the identity
  digest, separate from the lower digest bytes commonly used for direct visual
  parameters.
- `AvatarIdentity` equality uses constant-time digest comparison.
- Derived identity digests and temporary hash preimage buffers are zeroized
  when dropped.
- Encode APIs zeroize temporary owned raster buffers after encoding. JPEG
  export also zeroizes the temporary RGB flattening buffer. Returned encoded
  bytes and images returned by render APIs are caller-owned and must be cleared
  by the caller if their environment requires that.
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
  API rate limits, request concurrency limits, and caching.
- Internal rectangle helpers use saturating or clamping arithmetic.
- The SVG renderer emits generated shape markup from structured numeric values rather than from caller-provided SVG fragments.
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
