# Digest And Derivation Contract

Alpha.5 identity derivation retains the SHA-512 domains introduced by the
canonical renderer:

- `hashavatar/identity/v2/sha512/v1`
- `hashavatar/trait/v2/sha512/v1`

Identity preimages contain domain, tenant, style version, and caller input in
that order. Every component is prefixed by its little-endian `u64` byte length.
Trait preimages contain trait domain, the 64-byte identity digest, little-
endian `u64` style seed, the length-prefixed trait label, and a little-endian
`u32` counter. Alpha.5 currently uses counter zero and interprets the first two
digest bytes as a little-endian `u16` sample.

Labels are independent: adding or evaluating one trait does not consume shared
state or shift another trait. Identity digests are not public and are not cache
keys. Pixel digests use a separate domain and construction documented in
[PIXEL_CONTRACT.md](PIXEL_CONTRACT.md).

Alpha.5 adds four public, correlatable key types. Each is SHA-512 derived with
little-endian `u64` length prefixes and an independent domain; the first 32
digest bytes form the public key:

- `IdentityCacheKey`: `hashavatar/identity-cache-key/v2/sha512/v1`;
- `AvatarAssetKey`: `hashavatar/avatar-asset-key/v2/sha512/v1`;
- `SemanticEncodedAssetKey`:
  `hashavatar/encoded-semantic-key/v2/sha512/v1`;
- `BuildEncodedAssetKey`:
  `hashavatar/encoded-build-key/v2/sha512/v1`.

The canonical asset key binds the identity cache key, catalog, render, and
pixel contracts, dimensions, seed, and complete resolved style. The semantic
encoded key additionally binds the format and semantic encoder-settings
contract. The build key additionally binds a caller-provided `EncoderBuildId`.
These keys are storage identifiers, not authentication values or secret
digests. A semantic key does not promise byte-identical output across encoder
dependency builds.

Alpha.5 automatic palette, expression, and accessory selection reuses separate
named family samples. Compatibility fallback consumes no additional hash state
and does not introduce mutable RNG state.

The default namespace is tenant `public` and style version `v2-alpha3`.
Alpha.5 intentionally rebases the prerelease pixel KAT after correcting the
catalog's first production-versus-candidate visual review. Trait derivation and
catalog IDs remain stable; canonical pixels do not.
Applications needing stable application-specific output should always pass an
explicit tenant and style version.
