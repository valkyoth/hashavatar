# hashavatar 1.0.4

`1.0.4` is a maintenance release for `hashavatar` focused on dependency,
sanitization, tooling, and documentation freshness.

## Dependency Updates

- Replaced direct `zeroize` usage with the native `sanitization` crate API.
- Added `sanitization` `1.2.1` with `alloc` support.
- Removed direct `zeroize` dependency usage and the `sha2`/`blake3` zeroize
  feature hooks.
- Updated the fuzz harness `libfuzzer-sys` dependency to `0.4.13`.
- Refreshed Cargo lockfiles with the latest compatible crate versions.

## Sanitization

- Fixed-size digest and renderer seed copies now use `sanitization::Secret`.
- Hash preimage vectors, encoded-output buffers, temporary JPEG RGB buffers,
  and owned RGBA buffers are cleared through `sanitization` volatile clearing
  helpers.
- Security controls now document that third-party hasher internal state remains
  outside `hashavatar`'s cleanup boundary.

## CI

- Updated pinned GitHub Actions:
  - `actions/checkout` to `v7.0.0`
  - `taiki-e/install-action` to `v2.82.3`
- Confirmed `Swatinem/rust-cache` remains current at `v2.9.1`.

## Documentation

- Updated README installation snippets and compatibility wording to `1.0.4`.
- Updated the changelog for the `1.0.4` release.

## Compatibility

- No intentional avatar visual fingerprint changes.
- No public API removals.
