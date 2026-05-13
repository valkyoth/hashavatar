# GitHub Metadata

## Crate Repository

Repository:
- `hashavatar`

About:
- `Deterministic procedural avatars in Rust with SHA-512 identities and WebP, PNG, JPEG, GIF, and SVG export.`

Suggested topics:
- `rust`
- `avatar`
- `identicon`
- `procedural-generation`
- `graphics`
- `webp`
- `svg`
- `image-processing`
- `sha512`
- `open-source`

Short description variant:
- `A Rust crate for deterministic, code-generated avatars without bundled art assets.`

Repository security settings:
- Enable Dependabot alerts
- Enable Dependabot version updates from `.github/dependabot.yml`
- Enable CodeQL default setup for code scanning alerts
- Do not add an advanced CodeQL workflow while default setup is active
- Enable private vulnerability reporting or GitHub security advisories

Release title example:
- `hashavatar v0.5.0`

Release notes template:

```text
License migration and demo server hardening.

Highlights:
- Starting with 0.5.0, project licensing is MIT OR Apache-2.0
- LICENSE-MIT and LICENSE-APACHE are included
- Previous EUPL license files were removed
- Demo-server avatar encoding now runs on Tokio's blocking task pool
- Demo responses now include defense-in-depth HTTP security headers
- GitHub Actions are pinned to immutable commit SHAs
- Local and GitHub checks now run through scripts/checks.sh

Compatibility:
- No avatar rendering behavior changes are intended
- Published 0.4.x and older versions retain their original release licensing

Licence:
- MIT OR Apache-2.0
```

## API Repository

Repository:
- `hashavatar-api`

About:
- `Public HTTP avatar API and demo site for deterministic procedural avatars, designed for aggressive CDN caching.`

Suggested topics:
- `rust`
- `axum`
- `api`
- `avatar`
- `cdn`
- `cloudflare`
- `webp`
- `svg`
- `image-api`
- `procedural-generation`

Short description variant:
- `A cache-friendly HTTP avatar service built on top of the hashavatar Rust crate.`

Release title example:
- `hashavatar-api v0.1.0`

Release notes template:

```text
Initial public release of hashavatar-api.

Highlights:
- Query and path-based avatar endpoints
- WebP, PNG, and SVG output
- Cache headers ready for Cloudflare edge caching
- Dockerfile included
- Landing page and health endpoint

Licence:
- MIT OR Apache-2.0
```
