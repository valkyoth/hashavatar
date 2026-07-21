# Heapless Storage Roadmap

Status: required for `v2.0.0-alpha.6` and Hashavatar 2.0

Hashavatar 2.0 targets both `no_std + alloc` and a no-allocator caller-storage
profile. Heapless rendering must not force mutable scene authoring or internal
arena layouts into the stable public API.

## Admission Preconditions

- The private scene and budget model is stable enough to size storage safely.
- The design can remain safe Rust without an unaudited `MaybeUninit` boundary.
- Representative embedded and WASM32 targets define realistic memory and
  maximum-art constraints.

## Candidate Direction

- caller-provided preinitialized command/path/point/paint storage;
- typed `SceneCapacityExceeded` errors with required or conservative capacities;
- published per-family worst-case budgets;
- no silent allocation fallback;
- no public scene deserialization or arbitrary scene authoring requirement.

## Finish Line

Admit heapless support only after zero-allocation measurements, capacity and
adversarial tests, representative embedded builds, Kani bounds proofs, panic and
unsafe review, and an API design that does not freeze unnecessary renderer
internals. Failure to meet this finish line blocks Hashavatar 2.0.
