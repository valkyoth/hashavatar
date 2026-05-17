# hashavatar

`hashavatar` is a Rust crate for deterministic, procedural avatar generation. It is designed for services that need stable user or tenant avatars without bundled artwork, sprite sheets, external asset packs, or filesystem-side effects.

The crate starts conservative: validated avatar dimensions, bounded identity input, namespace-isolated hashing, safe Rust rendering, in-memory raster encoding, SVG string rendering, and a release process with dependency, audit, fuzz, package, SBOM, and reproducibility checks.

## Current Status

The current development version is `0.8.0`.

Implemented now:

- Pure library crate; no bundled demo server and no CLI binary.
- Deterministic avatars derived from SHA-512 identity hashes by default.
- Optional BLAKE3 and XXH3-128 identity derivation behind explicit Cargo
  features.
- Public enum variant lists use single-source `ALL` slices and byte-to-variant
  helpers for deterministic option derivation.
- Namespace-aware identity derivation for tenant isolation and visual rollouts.
- Length-prefixed hash components to avoid delimiter ambiguity.
- Avatar families: `cat`, `dog`, `robot`, `fox`, `alien`, `monster`, `ghost`, `slime`, `bird`, `wizard`, `skull`, `paws`, `planet`, `rocket`, `mushroom`, `cactus`, `frog`, `panda`, `cupcake`, `pizza`, `icecream`, `octopus`, and `knight`.
- Background modes: `themed`, `white`, `black`, `dark`, `light`, and `transparent`.
- In-memory `WebP`, `PNG`, `JPEG`, and `GIF` encoding.
- Compact SVG string rendering.
- Typed errors for invalid dimensions and oversized identity inputs.
- Private `AvatarSpec` fields so dimensions must pass construction-time validation.
- No public path-writing helpers; callers own their storage and filesystem boundary.
- `#![forbid(unsafe_code)]` in library code.
- Golden visual regression fingerprints.
- Isolated fuzz harness for avatar identities, families, backgrounds, SVG rendering, and PNG encoding.
- Local release gates for formatting, clippy, tests, docs, dependency policy, RustSec advisories, package contents, SBOM generation, reproducible build checks, and crates.io publish dry runs.

Planned or intentionally external:

- HTTP serving, rate limits, cache headers, security headers, observability, and abuse controls live in [`hashavatar-api`](https://github.com/valkyoth/hashavatar-api).
- Additional output formats such as AVIF or JPEG XL require dependency-policy review before admission.
- Larger identity inputs should be normalized or mapped by the application before calling this crate.

## Trust Dashboard

| Area | Status |
| --- | --- |
| License | `MIT OR Apache-2.0` |
| MSRV | Rust `1.95.0` |
| Crate shape | Library only |
| Runtime dependencies | `image`, `palette`, `rand`, `sha2`, `subtle`, `zeroize`; optional `blake3`, `xxhash-rust` |
| Unsafe policy | `#![forbid(unsafe_code)]` |
| Filesystem policy | No public path-writing APIs |
| Dimension limits | `64..=2048` pixels per side |
| Identity limits | 1024 bytes per identity input |
| Namespace limits | 128 bytes per tenant/style-version component |
| Hashing posture | SHA-512 default with length-prefixed domain, namespace, style, and identity components; optional BLAKE3 and non-cryptographic XXH3-128 |
| SVG posture | Generated numeric markup only; caller input is not inserted into SVG fragments |
| Release evidence | fmt, clippy, tests, docs, deny, audit, fuzz harness compile, package check, SBOM, reproducibility |

Security-control details live in [docs/SECURITY_CONTROLS.md](docs/SECURITY_CONTROLS.md). Dependency policy lives in [docs/DEPENDENCIES.md](docs/DEPENDENCIES.md). Panic policy lives in [docs/PANIC_POLICY.md](docs/PANIC_POLICY.md).

Future version planning for possible `no_std + alloc` support, visual layers,
and 1.0 stabilization lives in [docs/VERSION_PLAN.md](docs/VERSION_PLAN.md).
`0.8.0` prepares the internal boundary for a future core crate, but `no_std`
is not a public support contract yet.

## Install

```toml
[dependencies]
hashavatar = "0.8.0"
```

Optional identity hash algorithms are disabled by default:

```toml
[dependencies]
hashavatar = { version = "0.8.0", features = ["blake3", "xxh3"] }
```

For a local checkout:

```toml
[dependencies]
hashavatar = { path = "../hashavatar" }
```

The crate is dual-licensed:

```toml
license = "MIT OR Apache-2.0"
```

## Limits

| Limit | Value |
| --- | --- |
| Minimum width/height | `64` |
| Maximum width/height | `2048` |
| Maximum raster pixels | `4,194,304` |
| Maximum raw RGBA buffer | `16,777,216` bytes |
| Maximum identity input | `1024` bytes |
| Maximum namespace tenant | `128` bytes |
| Maximum namespace style version | `128` bytes |

These limits are enforced by constructors and render entry points. They are intended to make the safe path the normal path for public web endpoints.

## Example: Encode WebP

```rust
use hashavatar::{
    AvatarBackground, AvatarKind, AvatarOptions, AvatarOutputFormat, AvatarSpec,
    encode_avatar_for_id,
};

let spec = AvatarSpec::new(256, 256, 0)?;
let bytes = encode_avatar_for_id(
    spec,
    "robot@hashavatar.app",
    AvatarOutputFormat::WebP,
    AvatarOptions::new(AvatarKind::Robot, AvatarBackground::Transparent),
)?;

assert!(!bytes.is_empty());

# Ok::<(), Box<dyn std::error::Error>>(())
```

The returned bytes can be sent as an HTTP response, uploaded to object storage, written to a caller-selected path, or cached by a CDN.

## Example: Render SVG

```rust
use hashavatar::{
    AvatarBackground, AvatarKind, AvatarOptions, AvatarSpec, render_avatar_svg_for_id,
};

let spec = AvatarSpec::new(256, 256, 0)?;
let svg = render_avatar_svg_for_id(
    spec,
    "alien@hashavatar.app",
    AvatarOptions::new(AvatarKind::Alien, AvatarBackground::Transparent),
)?;

assert!(svg.starts_with("<svg "));
assert!(svg.contains("alien avatar"));

# Ok::<(), Box<dyn std::error::Error>>(())
```

Use SVG when you need vector output, easy inspection, text storage, or post-processing by application code.

## Example: Namespaced Tenants

```rust
use hashavatar::{
    AvatarBackground, AvatarKind, AvatarNamespace, AvatarOptions, AvatarOutputFormat,
    AvatarSpec, encode_avatar_for_namespace,
};

let namespace = AvatarNamespace::new("customer-a", "v2")?;
let spec = AvatarSpec::new(256, 256, 0)?;

let bytes = encode_avatar_for_namespace(
    spec,
    namespace,
    "user-123",
    AvatarOutputFormat::Png,
    AvatarOptions::new(AvatarKind::Cat, AvatarBackground::Themed),
)?;

assert!(!bytes.is_empty());

# Ok::<(), Box<dyn std::error::Error>>(())
```

Use namespaces when the same user identifier must not collide visually across tenants, products, or style-version rollouts.

## Example: Deterministic Options From Bytes

```rust
use hashavatar::{AvatarBackground, AvatarKind, AvatarOptions};

let digest_bytes = [42_u8, 199_u8];
let options = AvatarOptions::new(
    AvatarKind::from_byte(digest_bytes[0]),
    AvatarBackground::from_byte(digest_bytes[1]),
);

assert!(AvatarKind::ALL.contains(&options.kind));
assert!(AvatarBackground::ALL.contains(&options.background));
```

The `from_byte` helpers use each enum's `ALL` slice, so new public variants do
not require duplicated modulo constants in caller code.

## Example: Optional Hash Algorithm

```rust
use hashavatar::{
    AvatarBackground, AvatarHashAlgorithm, AvatarIdentityOptions, AvatarKind,
    AvatarNamespace, AvatarOptions, AvatarSpec, render_avatar_with_identity_options,
};

let namespace = AvatarNamespace::new("customer-a", "v3")?;
let identity_options = AvatarIdentityOptions::new(
    namespace,
    AvatarHashAlgorithm::Sha512,
);
let spec = AvatarSpec::new(128, 128, 0)?;

let image = render_avatar_with_identity_options(
    spec,
    identity_options,
    "user-123",
    AvatarOptions::new(AvatarKind::Robot, AvatarBackground::Themed),
)?;

assert_eq!(image.width(), 128);

# Ok::<(), Box<dyn std::error::Error>>(())
```

`AvatarHashAlgorithm::Sha512` is always available and is the security-sensitive
default. `AvatarHashAlgorithm::Blake3` is available with the `blake3` feature.
`AvatarHashAlgorithm::Xxh3_128` is available with the `xxh3` feature and is
non-cryptographic. Do not use XXH3-128 for adversarial or user-controlled
identifiers unless the application first maps those identifiers through its own
cryptographic boundary.

### BLAKE3 Feature Example

```toml
[dependencies]
hashavatar = { version = "0.8.0", features = ["blake3"] }
```

```rust
use hashavatar::{
    AvatarBackground, AvatarHashAlgorithm, AvatarIdentityOptions, AvatarKind,
    AvatarNamespace, AvatarOptions, AvatarSpec, render_avatar_svg_with_identity_options,
};

let namespace = AvatarNamespace::new("customer-a", "v3")?;
let spec = AvatarSpec::new(256, 256, 0)?;

let svg = render_avatar_svg_with_identity_options(
    spec,
    AvatarIdentityOptions::new(namespace, AvatarHashAlgorithm::Blake3),
    "user-123",
    AvatarOptions::new(AvatarKind::Alien, AvatarBackground::Themed),
)?;

assert!(svg.contains("alien avatar"));

# Ok::<(), Box<dyn std::error::Error>>(())
```

### XXH3-128 Feature Example

```toml
[dependencies]
hashavatar = { version = "0.8.0", features = ["xxh3"] }
```

```rust
use hashavatar::{
    AvatarBackground, AvatarHashAlgorithm, AvatarIdentityOptions, AvatarKind,
    AvatarNamespace, AvatarOptions, AvatarOutputFormat, AvatarSpec,
    encode_avatar_with_identity_options,
};

let namespace = AvatarNamespace::new("public-demo", "v3")?;
let spec = AvatarSpec::new(256, 256, 0)?;

let bytes = encode_avatar_with_identity_options(
    spec,
    AvatarIdentityOptions::new(namespace, AvatarHashAlgorithm::Xxh3_128),
    "demo-user-123",
    AvatarOutputFormat::WebP,
    AvatarOptions::new(AvatarKind::Robot, AvatarBackground::Themed),
)?;

assert!(!bytes.is_empty());

# Ok::<(), Box<dyn std::error::Error>>(())
```

XXH3-128 is fast and useful for non-adversarial distribution, but it is not a
cryptographic hash. Keep SHA-512 or BLAKE3 for adversarial or user-controlled
identity inputs.

## Example: Raw Image Buffer

```rust
use hashavatar::{
    AvatarBackground, AvatarKind, AvatarOptions, AvatarSpec, render_avatar_for_id,
};

let spec = AvatarSpec::new(128, 128, 0)?;
let image = render_avatar_for_id(
    spec,
    "fox@hashavatar.app",
    AvatarOptions::new(AvatarKind::Fox, AvatarBackground::Themed),
)?;

assert_eq!(image.width(), 128);
assert_eq!(image.height(), 128);

# Ok::<(), Box<dyn std::error::Error>>(())
```

Use raw buffers when the caller wants to composite, inspect pixels, run custom encoding, or integrate with an existing image pipeline.

## Handling Untrusted Input

```rust
use hashavatar::{
    AvatarBackground, AvatarKind, AvatarOptions, AvatarOutputFormat, AvatarSpec,
    encode_avatar_for_id,
};

fn avatar_response_bytes(user_id: &str, requested_size: u32) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let spec = AvatarSpec::new(requested_size, requested_size, 0)?;
    let options = AvatarOptions::new(AvatarKind::Cat, AvatarBackground::Transparent);

    encode_avatar_for_id(spec, user_id, AvatarOutputFormat::WebP, options)
        .map_err(Into::into)
}
```

The crate rejects unsupported sizes and oversized identities. Applications
should still enforce their own routing, authentication, rate limiting, cache
policy, response headers, request body limits, and concurrency limits. A single
maximum-size raster render needs up to `MAX_AVATAR_RGBA_BYTES` raw RGBA bytes
before encoder overhead, so public services should bound simultaneous large
renders at the API layer.

## API Reference Summary

Important public entry points:

- `AvatarSpec::new(width, height, seed) -> Result<AvatarSpec, AvatarSpecError>`
- `AvatarIdentity::new(input) -> Result<AvatarIdentity, AvatarIdentityError>`
- `AvatarIdentity::new_with_options(options, input) -> Result<AvatarIdentity, AvatarIdentityError>`
- `AvatarIdentityOptions::new(namespace, algorithm)`
- `AvatarNamespace::new(tenant, style_version) -> Result<AvatarNamespace, AvatarIdentityError>`
- `AvatarOptions::new(kind, background)`
- `encode_avatar_for_id(...)`
- `encode_avatar_for_namespace(...)`
- `render_avatar_for_id(...)`
- `render_avatar_for_namespace(...)`
- `render_avatar_with_identity_options(...)`
- `render_avatar_svg_for_id(...)`
- `render_avatar_svg_for_namespace(...)`
- `render_avatar_svg_with_identity_options(...)`

Lower-level identity-specific renderers are available for callers that want direct control over a specific avatar family.

## Output Formats

| Format | API value | Notes |
| --- | --- | --- |
| WebP | `AvatarOutputFormat::WebP` | Recommended default for modern web delivery. |
| PNG | `AvatarOutputFormat::Png` | Lossless and broadly compatible. |
| JPEG | `AvatarOutputFormat::Jpeg` | Transparent pixels are composited over white. |
| GIF | `AvatarOutputFormat::Gif` | Legacy-compatible single-frame output. |
| SVG | `render_avatar_svg_*` | Returns a string rather than raster bytes. |

AVIF and JPEG XL are not exposed because they add dependency or encoder maturity tradeoffs that have not cleared the crate's dependency policy.

## Determinism

The output is deterministic for the tuple:

```text
identity hash algorithm + namespace tenant + namespace style version + identity bytes + avatar kind + background + dimensions + seed
```

This makes the crate suitable for stable CDN-backed avatar URLs and golden regression tests. Namespace hashing uses length-prefixed components, so embedded separator bytes cannot create tenant/style-version ambiguity. The default SHA-512 path keeps the pre-0.7 identity preimage stable; non-default algorithms are domain-separated.

The renderer uses floating-point geometry internally. The project tests golden
fingerprints on the release platform, but it does not yet claim formal
bit-identical raster output across every CPU architecture, compiler backend,
and optimization mode. Future core-boundary work tracks fixed-point geometry
as the path to a stricter cross-platform determinism contract.

The procedural cat renderer seeds its internal RNG from bytes `32..64` of the
identity digest and uses the lower digest bytes for direct visual parameters.
That keeps RNG state separate from directly observed parameter bytes. The
change intentionally updates cat-family golden fingerprints in `0.7.0`.

`AvatarIdentity` equality uses constant-time digest comparison. Rendering and
encoding are not constant-time: shape counts, geometry, encoded size, and SVG
length can vary with identity digest bytes. Applications with strict side
channel requirements should not treat avatar render timing or output size as
secret-preserving signals.

When identity values are sensitive and an API must reduce render-time
observability, add the mitigation at the service boundary where request timing
is controlled:

```rust
use std::time::{Duration, Instant};

use hashavatar::{
    AvatarBackground, AvatarKind, AvatarOptions, AvatarSpec, render_avatar_for_id,
};

fn render_with_min_latency(
    id: &str,
    target_latency: Duration,
) -> Result<image::RgbaImage, Box<dyn std::error::Error>> {
    let started = Instant::now();
    let spec = AvatarSpec::new(256, 256, 0)?;
    let result = render_avatar_for_id(
        spec,
        id,
        AvatarOptions::new(AvatarKind::Monster, AvatarBackground::Themed),
    );

    let elapsed = started.elapsed();
    if elapsed < target_latency {
        std::thread::sleep(target_latency - elapsed);
    }

    Ok(result?)
}
```

For public web services, prefer CDN caching and stable cache keys so repeated
requests for the same avatar do not repeatedly expose renderer timing. In async
servers, use an async timer rather than blocking a runtime worker thread.

Encode APIs clear temporary raster buffers after encoding. Returned `Vec<u8>`
encoded bytes and `RgbaImage` render outputs are caller-owned; applications
with strict memory-sanitization requirements should clear those buffers after
use.

## Testing And Release Evidence

The repository includes:

- same-input stability tests
- different-input divergence tests
- raster export round-trip tests
- SVG safety and compactness tests
- enum parsing tests
- transparent background checks
- golden visual fingerprint tests
- fuzz harness compilation
- `cargo deny` policy
- RustSec advisory scanning
- reproducible package/build checks
- SBOM generation
- crates.io publish dry run

Run the standard local gate:

```bash
scripts/checks.sh
```

Run the fuller release gate:

```bash
scripts/stable_release_gate.sh check
```

## Provenance

The repository is intended to remain code-generated and asset-free. For a direct statement of how the visuals are produced, see [PROVENANCE.md](PROVENANCE.md).

## Web API And Demo

The crate is focused on reusable rendering code. The public HTTP API and demo website live in the separate [`hashavatar-api`](https://github.com/valkyoth/hashavatar-api) project.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) and the release note files for version-by-version details.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))
