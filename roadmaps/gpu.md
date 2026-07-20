# GPU Companion Roadmap

Status: required for `v2.0.0-alpha.9`, `alpha.10`, and Hashavatar 2.0

GPU acceleration belongs in an independently versioned `hashavatar-gpu` crate.
It must consume only stable, validated core output or an intentionally narrow
opaque execution interface. It must never add GPU dependencies to
`hashavatar-core` or the default `hashavatar` feature graph.

## Admission Preconditions

- The canonical CPU scene capabilities and corpus from earlier alphas are
  stable enough to evaluate a backend without exposing mutable scene authoring.
- The selected GPU stack passes current license, MSRV, maintenance, unsafe/FFI,
  platform, package-size, and dependency-graph review.

## Initial Contract

- GPU output is visually conforming and explicitly noncanonical.
- Unsupported capabilities return typed errors; fallback to CPU is never silent.
- Device/queue ownership, cancellation, timeout, and scheduling stay with the
  caller.
- Buffer bounds, zero-fill/reuse, completion fences, device loss, and cleanup
  limitations are documented.
- CPU remains the source for canonical `PixelDigest` and cache identity unless
  a future proof establishes exact equality.

## Finish Line

The crate is publishable only after multi-device differential tests, bounded
resource and failure tests, default-graph isolation, packaged downstream trials,
and an independent review of the third-party unsafe/driver boundary. A canonical
claim requires zero pixel differences across the declared vendor/driver matrix;
otherwise the backend ships as explicitly noncanonical. Failure to meet this
finish line blocks Hashavatar 2.0 without weakening the canonical CPU contract.
