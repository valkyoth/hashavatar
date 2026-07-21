<p align="center">
  <b>Feature-gated established image encoders for Hashavatar.</b><br>
  Canonical core pixels in; typed WebP, PNG, JPEG, or GIF assets out.
</p>

<div align="center">
  <a href="https://github.com/valkyoth/hashavatar">Hashavatar</a>
  |
  <a href="https://docs.rs/hashavatar-formats">Docs.rs</a>
  |
  <a href="https://github.com/valkyoth/hashavatar/blob/main/docs/FORMAT_CONTRACT.md">Format Contract</a>
  |
  <a href="https://github.com/valkyoth/hashavatar/blob/main/docs/SECURITY_CONTROLS.md">Security Controls</a>
</div>

<br>

<p align="center">
  <a href="https://github.com/valkyoth/hashavatar">
    <img src="https://raw.githubusercontent.com/valkyoth/hashavatar/main/.github/images/hashavatar.webp" alt="hashavatar Rust crate overview">
  </a>
</p>

# hashavatar-formats

`hashavatar-formats` isolates standard-library codec dependencies from the
portable `hashavatar-core` renderer. Most applications should enable formats
through the `hashavatar` facade.

The package is source-only during the 2.0 prerelease cycle:

```toml
[dependencies]
hashavatar-formats = { path = "../hashavatar/crates/hashavatar-formats" }
```

## Example

```rust
use hashavatar_core::{
    AvatarBackground, AvatarIdentity, AvatarKind, AvatarRequest, AvatarShape,
    AvatarStyle,
};
use hashavatar_formats::{AvatarOutputFormat, encode};

let identity = AvatarIdentity::new(b"user-123")?;
let style = AvatarStyle::new(
    AvatarKind::Robot,
    AvatarBackground::Ocean,
    AvatarShape::Circle,
);
let prepared = AvatarRequest::builder(identity)
    .size(256, 256)
    .style(style)
    .prepare()?;
let encoded = encode(&prepared, AvatarOutputFormat::WebP)?;

assert_eq!(encoded.metadata().media_type(), "image/webp");
assert!(!encoded.bytes().is_empty());
# Ok::<(), Box<dyn std::error::Error>>(())
```

The default feature enables lossless WebP. `png`, `jpeg`, and `gif` are
explicit features; `all-formats` enables all four. JPEG deterministically
flattens alpha over white. GIF performs palette quantization. Codec-owned
temporary buffers cannot be sanitized or completely bounded by this crate.

## Features

| Feature | Default | Encoding contract |
| --- | --- | --- |
| `webp` | Yes | Lossless straight-alpha RGBA WebP |
| `png` | No | Lossless RGBA, best compression and adaptive filtering |
| `jpeg` | No | Quality 92 after integer alpha flattening over white |
| `gif` | No | One speed-1 quantized palette frame |
| `all-formats` | No | Enables all established codecs |

Format enum variants remain available in every build. Selecting a disabled
variant returns `FormatError::FormatDisabled` before rendering or writing.

## Writer And Key Boundary

`encode` returns owned bytes and immutable metadata. `encode_to_writer`
streams to `std::io::Write`; failures may leave a partial prefix in the
caller-owned writer. `encode_to_writer_with_scratch` additionally reuses a
`ReusableRgbaBuffer` supplied by the caller.

`SemanticEncodedAssetKey` binds the canonical asset and semantic encoder
settings. `BuildEncodedAssetKey` additionally binds a caller-provided
`EncoderBuildId`. Use the build-bound key or hash final bytes when exact output
identity must survive dependency or toolchain changes.

## Security And Resources

- Canonical rendering and dimension validation remain in `hashavatar-core`.
- Hashavatar-owned failed output, reusable RGBA, and JPEG conversion buffers
  are sanitized; successful output is caller-owned public data.
- `FormatResourceBudget` reports canonical and Hashavatar-owned conversion
  storage. Upstream codec scratch is not completely bounded.
- Applications serving untrusted traffic must limit concurrent renders and
  include encoded output, codec scratch, and network buffers in admission.
- WebP and PNG are lossless. JPEG and GIF are deterministic only within the
  admitted provider/settings build and have explicit lossy semantics.

See the workspace [format contract](https://github.com/valkyoth/hashavatar/blob/main/docs/FORMAT_CONTRACT.md),
[security controls](https://github.com/valkyoth/hashavatar/blob/main/docs/SECURITY_CONTROLS.md),
and [current status](https://github.com/valkyoth/hashavatar/blob/main/docs/CURRENT_STATUS.md).

## License

Licensed under either Apache-2.0 or MIT, at your option.
