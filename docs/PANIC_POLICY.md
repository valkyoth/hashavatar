# Panic Policy

Public Hashavatar operations accept untrusted dimensions, namespace bytes, and
identity bytes through fallible constructors and return `AvatarError`.
Production library paths must not use `panic!`, `assert!`, `unwrap`, `expect`,
or `unreachable!` for caller-reachable validation.

Tests and Kani harnesses may assert invariants. OOM behavior controlled by the
global allocator and process abort behavior remain outside Rust unwind cleanup
guarantees.

`scripts/validate-panic-policy.sh` scans production Rust modules in every
workspace crate. Clippy additionally forbids panic, unwrap, and expect sites.
Any future exception requires narrow source annotation, security review,
documentation here, and a regression test proving it is not reachable from
untrusted public input.
