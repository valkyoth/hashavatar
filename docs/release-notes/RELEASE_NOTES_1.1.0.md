# hashavatar 1.1.0

`1.1.0` is a minor release for `hashavatar` focused on dependency,
sanitization, tooling, source layout, and documentation freshness.

## Dependency Updates

- Replaced direct `zeroize` usage with the native `sanitization` crate API.
- Added `sanitization` `1.2.2` with `alloc` support.
- Added `sanitization-crypto-interop` `1.2.2` so SHA-512 and optional BLAKE3
  hashing use the crypto crates' own hasher-state cleanup hooks through the
  `sanitization` sister crate.
- Removed direct `zeroize` dependency usage and the `sha2`/`blake3` zeroize
  feature hooks.
- Updated the fuzz harness `libfuzzer-sys` dependency to `0.4.13`.
- Refreshed Cargo lockfiles with the latest compatible crate versions.

## Sanitization

- Fixed-size digest and renderer seed copies now use `sanitization::Secret`.
- SHA-512 identity hashing and cache-key hashing now route through the
  `sanitization-crypto-interop` SHA-512 helper.
- The crate's direct `sha2` dependency is now dev-only; production SHA-512
  hashing reaches `sha2` through `sanitization-crypto-interop`.
- Removed a redundant SHA-512 digest `Secret` wrapper now that the interop
  helper owns hasher-state cleanup and the caller already guards the returned
  digest.
- Optional BLAKE3 XOF output now uses the `sanitization-crypto-interop` fill
  helper with a `sanitization::Secret` output buffer.
- Optional XXH3 digest accumulation now uses a `sanitization::Secret` guard for
  the 64-byte accumulator.
- Hash-preimage capacity checks now use release-mode assertions, so future
  size-accounting drift cannot silently bypass temporary buffer cleanup.
- Optional XXH3 chunk capacity and length checks are collapsed into one
  release assertion per chunk.
- Hash preimage vectors, encoded-output buffers, temporary JPEG RGB buffers,
  and owned RGBA buffers are cleared through `sanitization` volatile clearing
  helpers.
- Security controls now document the `sanitization-crypto-interop` cleanup
  boundary for SHA-512 and optional BLAKE3.
- `cargo-deny` now denies duplicate crate versions instead of only warning.

## CI

- Updated pinned GitHub Actions:
  - `actions/checkout` to `v7.0.0`
  - `taiki-e/install-action` to `v2.82.3`
- Confirmed `Swatinem/rust-cache` remains current at `v2.9.1`.

## Documentation

- Updated README installation snippets and compatibility wording to `1.1.0`.
- Updated the changelog for the `1.1.0` release.
- Split the former monolithic `src/lib.rs` into focused source files, including
  per-avatar raster and SVG renderer files, without changing the public API.

## Compatibility

- No intentional avatar visual fingerprint changes.
- No public API removals.
