# Security Controls

`hashavatar` is a deterministic avatar rendering crate. Its primary security concerns are resource exhaustion, panic safety for untrusted parameters, dependency hygiene, and safe SVG/raster output.

## Current Controls

- The library uses `#![forbid(unsafe_code)]`.
- `AvatarSpec` has private fields and validates dimensions at construction.
- Public raster/SVG dimensions are bounded by `MIN_AVATAR_DIMENSION` and `MAX_AVATAR_DIMENSION`.
- Identity inputs are bounded by `MAX_AVATAR_ID_BYTES`.
- Namespace tenant and style-version components are bounded by `MAX_AVATAR_NAMESPACE_COMPONENT_BYTES`.
- Image-generation APIs return typed errors for unsupported dimensions before allocating or encoding raster output.
- The crate exposes in-memory encoding and rendering APIs, but no public filesystem path-writing helpers.
- Namespace identity hashing length-prefixes every component, so tenant and style-version values cannot collide through embedded separator bytes.
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

The crate does not ship an HTTP server. Public web/API concerns such as request concurrency, rate limiting, CDN caching, security headers, observability, and abuse controls belong in `hashavatar-api`.
