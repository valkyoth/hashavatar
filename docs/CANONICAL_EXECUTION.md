# Canonical Execution

Canonical output is the safe-Rust CPU RGBA8 executor. For one source revision,
contract labels, request, namespace, and style seed, visible bytes must be
independent of thread scheduling, locale, CPU feature detection, optimization
level, and supported architecture.

The executor is currently single-threaded and uses no SIMD dispatch, ambient
entropy, clock, filesystem, network, mutable global state, or floating point.
Scene validation occurs before execution. Loops are bounded by validated image
dimensions, scene capacities, fixed curve steps, and stack limits. Arithmetic
that can affect indexing or geometry is checked or widened.

Debug and release KATs, cross-target compilation, Kani arithmetic proofs, fuzz
harnesses, and reproducible package archives provide evidence for this
contract. They are not a formal proof of every renderer path. SVG and future
GPU output are noncanonical semantic backends unless explicitly documented
otherwise.
