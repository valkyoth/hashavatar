# Pixel Contract

`PIXEL_CONTRACT_ID` is `hashavatar/rgba8-straight-srgb/v1`.

Canonical CPU output uses row-major, top-to-bottom, left-to-right RGBA8 in the
sRGB color space. Public bytes use straight alpha. Shape membership samples the
pixel center `(x + 0.5, y + 0.5)` in private Q16.16 coordinates. Internal
geometry, interpolation, alpha multiplication, and source-over compositing use
integer arithmetic with explicit nearest rounding; floating point is excluded.

For byte values in `0..=255`, `round255(n)` is `(n + 127) / 255` using integer
division. Applying opacity `o` changes source alpha `a` to
`round255(a * o)`. Source-over computes:

```text
out_a = src_a + round255(dst_a * (255 - src_a))
out_p = src_channel * src_a
      + round255(dst_channel * dst_a * (255 - src_a))
out_channel = (out_p + out_a / 2) / out_a
```

The final division is skipped when `out_a` is zero. Nested opacity scopes apply
the same rounded multiplication at each nesting boundary. Linear-gradient
channels, including alpha, use a clamped integer projection and nearest integer
interpolation between their two endpoint colors.

An output alpha of zero has canonical bytes `[0, 0, 0, 0]`. The first scene
command is an opaque full-surface fill, so canonical output does not depend on
the prior visible contents of a caller surface.

`RgbaSurfaceMut::new` accepts only dimensions in `64..=2048`, a stride at least
`width * 4`, and enough bytes for the last visible row. Execution changes only
the first `width * 4` bytes of each row. Padding and trailing bytes are
preserved. Zero-sized and dimension-mismatched surfaces are rejected.

`PixelDigest` is SHA-512 over, in order: the pixel contract ID bytes, little-
endian `u32` width, little-endian `u32` height, and each visible row. Stride,
padding, allocation capacity, and trailing bytes are excluded. Equal canonical
pixels therefore have equal digests across tight and padded surfaces.
Digesting streams each visible row through a fixed-size, clear-on-drop SHA-512
state and does not allocate a second image-sized preimage buffer.
