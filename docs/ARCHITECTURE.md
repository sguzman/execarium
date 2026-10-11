# Architecture and integration guide

## Ownership

Execarium owns the **wire contract** and a **reference parser**. It must remain
reusable by Jobflick and unrelated consumers. It does not own any process
scheduler, desktop UI, hotkey, queue, password prompt, clipboard policy, or
execution backend.

```text
Explicit producer (user, tool, program)
          |
          v
Execarium document (six-key YAML, schema: 1)
          |
          v
Execarium library -> validate structure -> typed Job
          |
          v
Consumer-specific policy + explicit approval if needed
          |
          v
Consumer's OWN queue, runner and result lifecycle
```

There is intentionally no automatic clipboard path in this diagram.

## Rust surface

```rust
let spec: execarium::Job = execarium::parse(source)?;
assert_eq!(spec.schema(), execarium::SCHEMA_VERSION);
match spec.shell() {
    execarium::Shell::Fish => { /* consumer may support Fish */ }
    other => { /* reject if unsupported */ }
}
match spec.privilege() {
    execarium::Privilege::None => { /* normal policy */ }
    execarium::Privilege::Doas | execarium::Privilege::Sudo => {
        /* hold for explicit approval, not auto-execution */
    }
}
let max_run_time = std::time::Duration::from_secs(spec.timeout_seconds());
// Consumer must enforce max_run_time at process launch; waiting in the queue
// or for credentials does not count. No timer exists in Execarium itself.
let _ = max_run_time;
// Validation has not started a process.
```

Public API:
* `parse(&str) -> Result<Job, Error>`: existing strict shape/semantic validation;
* `validate(&str) -> Result<(), Error>`: existing validity check;
* `v1::Validator::{parse,validate,construct,render}`: version-specific
  validation, typed construction and YAML emission;
* `parse_for_schema(version, source)` / `validate_for_schema(version, source)`:
  explicit validator selection (no fallback);
* `Job::to_yaml()`: serialize a validated job to a YAML document;
* `Job::{schema,shell,cwd,privilege,timeout_seconds,command}`: immutable accessors;
* `Shell` and `Privilege`: closed enumerations;
* `SCHEMA_VERSION`, `IMPLEMENTATION_VERSION`, `MAX_DOCUMENT_BYTES`,
  `MAX_COMMAND_BYTES`, `MAX_CWD_BYTES`, `MIN_TIMEOUT_SECONDS`,
  `MAX_TIMEOUT_SECONDS`: version and explicit limits.

Both parsed and constructed jobs are checked by the **same field
validator**, so typed construction cannot bypass timeout, path and command
limits. Serialization produces a complete six-field schema-1 YAML document.
The constructor is not a process runner.

The command and working directory are returned as decoded UTF-8 strings;
Execarium does not normalize the command, resolve paths, interpret permissions,
or compile the selected shell's syntax. A consumer SHOULD record the exact
approved spec, or its trusted digest, before enqueueing privileged work.

## Jobflick as first intended consumer — NOT YET INTEGRATED

Jobflick currently has a Fish-only raw command queue. After v1 is reviewed:

1. Add a **separate explicit** structured-submission entry point. Preserve
   existing raw `jobflick submit` compatibility.
2. Use Execarium only for parsing and validation.
3. Check that schema `1` and shell `fish` are allowed by Jobflick.
4. Resolve and validate `cwd` under Jobflick's own execution policy.
5. `privilege: none`: permit normal enqueue only under the user's explicit
   submission and Jobflick policy.
6. `privilege: doas` / `sudo`: hold for explicit human approval; after
   approval, use trusted OS authentication when scheduling allows.
7. Require the requested `timeout_seconds` to be supported, and supervise
   elapsed wall time starting when the shell launches; terminate and report
   expiration without falsely reporting success.
8. Persist, schedule, cancel and report through Jobflick's existing queue.

**Do not** infer that this plan authorizes immediate Jobflick modifications.
Jobflick development is currently paused until the specification is agreed.

## Separate concerns

| Concern | Owner |
| --- | --- |
| YAML syntax and version rules | Execarium |
| Schema fields and constraints | Execarium |
| Transport, hotkeys, clipboard reads | Consumer/application |
| Credential entry and approval | Consumer + OS |
| Queue fairness and concurrency | Consumer/application |
| Timeout value/type validation | Execarium |
| Runtime timeout enforcement, termination and reporting | Consumer/application |
| Process launching and OS sandboxing | Consumer/application |
| Output, logs, notifications, retention | Consumer/application |
| Remote/container/VM execution | Deferred outside schema 1 |

## Conformance

The reference implementation uses Serde for strict field/type decoding and
an independent YAML token scanner to explicitly reject YAML indirection features
without rejecting literal shell operators. See
[CONFORMANCE.md](CONFORMANCE.md) for accepted/rejected test coverage.

## Compatibility policy

Consumers MUST require an explicitly supported YAML schema version. Rust
crate SemVer and YAML schema integers are **independent**: a future 2.x
Rust crate may still support schema 1, and compatible Rust releases may
add additional schema validators. Such support must be explicit and retain
earlier schema semantics, never silently fall back or reinterpret documents.
Pin a release tag or reviewed revision and check schema compatibility using
`SUPPORTED_SCHEMAS` or a specific versioned validator. See
[VERSIONING.md](VERSIONING.md) and [RUST-API.md](RUST-API.md).
