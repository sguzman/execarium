# Execarium

**A strict, versioned description of local shell jobs. Not a command runner.**

Execarium defines and validates a small YAML document describing **what shell command
a consumer is being asked to run**, in which working directory, with what
requested privilege level and execution timeout. It is an independent Rust library and a validation-only CLI,
so Jobflick and future projects can share one contract without importing a scheduler.

> **Status:** stable **v1.0.0** YAML specification; Rust reference library
> **v1.1.0** adds versioned validation, typed construction and YAML rendering.
> No execution, clipboard monitoring, background service, privilege escalation,
> or installation hooks. This repository is not published to crates.io.

## Versions

**Execarium v1.0.0** established stable YAML **schema 1** and the
Rust **crate 1.0.0**. The compatible Rust **crate 1.1.0** adds construction and
explicit per-schema validator APIs. The YAML `schema` remains the integer
`1`, not `1.0.0`. Crate **major** version and schema integer are tied;
crate minor/patch changes may improve the parser/API without changing schema
1's meaning. Breaking changes to the YAML language require **schema 2** and
crate **2.x**. No implicit compatibility fallback.

See [versioning policy](docs/VERSIONING.md), [changelog](CHANGELOG.md), and
[ChatGPT authoring guide](docs/CHATGPT.md).

## The complete v1 document

```yaml
schema: 1
shell: fish
cwd: /home/user/Projects/example
privilege: none
timeout_seconds: 600
command: cargo test
```

All six fields are **required**. There are no implicit defaults, extension fields,
environment expansion, or execution backends in schema v1.

| Field | v1 type / accepted value | Meaning |
| --- | --- | --- |
| `schema` | Integer, exactly `1` | Exact format version; unsupported versions fail closed |
| `shell` | `fish`, `bash`, `sh` | Requested local shell (not automatically launched) |
| `cwd` | Absolute POSIX directory path | Requested working directory, validated lexically only |
| `privilege` | `none`, `sudo`, `doas` | Requested privilege **metadata**, not authorization |
| `timeout_seconds` | Integer, 1–604800 | Maximum elapsed execution time, **not** queue/approval waiting time |
| `command` | Nonempty UTF-8 string | Literal shell source, no parsing or execution by Execarium |

Multiline commands use ordinary YAML block scalars:

```yaml
schema: 1
shell: fish
cwd: /home/user/Projects/example
privilege: none
timeout_seconds: 600
command: |
  cargo fmt --check
  and cargo test
  and cargo build
```

See [v1 normative specification](docs/SPEC-v1.md), [security and trust boundaries](docs/SECURITY.md),
[architecture and integration](docs/ARCHITECTURE.md), [conformance](docs/CONFORMANCE.md),
and the [roadmap](docs/ROADMAP.md).

## Design boundary

**Execarium only parses and validates.** It does not decide that a command is
safe, grant privilege, verify passwords, schedule work, inspect the filesystem,
or run a shell. `privilege: none` is a request, **not a sandbox**: the contained
shell text can still include `sudo` or `doas`. Consumers are responsible for
policy, human authorization, authentication, timeouts, and actual execution.
`timeout_seconds` is a requested limit, not a timer implemented by the library.
A consumer must enforce it at runtime or reject the job; the library neither
spawns nor terminates processes.

For v1, consumers must not silently select or infer a remote/container/VM backend.
There is deliberately no `executor` field: a shell is assumed to be local by the
specification, but each consumer must explicitly opt into that execution model.

**Clipboard scanning and automatic ingestion are deferred.** Nothing in this
repository reads the clipboard or watches it. Jobflick integration itself is also
deferred until this standalone format has been reviewed and stabilized.

## Library

Use the **Rust library** as an optional reference parser; ChatGPT and other
producers can generate conforming YAML simply by following the specification.
The library is not the format's owner. It offers both **string validation**
and **construction** of conforming YAML; the v1 wire format is unchanged.

Add the repository as a Git dependency (not published to crates.io):

```toml
[dependencies]
execarium = { git = "https://github.com/sguzman/execarium", tag = "v1.1.0" }
```

```rust
use execarium::parse;

let job = parse("schema: 1\nshell: fish\ncwd: /tmp\nprivilege: none\ntimeout_seconds: 600\ncommand: echo ok\n")?;
assert_eq!(job.schema(), 1);
assert_eq!(job.command(), "echo ok");
assert_eq!(job.timeout_seconds(), 600);
// This parses data. It does not start a process or approve the command.
# Ok::<(), execarium::Error>(())
```

This references the immutable release tag. Cargo.lock pins the resolved
commit for applications; security-sensitive consumers may additionally
pin an exact reviewed `rev` instead of `tag`. Independently of the crate
version, consumers should explicitly restrict accepted schema versions.

### Versioned validator and constructor API

The original `execarium::parse` and `execarium::validate` remain supported.
New Rust code can use the v1 namespace. This is independent of whether ChatGPT
authored the YAML or a Rust application constructed it.

```rust
use execarium::{v1, Shell, Privilege};

let job = v1::Validator::construct(
    Shell::Fish,
    "/home/user/Projects/example",
    Privilege::None,
    600,
    "cargo test",
)?;
let yaml = v1::Validator::render(&job)?;
v1::Validator::validate(&yaml)?;
assert_eq!(v1::Validator::parse(&yaml)?, job);
# Ok::<(), execarium::Error>(())
```

`execarium::validate_for_schema(1, yaml)` selects exactly validator v1;
asking for version 2 returns `UnsupportedSchema`, not an implicit fallback.
The library handles **construction + serialization + validation**. The
CLI remains validation-only, never an executor.

Read the [complete Rust API guide](docs/RUST-API.md) for methods, invariants,
compatibility and integration examples.

## CLI (validation only)

```sh
cargo run -- --version
cargo run -- validate examples/local-fish.yaml
cargo run -- validate - < examples/multiline-fish.yaml
```

A successful validation exits 0 and prints the schema, shell, requested
privilege, and execution timeout *without printing command text*.
Invalid documents exit nonzero.

## Development

```sh
cargo fmt --all -- --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
cargo run --example create_yaml
cargo run -- validate examples/local-fish.yaml
```

The CI workflow runs those checks and tests valid and invalid fixtures.

## Rules for evolving the format

1. **Version first:** schema 1 is now stable. Any changes that make existing
   YAML incompatible or change semantics require a new integer `schema`
   version and a matching crate major version.
2. **Fail closed:** unknown keys, duplicate keys, unsupported versions and malformed
   YAML are errors; no silent coercions.
3. **No accidental authority:** parsing is never consent to execution; privilege
   requests must be authorized in the consuming application.
4. **No dynamic interpretation:** v1 has no variable interpolation, templates,
   include files, references, anchors, aliases, or custom YAML tags.
5. **Strict consumers:** a consumer decides its accepted schema versions and
   shells, independent of what this library can parse.

## Deferred on purpose

Automatic clipboard collection, auto-enqueue, container/jail/Docker/Podman,
microVM/Firecracker, remote execution, SSH and CI runners, dependencies,
general resource budgets (CPU/memory), and rich authorization workflows are
**not v1 work**. The execution-duration limit is the sole v1 resource bound.

## License

MIT. See [LICENSE](LICENSE).
