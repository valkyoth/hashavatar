# Versioning Policy

`hashavatar` is intended to be safe for deterministic avatar URLs.

## Stable Rendering Contract

Within a major release line, the project aims to keep avatar output stable for the same:

- `tenant`
- `style_version`
- identity hash algorithm
- `id`
- `kind`
- `background`
- `accessory`
- `color`
- `expression`
- `shape`
- `format`
- `size`

That means an application can cache and embed avatar URLs without expecting silent visual churn during normal minor and patch upgrades.

Callers that use the legacy `AvatarOptions` API implicitly use `accessory =
none`, `color = default`, `expression = default`, and `shape = square`.

## When Output May Change

Visual output may change when:

- you intentionally change `style_version`
- you intentionally change `tenant`
- you intentionally change the identity hash algorithm
- you adopt a new major crate release with documented breaking visual changes
- a narrowly scoped rendering bug fix is required and documented

## Recommended Production Strategy

- treat `tenant` as your product or environment namespace
- treat `style_version` as your avatar rollout version, for example `v2`
- use the default SHA-512 algorithm unless you have explicitly chosen and
  documented another algorithm
- do not send raw user emails if you can avoid it
- prefer stable internal ids or a one-way hash as the public avatar id

## Regression Protection

The repository includes golden fingerprint regression tests. Hashavatar 1.3
also freezes one complete request, style, asset key, RGBA digest, and SVG digest
for every 1.x family in `tests/compatibility_corpus_v1.tsv`. Those checks are
meant to catch unintended contract or visual changes before release.

## Explicit Contract And Cache IDs

Hashavatar 1.2 names the frozen 1.x automatic-selection mapping as
`CatalogVersion::LEGACY_V1` and the renderer behavior as
`RenderContractId::LEGACY_V1`. Built-in catalog entries expose stable IDs and
weights; those values must not be reordered or changed during 1.x.

New caches should use `IdentityCacheKey`, `AvatarAssetKey`, and
`SemanticEncodedAssetKey` or `BuildEncodedAssetKey` according to the level of
artifact being stored. The latter
two prevent dimensions, seed, style, format, or encoder-setting changes from
reusing an incomplete cache key. Existing `cache_key()` strings remain stable
for compatibility.

Hashavatar 1.3 adds `AvatarRequest` and `PreparedAvatar` as an additive
migration path. Explicit styles are strict by default in that path; opt-in
`LegacyV1` preparation reports both requested and effective style values.
Prepared keys and output methods are bound to the same immutable request tuple.

## Exact 1.x Output After 2.0

The 2.0 renderer may intentionally change pixels. Applications that require
the exact frozen 1.x corpus should pin the latest 1.x release. A separate
compatibility renderer is not currently planned and will be considered only if
downstream demand justifies maintaining two engines. See
`docs/MIGRATION_2.0.md`.

## Exhaustive Public Enums

The public option enums in the 1.x crate are exhaustive and are not marked
`#[non_exhaustive]`. Their variant sets and `ALL` slices are therefore frozen
for the remainder of 1.x. New variants require a major release or a separate
additive API that leaves the existing enum unchanged.

## Cross-Platform Determinism

The current renderer uses floating-point geometry internally. Golden
fingerprints protect the release platform, but `hashavatar` does not yet claim
a formal bit-identical raster contract across every CPU architecture, compiler
backend, and optimization mode. Future core work should move critical geometry
to fixed-point arithmetic before making that stronger guarantee.
