# hashavatar 0.5.0

`hashavatar` 0.5.0 prepares the crate for dual permissive licensing and hardens the bundled demo server without changing the deterministic avatar rendering contract.

## Highlights

- Starting with `0.5.0`, the project is licensed as `MIT OR Apache-2.0`
- Added `LICENSE-MIT` and `LICENSE-APACHE`
- Removed the previous EUPL license files from the current source tree
- Added Fluxheim-style local and GitHub CI checks through `scripts/checks.sh`
- Added Dependabot configuration for Cargo and GitHub Actions updates
- Pinned GitHub Actions to immutable commit SHAs for CodeQL-friendly workflow hardening

## Security And Quality

- Moved demo-server WebP rendering and encoding onto Tokio's blocking task pool to avoid starving async worker threads
- Added defense-in-depth HTTP security headers to demo HTML, image, and error responses
- Added regression coverage for the demo response security headers
- Updated `tokio` to `1.52.3`
- Verified with `scripts/checks.sh`, including formatting, build, clippy, tests, CLI smoke exports, `cargo deny check`, and `cargo audit`

## Compatibility

- No avatar rendering behavior changes are intended in this release.
- Existing deterministic fingerprints remain covered by the golden regression tests.
- Published `0.4.x` and older versions retain their original release licensing. The `MIT OR Apache-2.0` license applies starting with `0.5.0`.
