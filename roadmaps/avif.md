# AVIF Format Roadmap

Status: required for `v2.0.0-alpha.8` and Hashavatar 2.0

AVIF becomes an optional `hashavatar-formats` feature after the baseline format
boundary exists. It joins `all-formats` only after explicit admission and is
never enabled by default.

## Admission Review

- Recheck current encoders, transitive features, licenses, MSRV, assembly,
  threading, unsafe code, WASM support, maintenance, and advisories.
- Measure compile cost, package growth, peak memory, CPU time, output quality,
  cancellation behavior, and codec-owned scratch retention.
- Decode a representative corpus and compare pixels under the documented lossy
  tolerance; never imply encoded-byte stability across encoder upgrades.
- Keep the feature absent from core and default dependency graphs.

## Finish Line

AVIF is admitted only with typed capability/resource metadata, writer and error
tests, alpha handling tests, malformed-input/encoder failure coverage, package
evidence, security review, and accurate cleanup limitations. If no provider
qualifies, Hashavatar 2.0 waits; the project does not weaken policy to satisfy
the schedule.
