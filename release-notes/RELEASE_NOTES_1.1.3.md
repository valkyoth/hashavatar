# hashavatar 1.1.3

`1.1.3` is a security-hardening, maintenance, and release-policy correction
release. It prepares the remaining 1.x migration line without changing the
public Rust API, Cargo feature names, raster rendering, or raster fingerprints.

## Security And Reliability

- Routed raster encoders through a sanitizing writer that controls allocation
  growth and clears a retired allocation before replacing it.
- Added strict dimension and RGBA-length validation for output returned by
  custom `AvatarRenderer` implementations before encoding.
- Namespaced SVG paint-server and clip-path IDs using only dimensions, frame
  shape, and background. Differing definitions cannot cross-resolve, and SVG
  output does not disclose an identity-derived cache key.
- Made stable release mode require exactly `cargo-kani 0.67.0`, its documented
  Rust `1.90.0` verifier toolchain, and `cargo-sbom 0.10.0`.
- Replaced filename-list reproducibility evidence with two isolated package
  builds whose non-overrideable target directories are created under a fresh
  temporary root and whose complete `.crate` archives must be byte-identical.

## Documentation Corrections

- Corrected the admitted Kani harness count from four to five.
- Added a release-metadata check that compares the documented count with the
  `#[kani::proof]` inventory in `src/kani_proofs.rs`.
- Clarified that `AvatarKind`, `AvatarBackground`, `AvatarAccessory`,
  `AvatarColor`, `AvatarExpression`, `AvatarShape`, and `AvatarOutputFormat`
  are exhaustive public enums and are not marked `#[non_exhaustive]`.
- Froze those enum variant sets for the remainder of 1.x. Adding variants is a
  breaking change reserved for 2.0 or requires a separate additive API.

## Roadmap

- Made `docs/PLAN_TOWARDS_2.0.md` the accepted preparation and 2.0 roadmap.
- Marked `docs/VERSION_PLAN.md` as historical context for the path from 0.6 to
  the current crate.
- Documented that `hashavatar` remains one crate for `1.1.3`, `1.2.0`, and
  `1.3.0`; the workspace split starts at `2.0.0-alpha.1`.
- Kept JPEG XL unplanned. GPU, AVIF, schema, and heapless storage remain
  explicit 2.0 deliverables behind optional crate or feature boundaries.
- Updated service-boundary links for the renamed `hashavatar-website` hosted
  reference implementation.

## Compatibility

- No public API or Cargo feature changes.
- Updated `sanitization` and `sanitization-crypto-interop` from `1.2.4` to
  `1.2.5`, `serde` from `1.0.228` to `1.0.229`, optional `xxhash-rust` from
  `0.8.16` to `0.8.17`, and development `serde_json` from `1.0.150` to
  `1.0.151`. Compatible transitive lockfile dependencies were also refreshed.
- No intentional raster, identity, cache-key, or encoded-output changes; golden
  raster fingerprints remain unchanged.
- SVG definition IDs intentionally change without embedding identity-derived
  material. SVG geometry and appearance remain unchanged, but consumers
  comparing complete SVG strings must account for the corrected namespacing.
- Rust `1.90.0` remains the MSRV and Rust `1.97.1` remains the pinned
  development toolchain.

## Verification

Passed the complete fail-closed stable release gate: formatting, metadata,
dependency boundaries, unsafe and panic policies, Clippy, 98 unit tests with
one explicitly ignored diagnostic test, the doctest, MSRV feature combinations,
documentation, cargo-deny, RustSec audit, fuzz harness compilation, all five
bounded Kani harnesses, byte-identical `.crate` packaging, SBOM generation, and
a crates.io publish dry run.

The runtime and release-control state at commit `7d0cb53` completed the final
independent pentest retest without an unaccepted finding. The subsequent
release-readiness commit changes documentation only before GitHub validation.
