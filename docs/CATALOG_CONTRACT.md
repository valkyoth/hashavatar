# Catalog Contract

`CATALOG_CONTRACT_ID` is `hashavatar/catalog/v2-alpha3`.

Alpha.3 contains exactly the visual catalog available in Hashavatar 1.3: 31
families, 13 backgrounds, and five frame shapes. It does not add accessories,
expressions, palettes, or new artwork. Those composition layers remain alpha.4
work.

## Stable Identifiers

`AvatarKind::catalog_id()`, `AvatarBackground::catalog_id()`, and
`AvatarShape::catalog_id()` preserve the corresponding 1.x identifier order.
The complete ordered arrays are exposed as `ALL`. `from_byte` selects through
that order without hard-coded modulo counts.

The IDs identify semantic choices, not 1.x pixels. Alpha.3 compiles every
choice through the new canonical fixed-point renderer and therefore requires a
new style-version and cache namespace.

Existing IDs must never be reordered or reused. Adding a future entry appends
an ID under a new catalog contract and requires mapping, aggregate-KAT,
migration, and release-note updates.

## Capabilities

Every family supports every alpha.3 background and frame. The immutable
`AVATAR_FAMILY_CAPABILITIES` manifest also states whether a family has face
anchors suitable for alpha.4 layers. It does not claim that accessories or
expressions are already implemented.

Object and symbol families without face anchors are paws, planet, rocket,
mushroom, cactus, cupcake, pizza, icecream, diamond, coffee-cup, and shield.
Callers must not infer capabilities from enum names or duplicate this list.

## Derivation

Alpha.3 styles are explicit. `from_byte` is a deterministic catalog utility,
not automatic style policy. Family geometry and color samples are derived with
separate labels scoped by the canonical family label. Adding a sample to one
family cannot advance mutable RNG state or shift another sample.

## Verification

The integration corpus executes all 2,015 family/background/frame
combinations, parses every SVG, checks bounded scene reports, validates
transparent caller-surface clearing, and freezes one SHA-512 aggregate over the
canonical family pixel digests. Contact-sheet examples support human visual
review of both SVG and CPU raster output.
