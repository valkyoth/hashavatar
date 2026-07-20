# Schema Companion Roadmap

Status: required for `v2.0.0-alpha.7` and Hashavatar 2.0

A versioned JSON request document is a long-term wire-compatibility commitment.
It belongs in an independently versioned `hashavatar-schema` crate, not in core.

## Admission Preconditions

- The Rust `AvatarRequest` baseline from `alpha.5` is implemented and covered by
  migration fixtures before the document contract is frozen.
- `hashavatar-website` and a minimal independent service fixture exercise the
  common wire model.
- The shared fields can be separated cleanly from application policy.

## Scope

The crate may provide a bounded `AvatarRequestDocumentV1`, strict conversion to
core `AvatarRequest`, and optional Serde/JSON Schema support. It must reject
unknown and duplicate fields where the parser supports that distinction and
must not add serialization dependencies to core.

It must not own HTTP routes, query parsing, OpenAPI endpoints/status codes,
authentication, persistence, locale, telemetry, rate limits, redirects, or
object-storage policy. OpenAPI remains application-owned and may embed the
request schema.

## Finish Line

Publish only after schema snapshots, malformed/duplicate/unknown-field tests,
bounded allocation review, compatibility policy, MSRV and dependency isolation,
and packaged trials in both consumers. Failure to meet this finish line blocks
Hashavatar 2.0 without moving HTTP policy into the schema crate.
