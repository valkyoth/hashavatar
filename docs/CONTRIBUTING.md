# Contributing

Use the pinned toolchain from `rust-toolchain.toml` and run:

```bash
scripts/checks.sh
```

Keep changes focused, keep every Rust source file at or below 500 lines, and
add tests for public behavior, rendering, parsing, arithmetic, or error changes.
Do not add bundled avatar art, unreviewed binary assets, service dependencies,
or public scene internals.

Rendering changes must state whether they intentionally change trait vectors,
pixels, SVG, work estimates, or allocation bounds. Update KATs only after the
new output has been reviewed, and document the reason in the changelog and
release note.

Root `PENTEST.md` is temporary review input. Resolve each finding, test concrete
bugs, document accepted limitations, delete the scratch report, and retain a
sanitized milestone digest under
[`security/pentest`](../security/pentest/README.md).

The 1.3 maintenance branch accepts serious security and correctness fixes.
Feature work and visual changes target the current 2.0 prerelease on `main`.
