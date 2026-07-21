# SVG Contract

SVG is a semantic execution of the same validated private scene as canonical
RGBA8. It promises deterministic geometry, paint, command ordering, grouping,
and identifiers. Browser raster pixels are not canonical and need not equal the
CPU rasterizer because browser antialiasing and color processing vary.

`SvgOptions::document` emits a complete accessible `<svg>` with caller title
and description escaped as XML text. `SvgOptions::fragment` emits a `<g>` and
leaves accessibility to the host document. Identifier prefixes are 1 to 64
ASCII bytes, begin with a letter, and then contain only letters, digits, `-`, or
`_`. Generated gradient and clip IDs combine that prefix, role, and command
index, so separate prefixes prevent collisions in a shared document.

Q16.16 values are serialized as locale-independent exact decimal values.
Colors and markup structure come only from validated scene values. Caller text
is accepted only in document accessibility fields and is escaped. Unsupported
options fail before output. Streaming writer failure follows
[FAILURE_CONTRACT.md](FAILURE_CONTRACT.md).
