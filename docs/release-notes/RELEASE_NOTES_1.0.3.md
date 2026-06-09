# hashavatar 1.0.3

`1.0.3` is a security-hardening and CI maintenance release for `hashavatar`.

## Security And Hardening

- Encoded output is now accumulated in a `Zeroizing<Vec<u8>>` until successful
  return. If an encoder returns an error after writing partial bytes, those
  partially encoded bytes are scrubbed before being dropped.
- Identity, cache-key, and XXH3 chunk preimage builders now assert exact
  capacity in debug/test builds. This catches future component-size drift that
  would otherwise allow reallocations before zeroization.
- `AvatarBuilder` debug output now redacts namespace tenant and style-version
  values, not only the raw identity input.
- Security controls now document the accepted `1.x` visual-stability tradeoff
  that some established renderers use selected upper digest bytes directly for
  visible geometry.

## CI

- Updated pinned GitHub Actions:
  - `actions/checkout` to `v6.0.3`
  - `Swatinem/rust-cache` to `v2.9.1`
  - `taiki-e/install-action` to `v2.81.8`

## Compatibility

- No intentional avatar visual fingerprint changes.
- No public API removals.
- No dependency version changes.
