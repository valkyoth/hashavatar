# Security Controls

Hashavatar turns bounded identity bytes into public visual artifacts. Its main
security concerns are panic avoidance, resource exhaustion, deterministic
output, namespace separation, safe SVG, sensitive temporary lifetime, and
supply-chain scope.

## Enforced Controls

- Both workspace libraries are `no_std` and forbid first-party `unsafe`.
- Public dimensions are validated as `64..=2048` before preparation.
- Identity input is capped at 1024 bytes; tenant and style-version components
  are capped at 128 bytes each.
- Identity components are length-prefixed and SHA-512 domain separated.
- Trait samples use separate domain-separated labels, so adding a trait cannot
  shift mutable RNG state or correlate fields by consuming adjacent bytes.
- Request preparation does not retain raw identity or namespace input.
- Temporary hash preimages use `sanitization::SecretVec`; derived digest
  ownership uses `sanitization::Secret`.
- SHA-512 runs through `sanitization-crypto-interop`, including its reviewed
  upstream hasher cleanup path.
- Geometry uses checked signed Q16.16 values. No floating point is used.
- The canonical scene is private and capped at 64 commands, eight paths, 48
  lowered points per path, and eight levels each of clip and opacity stacks.
  Validation rejects malformed commands, stack imbalance, invalid paint or
  geometry, degenerate shapes, bad path references, and arithmetic failure.
- Both raster and SVG executors revalidate the scene before execution.
- Raster allocation, stride, and visible-row lengths use checked arithmetic;
  owned allocations use fallible reservation.
- Pixel writes use checked offsets and bounds-checked slices.
- Raster loop bounds are clipped to validated output dimensions.
- `SceneReport` exposes exact RGBA bytes and conservative pixel-test work.
- Pixel fingerprints stream visible rows through fixed-size, clear-on-drop
  SHA-512 state without allocating another image-sized buffer.
- Caller surfaces are prevalidated and preserve all row-padding bytes.
- Source-over compositing, opacity, gradients, paths, and curve lowering are
  integer-only and covered by explicit contracts.
- SVG caller prefixes are restricted to a safe ASCII identifier grammar.
  Accessibility strings are XML escaped before entering markup.
- Owned SVG writes are capped at a 64 KiB pre-reserved document bound; writer
  failures return a typed error with documented partial-prefix behavior.
- SVG is tested with an XML parser across the complete catalog, including
  non-square clip-path semantics.
- Production code is checked for panic-like sites; debug and release pixel
  fingerprints must match.
- Rust source files are capped at 500 lines to keep review boundaries small.

## Application Responsibilities

A valid 2048 by 2048 raster returns 16 MiB. Hashavatar bounds one request but
cannot control process-wide concurrency. Servers must use a semaphore or an
equivalent admission controller, set request and rate limits, and account for
the returned buffer plus application/network overhead.

SHA-512 is deterministic hashing, not password hashing. A visible avatar can
support offline dictionary testing of low-entropy identifiers. Applications
that treat identifiers as sensitive should first derive a keyed,
domain-separated pseudonym with a separately managed secret and pass only that
pseudonym to Hashavatar. Hashavatar does not own application key storage or
rotation policy.

Returned `CanonicalRgbaImage` bytes and SVG strings are caller-owned public
artifacts. The crate does not wipe them. Callers with an unusual requirement to
remove rendered output from memory must use a reviewed caller-side cleanup
container after use.

## Accepted Limitations

- Input length is visible through bounded allocation size. Callers needing
  length hiding must normalize or pad input before the crate boundary.
- Rendering time varies with admitted geometry and dimensions. Identity and
  namespace values must not be treated as timing-protected secrets.
- Secret cleanup is best effort under Rust drop semantics. It cannot guarantee
  removal from registers, compiler spills, allocator metadata, swap, crash
  dumps, hibernation, or hardware state, and it does not run after process
  abort.
- Digest output is copied between dependency and secret-container boundaries.
  Those unavoidable by-value copies are not a claim of perfect physical
  erasure.
- Allocation failure is returned where fallible Rust allocation APIs permit.
  A global allocator may still abort the process under its configured policy.
- Alpha implementation-stop commits are development evidence, not a stable API
  or pixel compatibility promise.

Do not report an accepted limitation as a vulnerability unless code contradicts
this document, a cleanup API fails its stated contract, or a new concrete
exploit path exists.

## Assurance

The gate includes strict Clippy, debug/release KATs, parser-backed SVG tests,
MSRV checks, cross-target core compilation, fuzz-harness compilation, eight
bounded Kani proofs, RustSec, cargo-deny, package inspection, unsafe/panic
policy checks, and reproducible archive comparison. Independent pentest
digests are retained under [`security/pentest`](../security/pentest/README.md).

These controls improve assurance but are not a claim of formal verification,
constant-time rendering, or fitness for a specific regulatory classification.
