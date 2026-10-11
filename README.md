# Execarium

**A strict, versioned description of local shell jobs. Not a command runner.**

Execarium defines and validates a small YAML document describing **what shell command
a consumer is being asked to run**, in which working directory, and with what
requested privilege level. It is an independent Rust library and a validation-only CLI,
so Jobflick and future projects can share one contract without importing a scheduler.

> **Status:** v1 specification and Rust reference parser, pre-MVP. No execution,
> clipboard monitoring, background service, privilege escalation, or installation hooks.

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

Add this repository as a Git dependency until the project is published:

```toml
[dependencies]
execarium = { git = "https://github.com/sguzman/execarium", rev = "<reviewed-commit-sha>" }
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

The dependency pin is intentionally a placeholder, not an actual commit SHA.
Applications should pin a reviewed revision and explicitly restrict accepted
schema versions.

## CLI (validation only)

```sh
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
cargo run -- validate examples/local-fish.yaml
```

The CI workflow runs those checks and tests valid and invalid fixtures.

## Rules for evolving the format

1. **Version first:** after the first stable v1 release, incompatible changes
   require a new `schema` integer. Draft v1 was explicitly revised to add
   `timeout_seconds` before any release/tag.
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
