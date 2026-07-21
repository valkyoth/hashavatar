# GitHub Metadata

This page keeps repository-facing descriptions separate from the technical
README. It is operational metadata, not part of the crate API.

## Hashavatar Repository

Repository: `hashavatar`

About:

> Canonical deterministic procedural avatars in Rust with stateless traits,
> bounded fixed-point scenes, RGBA8, and SVG output.

Suggested topics:

- `rust`
- `avatar`
- `identicon`
- `procedural-generation`
- `graphics`
- `svg`
- `image-processing`
- `no-std`
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

Hashavatar 2.0 alpha, beta, and release-candidate milestones use clearly named
implementation-stop commits instead of Git tags or GitHub releases. Only
approved stable releases require signed tags and release bodies.

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
