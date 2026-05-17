# Panic Policy

Library code should return `Result` for caller-controlled invalid input. Panics are allowed only for internal invariants that cannot be triggered after public validation, and each allowed site is checked by `scripts/validate-panic-policy.sh`.

## Reviewed Panic-Like Sites

- Public rendering APIs return `AvatarSpecError` for unsupported dimensions instead of panicking.
- Rectangle helpers use saturating or clamping arithmetic so future internal placement changes cannot overflow or panic in these helpers.
- Tests may use `expect`, `panic`, and related assertions freely.

New production `unwrap`, `expect`, `panic`, or `unreachable` sites should be added only with a concrete invariant and a matching update to the validation script.
