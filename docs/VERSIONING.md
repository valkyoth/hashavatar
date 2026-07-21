# Versioning Policy

## Supported Lines

- `1.3.x` is the maintained published line on `release/1.3`.
- `2.0.0-alpha.x`, beta, and RC names identify implementation-stop commits;
  they are not Git tags or crates.io releases.
- Stable `2.0.0` will resume crates.io publication after its package and
  compatibility contracts are frozen.

## Deterministic Output

Within a stable major release, canonical output is intended to remain stable
for the complete documented identity, namespace, request, catalog, render, and
output contract. A patch or minor release must not silently change canonical
pixels or SVG unless a security/correctness fix requires it and the release
notes explicitly identify the change.

Prerelease APIs and pixels may change. Every intentional change must update
known-answer tests, release notes, migration guidance, and cache-domain
documentation. Callers must use a separate style-version and cache namespace
for each 2.0 prerelease trial.

## Alpha.1 Contract

Alpha.1 records deterministic evidence for one Cat scene:

- SHA-512 identity and label-separated trait vectors;
- checked Q16.16 geometry and command order;
- canonical safe-Rust CPU RGBA8 output;
- deterministic SVG from the same scene;
- identical debug/release pixel fingerprint for the pinned fixture.

This evidence detects drift but does not promise compatibility with alpha.2 or
stable 2.0. Scene structures and fixed-point representation remain private.

## Alpha.3 Contract

Alpha.3 records deterministic evidence for the complete existing family,
background, and frame catalog. Catalog IDs retain their 1.x ordering, while
pixels intentionally use the new 2.0 renderer. The complete aggregate pixel
fingerprint detects cross-family drift; it remains a prerelease KAT rather than
a stable compatibility promise.

## Alpha.4 Contract

Alpha.4 records deterministic evidence for typed layered composition. It
freezes prerelease accessory/palette/expression identifiers, fixed stack
capacity, canonical slot and z-band ordering, strict errors, automatic fallback
order, family anchor profiles, and complete decision reporting. Layer-free
alpha.3 KATs remain unchanged; layered pixels remain prerelease evidence rather
than a stable compatibility promise.

## Alpha.5 Contract

Alpha.5 records deterministic evidence for the prepared-request and
established-format boundaries. It adds owned redacted identities, resource
budgets, reusable RGBA storage, independent public key domains, and an isolated
format provider. The semantic WebP, PNG, JPEG, and GIF settings and alpha
behavior are named contracts; only WebP and PNG decode exactly to canonical
RGBA. Encoded byte identity across dependency builds is not promised unless an
application also binds an `EncoderBuildId` or hashes the final bytes.

These APIs, key domains, settings, and package boundaries remain prerelease
evidence and may change before stable 2.0 with explicit migration notes.

## Exact 1.x Output

Applications requiring exact 1.x output should pin `=1.3.0` or a later
maintenance patch from the 1.3 line. Main does not carry the old renderer. See
[MIGRATION_2.0.md](MIGRATION_2.0.md).
