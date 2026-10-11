# Changelog

All notable releases are documented here. Rust implementation releases follow
Semantic Versioning **independently** of the YAML schema integer; see
[VERSIONING.md](docs/VERSIONING.md).

## [1.2.0] — 2026-10-11

**Backwards-compatible Rust API addition: new independent YAML schema 2.**

* Add strict `v2::Validator` with a `commands:` sequence of 1–256 shell steps,
  stop-on-first-failure consumer semantics, and one overall execution timeout.
* Add explicit `parse_document` multi-version dispatch; preserve the existing
  `v1::Validator`, `parse`, `construct`, and `parse_for_schema` APIs.
* Extend validation-only CLI to recognize schema 1 and schema 2 documents.
* Document normative v2 semantics and keep frozen schema 1 unchanged.
* No execution, clipboard monitoring or privilege authorization in Execarium.

## [1.1.2] — 2026-10-11

**Independent versioning policy; YAML schema 1 unchanged.**

* Removed the rule requiring Rust crate major versions to match YAML schema
  integers. A future Rust 2.x release may still implement schema 1; a future
  compatible Rust 1.x release could add a separately validated schema 2.
* Updated version documentation, architecture and decision records.
* Replaced the version-equality conformance test with independent checks for
  Rust SemVer metadata and explicit schema support.
* CI checks the version declared by Cargo.toml dynamically, rather than
  hardcoding the current crate patch release.
* No changes to v1 YAML fields, parser/builder behavior or execution policy.

## [1.1.1] — 2026-10-11

**Execarium MVP acceptance release, YAML schema 1 unchanged.**

* 20 standalone version-1 YAML conformance documents: six valid, fourteen invalid.
* Added independent real CLI integration tests covering stdin, files,
  invalid UTF-8, bounded input, argument handling, and no-command-execution.
* Added deterministic stress tests: 2,000 construction/YAML/parse round trips,
  3,000 malformed-input robustness checks, and 54 shell/privilege/timeout
  combinations.
* Expanded boundary, lexical-path, scalar-type, unknown/duplicate-field,
  YAML tag/alias and exact schema-selection checks.
* CI enforces the portable corpus through both the Rust API and CLI.
* Acceptance policy and results documented in
  [MVP-ACCEPTANCE.md](docs/MVP-ACCEPTANCE.md).
* No changes to the normative wire schema, builder, parser, or executor policy.

## [1.1.0] — 2026-10-11

**Compatible Rust library feature release; YAML schema remains 1.**

* Public `execarium::v1::Validator` with explicit version-1
  `parse`, `validate`, `construct`, and `render` methods.
* `parse_for_schema`, `validate_for_schema` and
  `SUPPORTED_SCHEMAS` for deliberate version selection, no fallback.
* Typed construction shares the exact field constraints of the
  existing YAML parser. `Job::to_yaml()` renders a complete
  schema-1 document.
* Rust example and round-trip tests for scalar quoting, multiline
  commands, input rejection, and backwards compatibility.
* Dedicated [Rust API guide](docs/RUST-API.md) for other projects.
* Original crate 1.0.0 entry points remain available.

**No schema 1 field changes.** No process execution or clipboard integration.

## [1.0.0] — 2026-10-11

**First stable specification and Rust reference implementation.**

### Stable YAML schema 1

Exactly six required fields:

```yaml
schema: 1
shell: fish
cwd: /home/user/Projects/example
privilege: none
timeout_seconds: 600
command: cargo test
```

* Strict integers, enumerated shells (`fish`, `bash`, `sh`) and privilege
  requests (`none`, `sudo`, `doas`).
* Absolute working directory, bounded UTF-8 command, required bounded
  execution timeout (1–604800 seconds).
* YAML duplicate/unknown keys, indirection, unsupported versions and invalid
  scalar types are rejected.
* Validates data without executing commands, collecting clipboard contents,
  prompting for credentials or granting execution authority.

### Rust reference implementation

* Library: `execarium::parse`, `execarium::validate`, typed `Job` and
  public enum/limit/version constants.
* CLI: `execarium validate FILE`, `execarium validate -` (stdin), and
  `execarium --version`.
* Conformance fixtures, documentation tests and GitHub Actions CI.

### Historical note

Earlier commits contained **unreleased draft** formats, including a
five-field prototype before `timeout_seconds` was introduced. They were
never stable schemas or releases. **v1.0.0 defines the six-field schema 1
baseline**. The five-field prototype is invalid under v1.0.0.

### Not implemented in this release

No command execution, automatic clipboard integration, queue manager, user
authentication, Docker, microVMs or remote execution. Those concerns belong
to consumers or later specifications.

[1.0.0]: https://github.com/sguzman/execarium/tree/v1.0.0

[1.1.0]: https://github.com/sguzman/execarium/tree/v1.1.0

[1.1.1]: https://github.com/sguzman/execarium/tree/v1.1.1

[1.1.2]: https://github.com/sguzman/execarium/tree/v1.1.2
