# Stability Policy

## Published 1.x

`v1.3.0` is the final feature release of the 1.x renderer. The `release/1.3`
branch receives serious security and correctness fixes. Its public API and
frozen pixels remain governed by the documentation and corpus on that branch.

## 2.0 Prereleases

Alpha, beta, and RC APIs and pixels may change. They are tagged on GitHub for
exact-source testing but are not uploaded to crates.io. Every prerelease change
must update its release note, current status, crate matrix, tests, and pentest
digest before tagging.

Alpha.1 freezes evidence for one Cat vertical slice, not the final 2.0 public
surface. The following are intentional current contracts:

- checked request bounds and typed failures;
- stateless label-separated trait derivation;
- private fixed-point scene representation;
- one scene used by canonical CPU RGBA8 and SVG output;
- deterministic output within the same source revision and contract labels;
- no exposure of raw identity digests or scene internals.

Changing alpha.1 domain labels, rounding, command order, containment tests,
colors, or SVG serialization changes output and requires explicit KAT updates
and release-note disclosure.

## Stable 2.0

The stable release will freeze the package/API contract, public trait and
catalog identifiers, numeric and compositing rules, canonical CPU output,
schema conversions, cache-key domains, and supported target matrix described
in [PLAN_TOWARDS_2.0.md](PLAN_TOWARDS_2.0.md). Optional format and GPU package
output has separate compatibility semantics.
