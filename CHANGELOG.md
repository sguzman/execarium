# Changelog

All notable releases are documented here. Rust implementation releases follow
Semantic Versioning. The **crate major version matches the YAML schema
integer**; see [VERSIONING.md](docs/VERSIONING.md).

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
