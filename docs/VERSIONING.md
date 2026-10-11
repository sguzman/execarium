# Versioning and release policy

## Two independent version tracks

Execarium has **two contracts** with different compatibility requirements.

1. **YAML wire schema:** a required, unsigned integer inside every document,
   currently `schema: 1`. Version 1 is the immutable six-field wire contract
   defined in [SPEC-v1.md](SPEC-v1.md). New or changed YAML fields or meanings
   require a **new schema integer**, for example `schema: 2`.
2. **Rust reference crate:** conventional Semantic Versioning (SemVer),
   currently **1.1.2**. A major increment indicates a breaking **Rust API**
   change; minor releases add backward-compatible Rust features; patches
   correct behavior, documentation, packaging, or tests without breaking the
   Rust API.

**There is no equality requirement between a crate's major version and any
YAML schema number.** They happened to begin together in 1.0.0, but this
was an initial convenience, not an architectural constraint.

For example:

| Example | Possible Rust crate | Supported YAML schemas |
| --- | --- | --- |
| Current MVP | `1.1.2` | `[1]` |
| Future crate with redesigned Rust API only | `2.0.0` | `[1]` |
| Future backwards-compatible multi-schema support | `1.2.0` | `[1, 2]` |

These future examples illustrate the *policy*, not promised releases or
implemented validators. **Currently only `v1::Validator` exists.**

## Schema compatibility is explicit

* Every document MUST declare an exact schema integer. No implicit default.
* The currently implemented schema-1 validator MUST reject unknown, missing,
  duplicated, or mismatched fields and unsupported YAML syntax.
* A consumer MUST decide which document schema versions it accepts.
* Query the Rust library's `SUPPORTED_SCHEMAS` and select
  `parse_for_schema(version, input)`, `validate_for_schema(version, input)`,
  or the explicit `v1::Validator`. Never infer a schema from the crate
  version or silently select a different validator.
* A future crate MAY implement several versioned parsers concurrently, but
  MUST preserve earlier format semantics, rejection behavior and conformance
  suites. Breaking changes to a *Rust* public API are versioned by Rust
  SemVer, regardless of supported schema integers.

The library `SCHEMA_VERSION` constant currently denotes schema 1 and is
**not** a general algorithm for deriving supported versions from a crate
number. The `SUPPORTED_SCHEMAS` list is the compatibility declaration.

## Change classification

| Change | YAML schema | Rust SemVer |
| --- | --- | --- |
| Documentation, test, packaging or CI refinement | unchanged | patch, if releasing |
| Fix implementation bug to conform to an unchanged normative schema | unchanged | patch (with regression test) |
| Add compatible Rust builder/parser API | unchanged | minor |
| Break or redesign the public Rust API | unchanged if YAML unchanged | **new Rust major**, irrespective of YAML |
| Add a new wire key, remove one, or change validation/meaning | **new schema integer** | depends on Rust API compatibility, potentially minor |
| Add a new versioned validator next to existing validators | old schema stays; new integer also supported | minor if additive to the Rust API |
| Add shell-independent command lists | requires new schema integer (e.g. 2) | independent crate release |

**Strict-parser consequence:** Even an *optional* new YAML field requires
a new schema version, because older strict parsers reject unknown keys.
Schema 1 will not change meaning in place.

## Release practice

* Git release tags (`v1.0.0`, `v1.1.0`, `v1.1.1`, `v1.1.2`) identify
  **Rust crate releases**, not YAML schema revisions.
* A schema's historical first release is recorded in its normative
  specification and changelog.
* Crate releases require passing formatting, tests, conformance and CI.
  The tagging workflow must never move an existing release tag.
* A Git dependency SHOULD pin an immutable tag or reviewed commit. Cargo.lock
  pins the resolved Git revision for consuming applications.
* The Rust crate is not published to crates.io merely because it has a Git
  tag.
* Schema version changes and Rust crate version changes are independently
  reviewed and documented, even if they occur in the same commit.

## Stable schema-1 baseline

YAML schema 1 contains exactly these **six required fields**:
`schema`, `shell`, `cwd`, `privilege`, `timeout_seconds`, `command`.
It does not include `commands`, `executor`, variable interpolation,
implicit defaults, or an automatic clipboard trigger.

See [SPEC-v1.md](SPEC-v1.md) for the normative wire contract,
[RUST-API.md](RUST-API.md) for the reference crate interface and
[CHATGPT.md](CHATGPT.md) for AI-facing authoring instructions.
