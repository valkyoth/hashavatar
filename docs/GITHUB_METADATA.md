# GitHub Metadata

This page keeps repository-facing descriptions separate from the technical
README. It is operational metadata, not part of the crate API.

## Hashavatar Repository

Repository: `hashavatar`

About:

> Secure deterministic procedural avatars in Rust with bounded inputs, typed
> cache keys, and WebP/SVG output.

Suggested topics:

- `rust`
- `avatar`
- `identicon`
- `procedural-generation`
- `graphics`
- `webp`
- `svg`
- `image-processing`
- `security`

Repository security settings:

- Enable Dependabot alerts and the updates configured in
  `.github/dependabot.yml`.
- Use GitHub CodeQL default setup. Do not add a competing advanced CodeQL
  workflow while default setup remains enabled.
- Enable private vulnerability reporting or GitHub security advisories.
- Protect `main` and `release/1.3` according to the repository's chosen review
  and status-check policy.

Release titles use `hashavatar vX.Y.Z`. Release bodies should summarize the
version's root `release-notes/RELEASE_NOTES_X.Y.Z.md`, link the permanent
`security/pentest/vX.Y.Z.md` report, identify compatibility changes, and state
whether crates were published.

Hashavatar 2.0 alpha, beta, and release-candidate tags are source-only. Their
release bodies must state that they were not published to crates.io.

## Website Repository

Repository: `hashavatar-website`

About:

> Public avatar generator and reference web integration for the hashavatar
> Rust crates.

Suggested topics:

- `rust`
- `avatar`
- `website`
- `axum`
- `webp`
- `svg`
- `procedural-generation`
- `reference-implementation`

The website demonstrates integration. It is not the reusable crate API and
must own its HTTP routing, headers, caching, concurrency, rate limits, and
abuse controls.
