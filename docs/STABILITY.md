# Stability Policy

## Published 1.x

`v1.3.0` is the final feature release of the 1.x renderer. The `release/1.3`
branch receives serious security and correctness fixes. Its public API and
frozen pixels remain governed by the documentation and corpus on that branch.

## 2.0 Prereleases

Alpha, beta, and RC APIs and pixels may change. Each milestone ends at a named
implementation-stop commit used for exact-SHA testing; prerelease tags are not
created and crates are not uploaded to crates.io. Every milestone must update
its release note, current status, crate matrix, tests, and pentest digest before
work begins on the next milestone.

Alpha.2 freezes evidence for one complete canonical renderer. Alpha.3 adds
evidence for the existing 31-family, 13-background, five-frame catalog.
Alpha.4 adds typed layered composition. Alpha.5 adds the prepared-request,
resource, reusable-raster, key, established-format, and facade boundaries.
Neither milestone is the final 2.0 public surface. The
following are intentional current contracts:

- checked request bounds and typed failures;
- stateless label-separated trait derivation;
- private fixed-point scene representation;
- one scene used by canonical CPU RGBA8 and SVG output;
- frozen catalog IDs and explicit family capability declarations;
- bounded accessory capacity, typed slots, canonical ordering, and reported
  strict/automatic compatibility decisions;
- integer palette roles and per-family integer anchor profiles;
- deterministic output within the same source revision and contract labels;
- owned identities that retain no raw identifier or namespace components;
- explicit public identity, canonical asset, semantic encoded, and
  deployment-build key domains;
- lossless WebP by default with explicit PNG, JPEG, GIF, and all-format
  features isolated outside the portable core;
- typed writer and allocation failures with documented partial-output rules;
- no exposure of raw identity digests or scene internals.

Changing current domain labels, catalog order, family compilers, rounding,
command order, containment tests, colors, or SVG serialization changes output
and requires explicit KAT updates and release-note disclosure.

## Stable 2.0

The stable release will freeze the package/API contract, public trait and
catalog identifiers, numeric and compositing rules, canonical CPU output,
schema conversions, cache-key domains, and supported target matrix described
in [PLAN_TOWARDS_2.0.md](PLAN_TOWARDS_2.0.md). Optional format and GPU package
output has separate compatibility semantics.
