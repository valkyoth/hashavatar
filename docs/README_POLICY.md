# README Policy

## Facade README

The repository-root `README.md` is the canonical README for both GitHub and the
published `hashavatar` facade crate. The facade manifest must reference that
same file rather than maintain a copied crate-local document.

For the root facade package:

```toml
readme = "README.md"
```

If the facade later moves to `crates/hashavatar`, its manifest should use:

```toml
readme = "../../README.md"
```

Package validation must confirm that the archive contains the canonical README
bytes. The root README is a technical usage guide, not a release dashboard.
Changing release state, tool versions, evidence, and roadmap progress belongs
in [CURRENT_STATUS.md](CURRENT_STATUS.md), release notes, or release policy.

## Companion Crates

Every published companion crate has its own concise README because crates.io
and docs.rs present packages independently. Use the same visual identity and
documentation quality as the facade, but explain that crate's exact boundary.

Each companion README should contain:

1. The shared Hashavatar image and links to the facade, its own docs.rs page,
   security policy, roadmap, and relevant technical policy.
2. The exact package name and one-sentence purpose.
3. Guidance that most users should depend on the `hashavatar` facade.
4. Installation and one minimal compile-checked example.
5. Features and default dependency behavior.
6. Security, resource, determinism, and portability boundaries specific to the
   package.
7. A direct link to the workspace security policy and current status.

Do not copy the complete facade README into support crates. Avoid release
chronology, aspirational capability claims, or large status matrices in crate
READMEs. Keep details in `docs/` and link to them with absolute GitHub URLs so
links work identically on GitHub, crates.io, and docs.rs.

## Planned Package Focus

| Package | README focus |
| --- | --- |
| `hashavatar` | Recommended request, preparation, render, SVG, encoding, and key workflow |
| `hashavatar-core` | Canonical scene, CPU/SVG contracts, portability, and caller surfaces |
| `hashavatar-formats` | Encoder features, writer APIs, codec allocation, and format contracts |
| `hashavatar-heapless` | Caller storage, exact capacity failures, and no-allocator profiles |
| `hashavatar-schema` | Bounded request documents and transport-neutral conversion |
| `hashavatar-gpu` | Optional noncanonical backend capability, failure, and fallback contracts |
