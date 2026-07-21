# Contributing

Thanks for helping improve `hashavatar`.

## Development

Use the pinned development toolchain from `rust-toolchain.toml`.

```bash
scripts/checks.sh
```

For a faster local loop while developing, run the individual Cargo commands the
script prints before the failing step.

## Pull Requests

- Keep changes focused and explain the user-visible behavior.
- Add or update tests when rendering behavior, encoders, parsing, or public API types change.
- Do not add bundled avatar art, stock assets, or generated binary assets without prior discussion.
- Preserve deterministic output unless the change is explicitly a visual-version change.

## Pentest Findings

Root `PENTEST.md` is temporary review input. Resolve each actionable finding,
add focused regression coverage, record accepted limitations in the relevant
security document, and delete the scratch report. Release candidates retain a
sanitized per-tag summary under `security/pentest/` according to
[`security/pentest/README.md`](../security/pentest/README.md).

## Visual Stability

`hashavatar` is deterministic. Changes to shape generation, colors, hashing, randomization, or encoder behavior can affect downstream users. When a change intentionally affects output, document it in the changelog.
