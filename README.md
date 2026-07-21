<p align="center">
  <b>Canonical deterministic procedural avatars for Rust.</b><br>
  Stateless identity traits, validated fixed-point scenes, bounded CPU rasterization, and SVG from one source of truth.
</p>

<div align="center">
  <a href="https://crates.io/crates/hashavatar">Crates.io</a>
  |
  <a href="https://docs.rs/hashavatar">Docs.rs</a>
  |
  <a href="https://github.com/valkyoth/hashavatar/blob/main/docs/CURRENT_STATUS.md">Current Status</a>
  |
  <a href="https://github.com/valkyoth/hashavatar/blob/main/docs/SECURITY_CONTROLS.md">Security Controls</a>
  |
  <a href="https://github.com/valkyoth/hashavatar/blob/main/docs/PLAN_TOWARDS_2.0.md">2.0 Plan</a>
  |
  <a href="https://github.com/valkyoth/hashavatar/security/policy">Security</a>
</div>

<br>

<p align="center">
  <a href="https://github.com/valkyoth/hashavatar">
    <img src="https://raw.githubusercontent.com/valkyoth/hashavatar/main/.github/images/hashavatar.webp" alt="hashavatar Rust crate overview">
  </a>
</p>

# hashavatar

`hashavatar` is the recommended facade for deterministic avatar generation.
The `2.0.0-alpha.1` source tree proves one complete Cat workflow: bounded
identity input becomes independently derived traits, a private validated
Q16.16 scene, canonical straight-alpha RGBA8 pixels, and deterministic SVG.
Both outputs execute the same scene.

Alpha.1 is deliberately small. It has no codecs, image-library types, mutable
rendering RNG, filesystem API, CLI, server, or user-supplied SVG. Later 2.0
alphas will extend this proven architecture to the existing catalog and
separate optional packages.

## Release Status

The latest crates.io release is `1.3.0`, maintained on
[`release/1.3`](https://github.com/valkyoth/hashavatar/tree/release/1.3). The
`2.0.0-alpha.x` line uses named implementation-stop commits. Exact commit SHAs
are tested through GitHub and `hashavatar-website`; prereleases are neither
tagged nor uploaded to crates.io.

To test alpha.1 from a local checkout:

```toml
[dependencies]
hashavatar = { path = "../hashavatar" }
```

## Cat Vertical Slice

```rust
use hashavatar::CatRequest;

let prepared = CatRequest::with_namespace(
    256,
    256,
    0,
    b"tenant-a",
    b"website-v2-alpha1",
    b"user-123",
)?
.prepare()?;

let report = prepared.scene_report();
let rgba = prepared.render_rgba()?;
let svg = prepared.render_svg()?;

assert_eq!(rgba.dimensions(), (256, 256));
assert_eq!(rgba.pixels().len(), report.rgba_bytes());
assert!(svg.starts_with("<svg"));

# Ok::<(), hashavatar::CatError>(())
```

`CatRequest` borrows input only until `prepare()`. `PreparedCat` retains the
private scene, public named trait samples, and validated resource report, but
not the raw identifier, tenant, style-version, or identity digest.

## Contracts

- Dimensions are restricted to `64..=2048` per side.
- Identity input is restricted to 1024 bytes.
- Tenant and style-version components are restricted to 128 bytes each.
- Components are length-prefixed and SHA-512 domain separated.
- Traits are sampled independently by stable labels; adding a trait does not
  consume mutable RNG state or shift existing traits.
- Geometry is private signed Q16.16 fixed point with checked construction.
- Scenes contain at most 16 admitted commands in alpha.1.
- Scene validation runs before raster or SVG execution.
- Raster output is tightly packed, straight-alpha RGBA8.
- SVG numeric values are exact decimal representations of the same Q16.16
  scene values and are parser-tested as XML.
- First-party Rust code forbids `unsafe` and production panic-like paths.

`SceneReport::rgba_bytes()` gives the exact returned raster allocation.
`estimated_pixel_tests()` gives a conservative CPU-work value for application
admission and concurrency policy. A caller serving untrusted traffic must
still enforce process-wide concurrency and rate limits.

## Crates

| Package | Alpha.1 role |
| --- | --- |
| `hashavatar` | Recommended thin facade and stable import path |
| `hashavatar-core` | `no_std + alloc` identity, trait, scene, CPU RGBA8, and SVG core |

The private scene representation is not a general graphics API. Future format,
schema, heapless, and GPU packages will consume narrow reviewed boundaries
without making internal scene layout a compatibility promise.

## Security Boundaries

Hashavatar creates public visual artifacts; it is not an authentication or
password-hashing system. SHA-512 separates bounded namespace and identity
components but does not stop offline guessing of low-entropy identifiers.
Applications handling sensitive email addresses, usernames, or internal IDs
should pass a domain-separated keyed pseudonym rather than the original value.

`sanitization` guards temporary preimages and derived digest storage. Returned
RGBA bytes and SVG strings are caller-owned public output and are not wiped by
the crate. Allocation timing can reveal bounded input length, and render time
can reveal visual complexity. See [Security Controls](docs/SECURITY_CONTROLS.md)
for the exact guarantees and accepted limitations.

## Verification

```bash
scripts/checks.sh
scripts/check_kani.sh
cargo test --workspace --release
```

The standard gate checks formatting, strict Clippy, debug and release KATs,
rustdoc, MSRV `1.90.0`, package metadata, dependency and license policy,
RustSec, fuzz harness compilation, source size, unsafe boundaries, and
production panic policy. CI additionally compiles `hashavatar-core` for WASM,
AArch64 Linux, and 32-bit x86 Linux.

## License

Licensed under either [Apache License 2.0](LICENSE-APACHE) or
[MIT](LICENSE-MIT), at your option.
