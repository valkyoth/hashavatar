# Security Policy

## Supported Versions

Security fixes are expected for the latest published release series.

## Reporting a Vulnerability

Please report security issues privately through GitHub Security Advisories for:

`https://github.com/valkyoth/hashavatar/security/advisories/new`

If GitHub advisories are unavailable, open a minimal public issue that asks for a private contact path without disclosing exploit details.

## Scope

Relevant security issues include:

- panics or resource exhaustion from untrusted avatar parameters
- unsafe SVG or output encoding behavior
- vulnerable dependency paths
- license or provenance concerns that affect safe redistribution

Please include reproduction steps, affected versions, and any known mitigations.

## Local Security Checks

Run the crate security and release policy checks with:

```bash
scripts/checks.sh
```

The checks cover release metadata, package contents, dependency scope, unsafe-code policy, reviewed panic-like sites, fuzz harness compilation, dependency licenses, and RustSec advisories.
