# Panic Policy

Library code should return `Result` for caller-controlled invalid input. Panics
are allowed only for internal invariants that cannot be triggered after public
validation, and each allowed site is checked by
`scripts/validate-panic-policy.sh`. The script scans every production Rust
module and excludes only the dedicated test and Kani proof modules.

## Reviewed Panic-Like Sites

- Public rendering APIs return `AvatarSpecError` for unsupported dimensions instead of panicking.
- Rectangle helpers use saturating or clamping arithmetic so future internal placement changes cannot overflow or panic in these helpers.
- `AvatarIdentity::byte` uses a debug-only assertion to catch internal digest
  offset mistakes in tests and debug builds. Release builds keep the
  non-panicking fallback for defense in depth.
- Identity, cache-key, and XXH3 preimage builders use debug-only assertions to
  detect internal size-accounting drift. Their buffers are already owned by
  `sanitization::SecretVec`, so cleanup does not depend on those assertions.
- Tests may use `expect`, `panic`, and related assertions freely.

New production `unwrap`, `expect`, `panic`, `debug_assert`, or `unreachable`
sites should be added only with a concrete invariant and a matching update to
the validation script.
