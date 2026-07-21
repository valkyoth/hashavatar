# Digest And Derivation Contract

Alpha.2 identity derivation uses SHA-512 and these ASCII domains:

- `hashavatar/identity/v2/sha512/v1`
- `hashavatar/trait/v2/sha512/v1`

Identity preimages contain domain, tenant, style version, and caller input in
that order. Every component is prefixed by its little-endian `u64` byte length.
Trait preimages contain trait domain, the 64-byte identity digest, little-
endian `u64` style seed, the length-prefixed trait label, and a little-endian
`u32` counter. Alpha.2 currently uses counter zero and interprets the first two
digest bytes as a little-endian `u16` sample.

Labels are independent: adding or evaluating one trait does not consume shared
state or shift another trait. Identity digests are not public and are not cache
keys. Pixel digests use a separate domain and construction documented in
[PIXEL_CONTRACT.md](PIXEL_CONTRACT.md). Future scene, asset, or schema keys must
use their own domains rather than reusing identity or pixel digest bytes.

The default namespace is tenant `public` and style version `v2-alpha2`.
Applications needing stable application-specific output should always pass an
explicit tenant and style version.
