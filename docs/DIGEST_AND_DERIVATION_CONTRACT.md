# Digest And Derivation Contract

Alpha.4 identity derivation retains the SHA-512 domains introduced by the
canonical renderer:

- `hashavatar/identity/v2/sha512/v1`
- `hashavatar/trait/v2/sha512/v1`

Identity preimages contain domain, tenant, style version, and caller input in
that order. Every component is prefixed by its little-endian `u64` byte length.
Trait preimages contain trait domain, the 64-byte identity digest, little-
endian `u64` style seed, the length-prefixed trait label, and a little-endian
`u32` counter. Alpha.4 currently uses counter zero and interprets the first two
digest bytes as a little-endian `u16` sample.

Labels are independent: adding or evaluating one trait does not consume shared
state or shift another trait. Identity digests are not public and are not cache
keys. Pixel digests use a separate domain and construction documented in
[PIXEL_CONTRACT.md](PIXEL_CONTRACT.md). Future scene, asset, or schema keys must
use their own domains rather than reusing identity or pixel digest bytes.

Alpha.4 automatic palette, expression, and accessory selection reuses separate
named family samples. Compatibility fallback consumes no additional hash state
and does not introduce mutable RNG state.

The default namespace is tenant `public` and style version `v2-alpha3` so
layer-free alpha.3 KATs remain stable through alpha.4.
Applications needing stable application-specific output should always pass an
explicit tenant and style version.
