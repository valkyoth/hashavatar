# hashavatar 0.10.0

`0.10.0` introduces the visual layer model while keeping the existing
`AvatarOptions` rendering path stable.

## Added

- `AvatarAccessory` with `none`, `glasses`, `hat`, `headphones`, `crown`,
  `bowtie`, `eyepatch`, `scarf`, `halo`, and `horns`.
- `AvatarColor` with `default`, `neon-mint`, `pastel-pink`, `crimson`, `gold`,
  and `deep-sea-blue`.
- `AvatarExpression` with `default`, `happy`, `grumpy`, `surprised`, `sleepy`,
  `winking`, `cool`, and `crying`.
- `AvatarShape` with `square`, `circle`, `squircle`, `hexagon`, and `octagon`.
- `AvatarStyleOptions` for explicit kind, background, accessory, color,
  expression, and frame-shape selection.
- Style-aware raster, SVG, and encode APIs:
  - `render_avatar_style_for_id`
  - `render_avatar_svg_style_for_id`
  - `encode_avatar_style_for_id`
  - namespace and identity-options variants for each
- Automatic style APIs:
  - `render_avatar_auto_for_id`
  - `render_avatar_svg_auto_for_id`
  - `encode_avatar_auto_for_id`
  - namespace and identity-options variants for each
- Public digest-byte constants for automatic style derivation.
- Family-aware face anchors for accessory and expression placement.
- README option catalog covering all public enum values and the difference
  between `AvatarOptions` and `AvatarStyleOptions`.

## Compatibility

Existing `AvatarOptions` output is unchanged. Legacy options map to
`accessory = none`, `color = default`, `expression = default`, and
`shape = square`.

Automatic style rendering is opt-in. It derives kind, background, accessory,
color, expression, and shape from distinct identity digest bytes using each
enum's `ALL` list.

Accessories and expressions are deterministic no-ops for non-face avatar
families where the layer cannot be placed sensibly. For example, `paws` with an
eyepatch renders the baseline paws avatar rather than placing a patch at an
arbitrary canvas position.

## Security And Testing

- README security guidance now includes explicit caller-side zeroization and
  service-side render-concurrency examples for high-assurance deployments.
- Fuzz coverage now exercises the style-aware render and encode APIs.
- Golden fingerprints cover representative layered avatars and automatic style
  output.
- Tests enforce parser/display round trips, enum `ALL` drift protection,
  distinct automatic digest offsets, manual layer selection, and raster/SVG
  support for all baseline layer variants.
- Tests enforce deterministic no-op behavior for unsupported family/layer
  combinations.
