# Versioning and release policy

## Two independent version tracks

Execarium has **two contracts** with different compatibility requirements.

1. **YAML wire schema:** a required, unsigned integer inside every document,
   currently schemas `1`, `2`, and `3`. Schema 1 is the immutable six-field wire
   contract defined in [SPEC-v1.md](SPEC-v1.md); schema 2 defines ordered
   `commands:` sequences in [SPEC-v2.md](SPEC-v2.md); schema 3 defines
   execution contexts in [SPEC-v3.md](SPEC-v3.md). Any further wire changes
   require a **new schema integer**, not an in-place reinterpretation.
2. **Rust reference crate:** conventional Semantic Versioning (SemVer),
   currently **1.3.0**. A major increment indicates a breaking **Rust API**
   change; minor releases add backward-compatible Rust features; patches
   correct behavior, documentation, packaging, or tests without breaking the
   Rust API.

**There is no equality requirement between a crate's major version and any
YAML schema number.** They happened to begin together in 1.0.0, but this
was an initial convenience, not an architectural constraint.

For example:

| Example | Possible Rust crate | Supported YAML schemas |
| --- | --- | --- |
| Stable schema-1 MVP | `1.1.2` | `[1]` |
| Future crate with redesigned Rust API only | `2.0.0` | `[1]` |
| Additive schema-2 implementation | `1.2.0` | `[1, 2]` |
| Additive schema-3 execution contexts | `1.3.0` | `[1, 2, 3]` |

The 1.2.0 line adds `v2::Validator`; the 1.3.0 line adds `v3::Validator` and execution-context declarations. Schema 1 and 2 contracts remain unchanged.
The hypothetical 2.0.0 line illustrates crate/schema version independence.

## Schema compatibility is explicit

* Every document MUST declare an exact schema integer. No implicit default.
* The currently implemented schema-1 validator MUST reject unknown, missing,
  duplicated, or mismatched fields and unsupported YAML syntax.
* A consumer MUST decide which document schema versions it accepts.
* Use `parse_document`/`parse_document_for_schema` for multi-schema
  selection, or the explicit `v1::Validator` / `v2::Validator` / `v3::Validator`.
  Legacy `parse`, `parse_for_schema`, and `validate_for_schema` continue
  returning the original v1 Job API for compatibility. Never infer a wire
  schema from the crate version.
* This crate implements three versioned parsers concurrently and
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

* Git release tags (`v1.0.0` through `v1.3.0`, when published) identify
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

See [SPEC-v1.md](SPEC-v1.md), [SPEC-v2.md](SPEC-v2.md), and [SPEC-v3.md](SPEC-v3.md) for wire contracts,
[RUST-API.md](RUST-API.md) for the reference crate interface and
[CHATGPT.md](CHATGPT.md) for AI-facing authoring instructions.
