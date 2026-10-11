# Changelog

All notable releases are documented here. Rust implementation releases follow
Semantic Versioning. The **crate major version matches the YAML schema
integer**; see [VERSIONING.md](docs/VERSIONING.md).

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
