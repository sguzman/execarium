# Versioning and release policy

## Rule: crate major == wire schema integer

**Execarium 1.0.0** is the first stable release. The Rust package
version is now `1.1.0`, which adds backwards-compatible construction and
versioned validator APIs. Conforming YAML documents still declare
`schema: 1`.

This is deliberate, not a typo:

* **Initial Git tag:** `v1.0.0` / Rust crate `1.0.0`.
* **Current Git tag / implementation:** `v1.1.0` / Rust crate `1.1.0`.
* **YAML wire schema:** integer `1`, not `1.0.0` or `"1"`.
* **Compatibility mapping:** crate major 1 implements schema 1.
  A future crate major 2 implements schema 2; any optional support for schema
  1 must be explicit, never inferred or silently substituted.

Each released library major version provides a version-namespaced validator,
starting with `v1::Validator` for schema 1. The `parse_for_schema` and
`validate_for_schema` dispatchers **reject unknown versions**.
Older schemas, if explicitly supported in a future major crate, must keep
their own independent validators and conformance suites.

The same *major version* is shared; the patch and minor components are
implementation release metadata. This avoids introducing a second, redundant
three-part version inside YAML.

## Changes after v1.0.0

| Change | YAML schema | Crate SemVer |
| --- | --- | --- |
| Correct documentation wording, tests, packaging | stays `1` | patch release (`1.0.1`) |
| Fix parser bug without changing the normative v1 contract | stays `1` | patch release, with regression test |
| Add backwards-compatible library API or CLI convenience | stays `1` | minor release (`1.1.0`, `1.2.0`, etc.) |
| Add any new YAML key (even optional), remove/change a key, change field semantics or validation requirements | **new schema integer `2`** | **crate `2.0.0`** |
| Add command lists with independent step semantics | new schema integer `2` | crate `2.0.0` |

**Strict-parser consequence:** Adding even an optional YAML key would cause
earlier conforming parsers to reject new documents. Such changes are therefore
not minor changes to the wire contract.

## Support and validation

* Each document MUST specify exactly one supported schema integer.
* Unknown schemas MUST fail closed. Do not interpret `schema: 2` as v1.
* A consumer MUST choose allowed schema versions explicitly; using crate
  `1.x` is not permission to execute any untrusted document.
* Crate SemVer concerns the *Rust API*. The YAML spec remains language-neutral
  and is the authority for cross-language consumers.
* Implementation releases must run the conformance suite and pass CI before
  receiving a release tag.
* Tags are immutable release pointers: never retag an existing version.
* Git dependencies SHOULD pin an immutable tag or reviewed commit; deployments
  using Cargo.lock resolve and lock a specific Git commit.
* Publishing to crates.io is a separate decision and is **not** implied by a
  GitHub release tag.

## v1.0.0 baseline

Schema 1 has six **required** fields, exactly once each:
`schema`, `shell`, `cwd`, `privilege`, `timeout_seconds`, `command`.
No `commands` list, `executor`, variable interpolation, implicit defaults,
or automatic clipboard trigger is part of v1.

For actual field semantics consult [SPEC-v1.md](SPEC-v1.md); for an AI-facing
production checklist consult [CHATGPT.md](CHATGPT.md).
