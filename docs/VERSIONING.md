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

## Exact 1.x Output

Applications requiring exact 1.x output should pin `=1.3.0` or a later
maintenance patch from the 1.3 line. Main does not carry the old renderer. See
[MIGRATION_2.0.md](MIGRATION_2.0.md).
