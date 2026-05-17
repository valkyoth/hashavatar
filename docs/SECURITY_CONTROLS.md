# Security Controls

`hashavatar` is a deterministic avatar rendering crate. Its primary security concerns are resource exhaustion, panic safety for untrusted parameters, dependency hygiene, and safe SVG/raster output.

## Current Controls

- The library uses `#![forbid(unsafe_code)]`.
- Public raster/SVG dimensions are bounded by `MIN_AVATAR_DIMENSION` and `MAX_AVATAR_DIMENSION`.
- Image-generation APIs return typed errors for unsupported dimensions before allocating or encoding raster output.
- Namespace identity hashing length-prefixes every component, so tenant and style-version values cannot collide through embedded separator bytes.
- Internal rectangle edge helpers use saturating arithmetic.
- The SVG renderer emits generated shape markup from structured numeric values rather than from caller-provided SVG fragments.
- Golden fingerprint tests protect deterministic rendering output.
- The crate package excludes fuzz harnesses and generated build output.
- `scripts/checks.sh` runs formatting, metadata, dependency, unsafe-boundary, panic-policy, tests, `cargo deny`, and `cargo audit`.

## Service Boundary

The crate does not ship an HTTP server. Public web/API concerns such as request concurrency, rate limiting, CDN caching, security headers, observability, and abuse controls belong in `hashavatar-api`.
