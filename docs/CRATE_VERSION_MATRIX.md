# Crate Version Matrix

This matrix records every publishable package in the Hashavatar workspace and
the release decision for the current source state. The machine-readable source
is [`release-crates.toml`](../release-crates.toml).

## Current Baseline

| Package | Current version | crates.io state | Role |
| --- | --- | --- | --- |
| `hashavatar` | `1.3.0` | Published; unchanged on the 2.0 preparation commit | Current single-crate facade, renderer, and encoders |

The maintained 1.x line lives on `release/1.3`. Main is preparing the 2.0
workspace and will change this matrix when the first alpha version is created.

## 2.0 Prerelease Policy

Every workspace package must appear in `release-crates.toml`, including an
explicit version, change classification, publication decision, and reason.
Dependency crates must precede dependants in `publish_order`.

Alpha, beta, and release-candidate tags are source-only. They are tested by
`hashavatar-website` through local path checkouts or tagged source references,
but no prerelease package is uploaded to crates.io. The release script rejects
any prerelease entry with `publish = true`.

## Stable Publication

For the final stable 2.0 release, changed publishable crates use
`publish = true`; unchanged crates remain explicit with `change = "unchanged"`
and `publish = false`. The release script validates the complete workspace,
packages every crate, and publishes selected crates in dependency order only
after the signed tag and permanent pentest report exist.
