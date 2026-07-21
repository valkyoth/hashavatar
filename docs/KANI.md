# Kani Proofs

Kani provides bounded model checking for small arithmetic and admission
contracts in `hashavatar-core`. These proofs complement tests and fuzzing; they
are not whole-crate formal verification and do not prove SHA-512 internals.

- Source: `crates/hashavatar-core/src/kani_proofs.rs`
- Current admitted harness count: `8`.
- Pinned tool: `cargo-kani 0.67.0`
- Toolchain: Rust `1.90.0-x86_64-unknown-linux-gnu`

The harnesses cover exact dimension admission, Q16.16 unit conversion,
bounded fixed-point interpolation, pixel-center rounding, exact RGBA resource
accounting, source-over canonicalization, opacity multiplication bounds, and
catalog byte-selection bounds.

Run opportunistically:

```bash
scripts/check_kani.sh
```

Run as required release evidence:

```bash
scripts/check_kani.sh --required
```

The explicit harness list keeps verification bounded and reviewable. Catalog art
quality, SVG parser behavior, allocator failures, crypto implementation
internals, and complete raster loops remain covered by other evidence rather
than claimed as Kani proofs.
