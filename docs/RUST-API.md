# Rust API reference — Execarium crate 1.1.2 (MVP)

Execarium has **two distinct deliverables**:

1. The independent, language-neutral [YAML schema 1](SPEC-v1.md).
2. An optional **Rust reference library** providing typed job construction,
   schema-specific validation, and YAML serialization.

The Rust library never executes the described command and is never the
authority that grants privileges. Jobflick or any other consumer must explicitly
accept the document and enforce runtime policy, authorization, timeout,
scheduling and process cleanup.

## Why Rust?

Programs can create correctly typed requests without manually concatenating
YAML strings, and validate incoming text without implementing the entire
specification themselves. This avoids duplicated format-specific validation
logic across different Rust projects. ChatGPT can produce the same YAML
without using Rust; a consumer can validate the result with this library or
its own conformant implementation.

The stable schema and crate versions are distinct:

| Dimension | Value |
| --- | --- |
| YAML wire schema | `schema: 1` |
| First stable crate | `1.0.0` |
| Rust API additions | `1.1.0` |
| Tested MVP crate release | `1.1.1` |
| Independent crate versioning policy | `1.1.2` |
| Validator namespace | `execarium::v1` |
| Future breaking YAML changes | a new schema integer, independent of crate major |
| Future breaking Rust API changes | new crate major, independent of schema |

## Installation

Use the pinned GitHub tag until any future crates.io publication:

```toml
[dependencies]
execarium = { git = "https://github.com/sguzman/execarium", tag = "v1.1.2" }
```

Rust 2021 edition, at least Rust 1.76 as declared in Cargo.toml.
Cargo.lock locks resolved Git revisions. Version selection still must be
independent of the user's consent to run a job.

## 1. Validate an existing YAML string

```rust
use execarium::v1;

let source = "schema: 1\nshell: fish\ncwd: /tmp\nprivilege: none\ntimeout_seconds: 600\ncommand: cargo test\n";
v1::Validator::validate(source)?;
let job: v1::Job = v1::Validator::parse(source)?;
assert_eq!(job.timeout_seconds(), 600);
# Ok::<(), execarium::Error>(())
```

The existing `execarium::parse` and `execarium::validate` functions remain
supported for projects already using the v1.0.0 API.

## 2. Construct from typed Rust fields

```rust
use execarium::{v1, Shell, Privilege};

let job = v1::Validator::construct(
    Shell::Fish,
    "/home/user/Projects/example",
    Privilege::None,
    600,
    "cargo test",
)?;
assert_eq!(job.schema(), 1);
assert_eq!(job.command(), "cargo test");
# Ok::<(), execarium::Error>(())
```

Construction invokes the **same field validator as YAML parsing**. It will
reject a relative `cwd`, dot/parent segments, invalid timeout, blank command,
embedded NUL and excessive lengths. It does not confirm that `cwd` exists,
that Fish is installed, that any command is correct, or that the user approved
it. `Job` has private fields, so callers cannot bypass these checks via a
public struct literal.

## 3. Render to YAML

```rust
use execarium::{v1, Shell, Privilege};

let job = v1::construct(Shell::Bash, "/tmp", Privilege::None, 30, "echo 'ok'")?;
let document: String = v1::Validator::render(&job)?;
assert_eq!(v1::Validator::parse(&document)?, job);
# Ok::<(), execarium::Error>(())
```

Equivalently, call `job.to_yaml()`. The serializer emits exactly the
six schema-1 data fields, not YAML directives to execute anything. It escapes
or quotes ambiguous scalar values when needed. Round-trip tests cover string
values such as `null`, `true`, shell punctuation and multiline commands.

## 4. Select a schema validator explicitly

```rust
use execarium::{parse_for_schema, validate_for_schema, SUPPORTED_SCHEMAS};

assert_eq!(SUPPORTED_SCHEMAS, &[1]);
let source = "schema: 1\nshell: sh\ncwd: /tmp\nprivilege: none\ntimeout_seconds: 60\ncommand: printf ok\n";
validate_for_schema(1, source)?;
let job = parse_for_schema(1, source)?;
assert_eq!(job.schema(), 1);
assert!(parse_for_schema(2, source).is_err()); // no version guessing
# Ok::<(), execarium::Error>(())
```

Explicit version selection checks both the **caller-selected version** and
the **document's declared schema**. A request to validate with a nonexistent
v2 validator fails closed; a source declaring v2 passed to v1 also fails.

A future library release can add `v2::Validator` alongside `v1::Validator`
without necessarily bumping its Rust major version, provided its public API
change is compatible. Conversely, a breaking Rust API redesign may increment
the crate major while still supporting only schema 1. The two tracks are
independent. This repository currently implements **only v1**. It does not
invent v2 syntax or import any historical draft as a separate format.

## 5. The API

| Public interface | Responsibility |
| --- | --- |
| `v1::Validator::parse(&str)` | Strict version-1 YAML parsing |
| `v1::Validator::validate(&str)` | Version-1 validation without returning fields |
| `v1::Validator::construct(shell,cwd,privilege,seconds,command)` | Construct and validate typed v1 job |
| `v1::Validator::render(&Job)` | Serialize validated v1 job to YAML |
| `v1::{parse,validate,construct}` | Functional aliases |
| `parse_for_schema(u64,&str)` | Exact validator selection + parse |
| `validate_for_schema(u64,&str)` | Exact validator selection + validity check |
| `SUPPORTED_SCHEMAS` | Authoritative list of implemented schema integers |
| `Job::to_yaml()` | Render validated job as YAML |
| `Job::{schema,shell,cwd,privilege,timeout_seconds,command}` | Read-only fields |
| `Shell` | `Fish`, `Bash`, `Sh` |
| `Privilege` | `None`, `Sudo`, `Doas` |
| `Error` | Strict typed error for unsupported version, YAML, path, command and timeout failures |

`Job` remains a **value object**, not an instruction to the operating system.

## 6. Error behavior

* A total source document over 256 KiB is rejected before YAML parsing.
* Unknown, missing, or repeated keys, unsafe YAML features, and unsupported
  versions fail closed.
* String fields must actually be YAML strings, not coerced nulls or integers.
* The timeout is an unquoted unsigned YAML integer from 1–604800 seconds.
* `cwd` must be an absolute literal POSIX path without `.` or `..`
  segments, NUL or newlines (maximum 4096 UTF-8 bytes).
* `command` must be nonempty and at most 128 KiB UTF-8; it cannot contain NUL.
* Typed construction enforces the same **field rules** as parsing. YAML-only
  rules (such as rejecting anchors and duplicate keys) apply to text input.

If an application needs to transform a validated document, it should
construct a **new** `Job` using the constructor so the result is validated
again, rather than mutating a previously approved one. Privileged-job
approval belongs to the consumer and must be rebound to that new description.

## 7. Security and integration

Parsing and constructing have no filesystem, clipboard, subprocess, sudo/doas,
or network side effects. Rendered YAML does **not** imply user approval.
A consumer must explicitly accept the request, check shell and cwd policy,
hold privileged work for separate human authorization, enforce the requested
runtime timeout at process launch, and supervise jobs.

The v1 format has one shell-specific `command` string. Sequential independent
`commands:` arrays are not part of schema 1; they would be a future schema
and an independently versioned library release.

## 8. Tests, examples, and CI

```sh
cargo test --all-targets
cargo test --doc
cargo clippy --all-targets -- -D warnings
cargo run --example create_yaml
cargo run -- validate examples/local-fish.yaml
```

* `tests/conformance.rs`: v1 acceptance and rejection rules.
* `tests/versioned_api.rs`: per-version selection, typed construction,
  validation parity, rendering/parse round-trips, and compatibility checks.
* `examples/create_yaml.rs`: complete compileable Rust construction example.
* `.github/workflows/ci.yml`: fmt, Rust and doc tests, Clippy,
  the construction CLI pipeline, and negative validation tests.

The **schema-1 specification** remains [SPEC-v1.md](SPEC-v1.md); this page
describes the Rust convenience implementation, not new YAML behavior.
