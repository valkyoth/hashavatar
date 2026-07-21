# Catalog Contract

`CATALOG_CONTRACT_ID` is `hashavatar/catalog/v2-alpha4`.

Alpha.4 retains the Hashavatar 1.3 catalog of 31 families, 13 backgrounds, five
frame shapes, nine non-empty accessories, six palettes, and eight expressions.
It changes composition semantics by replacing the single optional accessory
with a bounded typed stack. It does not add new family artwork.

## Stable Identifiers

`AvatarKind::catalog_id()`, `AvatarBackground::catalog_id()`,
`AvatarShape::catalog_id()`, and `AvatarAccessory::catalog_id()` preserve the
corresponding 1.x identifier order. Palettes and expressions retain their 1.x
order beginning at zero.
The complete ordered arrays are exposed as `ALL`. `from_byte` selects through
that order without hard-coded modulo counts.

The IDs identify semantic choices, not 1.x pixels. The canonical fixed-point
renderer therefore requires a new style-version and cache namespace. Alpha.5
corrected the prerelease port's palettes and recognizable family geometry after
its first comparison with the deployed 1.3 website; this intentionally changed
2.0 prerelease pixels without changing semantic IDs.

Existing IDs must never be reordered or reused. Adding a future entry appends
an ID under a new catalog contract and requires mapping, aggregate-KAT,
migration, and release-note updates.

## Capabilities

Every family supports every background, frame, and palette. The immutable
`AVATAR_FAMILY_CAPABILITIES` manifest states whether a family has face anchors
and therefore supports currently admitted accessory slots and expressions.

Object and symbol families without face anchors are paws, planet, rocket,
mushroom, cactus, cupcake, pizza, icecream, diamond, coffee-cup, and shield.
Callers must not infer capabilities from enum names or duplicate this list.

## Derivation

Explicit styles are strict. `AvatarStyle::automatic` derives palette,
expression, and two accessory requests from separate existing labeled samples,
then uses the frozen fallback policy in `LAYERED_STYLE_CONTRACT.md`. Family
geometry and color samples remain separately scoped by family label.

## Verification

The integration corpus retains the 2,015 family/background/frame matrix and
adds every family/palette, family/expression, and family/accessory combination,
strict and automatic resolution cases, permutation invariance, maximum stacks,
pixel-distinct representative choices, ordered per-family fingerprints, and a
full layered visual corpus. Every catalog-port milestone must also compare full
raster and SVG contact sheets with the deployed 1.3 catalog; checksums prove
stability after review, not visual quality by themselves.
