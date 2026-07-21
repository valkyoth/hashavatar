# Established Format Contract

Alpha.5 introduces `hashavatar-formats` as the standard-library codec boundary.
It consumes only a validated `PreparedAvatar` and canonical straight-alpha
RGBA8 output from `hashavatar-core`. Codecs cannot author or mutate scenes.

## Features

| Feature | Format | Fixed settings | Alpha | Pixel class |
| --- | --- | --- | --- | --- |
| `webp` | WebP | image-rs lossless VP8L | Full | Lossless |
| `png` | PNG | Best compression, adaptive filter | Full | Lossless |
| `jpeg` | JPEG | Quality 92, alpha flattened over white | None | Lossy |
| `gif` | Single-frame GIF | Speed 1 palette quantization | Binary | Lossy |

The facade and formats package enable only `webp` by default. `png`, `jpeg`,
and `gif` are explicit opt-ins. `all-formats` enables all four. Format enum
variants always exist, but selecting a disabled format returns
`FormatError::FormatDisabled` before rendering or writing.

The alpha.5 provider is `image-rs/image/0.25.10`. `hashavatar-core` and the
no-default facade do not depend on `image` or a codec.

## Encoding

`encode` returns owned `EncodedAvatar` bytes and metadata. `encode_to_writer`
streams to `std::io::Write`; a writer or codec error may leave an arbitrary
encoded prefix and retry requires a fresh destination.
`encode_to_writer_with_scratch` reuses caller-retained `ReusableRgbaBuffer`
storage. That scratch remains public caller-owned output after success or
failure until cleared or dropped.

WebP and PNG decode exactly to canonical RGBA8 in the admitted test corpus.
JPEG first applies, per channel:

```text
(channel * alpha + 255 * (255 - alpha) + 127) / 255
```

and then encodes RGB at quality 92. GIF quantizes RGBA to one palette frame.
JPEG/GIF decoded pixels are tested against explicit lossy evidence bounds, not
promised byte-for-byte equality with canonical RGBA.

## Metadata And Keys

Completion metadata includes format, media type, extension, encoded length,
alpha capability, provider, encoder contract, resource information, and a
`SemanticEncodedAssetKey`.

The semantic key uses SHA-512 domain
`hashavatar/encoded-semantic-key/v2/sha512/v1`, length-prefixed components,
and the first 32 digest bytes. It binds the canonical `AvatarAssetKey` and
format/settings contract. It does not claim encoded-byte identity across
provider builds.

`BuildEncodedAssetKey` uses domain
`hashavatar/encoded-build-key/v2/sha512/v1` and additionally binds a caller-
supplied `EncoderBuildId`. Applications must change that ID when lockfiles,
targets, build flags, or encoder implementation details can change bytes. For
content-addressable storage, hash the returned bytes instead.

## Resource And Cleanup Boundary

`FormatResourceBudget` reports exact canonical RGBA bytes and exact additional
Hashavatar-owned JPEG RGB conversion bytes. image-rs does not expose complete
codec scratch bounds, so `codec_scratch_is_bounded()` is false. Applications
must include conservative codec and output overhead in concurrency policy.

Hashavatar-owned temporary RGBA storage, failed owned-output buffers, replaced
output allocations, and JPEG conversion buffers are sanitized. Successful
encoded bytes are transferred to the caller and are not secret containers.
Codec-owned compression/quantization buffers, allocator internals, writer
buffers, registers, crash dumps, paging, and process abort remain outside
Hashavatar's cleanup control. GIF in particular creates an internal
quantization copy that Hashavatar cannot sanitize.
