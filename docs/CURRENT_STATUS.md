# Current Status

## Published Line

The latest crates.io release is `hashavatar 1.3.0`. The signed `v1.3.0` tag and
`release/1.3` branch preserve the final 1.x renderer. That branch accepts
serious security and correctness fixes; new features and intentional visual
changes belong to 2.0.

## Development Line

`main` implements `2.0.0-alpha.1`. This source-only prerelease introduces the
resolver-3 `hashavatar`/`hashavatar-core` workspace and one canonical Cat
vertical slice:

- stateless, label-separated SHA-512 trait derivation;
- private checked Q16.16 geometry;
- a bounded validated private scene;
- one safe-Rust CPU straight-alpha RGBA8 executor;
- deterministic SVG from the same scene;
- debug/release pixel KATs and parser-backed SVG checks;
- a focused fuzz target and five Kani harnesses;
- source-size enforcement at 500 lines per Rust file;
- portable core CI for WASM, AArch64 Linux, and 32-bit x86 Linux.

The implementation is ready for exact-commit external pentesting. It must not
be tagged until the findings are resolved, the permanent pentest digest is
added, GitHub is green, and `hashavatar-website` passes against this source.

## Prerelease Policy

Alpha, beta, and release-candidate tags are pushed to GitHub but not crates.io.
`hashavatar-website` follows each reviewed source tag through a local or Git
dependency. Every tag receives local release checks, GitHub CI and CodeQL,
downstream integration testing, and a permanent digest under
[`security/pentest`](../security/pentest/README.md).

## Toolchain

| Item | Version or policy |
| --- | --- |
| MSRV | Rust `1.90.0` |
| Development toolchain | Rust `1.97.1` |
| Workspace resolver | Cargo resolver `3` |
| CodeQL | GitHub default setup |
| Unsafe Rust | Forbidden in first-party library code |
| Alpha.1 identity mode | Domain-separated SHA-512 |
| Alpha.1 outputs | Canonical RGBA8 and SVG |

Version details live in [release notes](../release-notes/) and the
[changelog](../CHANGELOG.md).
