# Execarium schema 2: ordered command sequences

**Status:** new strict YAML wire schema. Version integer `2`; schema 1 remains unchanged.
**Semantics:** a bounded ordered sequence of independent local shell invocations,
executed by the consumer with `and`-equivalent stop-on-first-failure behavior.

## Complete example

```yaml
schema: 2
shell: fish
cwd: /home/user/Projects/example
privilege: none
timeout_seconds: 600
commands:
  - cargo fmt --check
  - cargo test
  - cargo build
```

All six keys (`schema`, `shell`, `cwd`, `privilege`, `timeout_seconds`,
`commands`) are mandatory and unique. Unknown keys are rejected. In particular,
`command` and `commands` are mutually exclusive across the two versions:
schema 1 accepts only `command`; schema 2 accepts only `commands`.

## Values and limits

- `schema`: exact YAML unsigned integer `2`, with no implicit fallback.
- `shell`: `fish`, `bash`, or `sh`; consumers must reject unsupported modes.
- `cwd`: same absolute POSIX path constraints as schema 1; every step starts here.
- `privilege`: `none`, `sudo`, or `doas` metadata only, never authorization.
- `timeout_seconds`: one wall-clock budget for the **entire running sequence**,
  1 to 604800 seconds, excluding time spent in a job queue.
- `commands`: YAML sequence of 1 to 256 nonempty, literal UTF-8 shell-source
  strings. Each step observes schema-1 command validation (<=131072 UTF-8 bytes,
  no NUL), and the aggregate command text is limited to 131072 UTF-8 bytes.

The complete UTF-8 YAML document is limited to 262144 bytes. Reject duplicates,
unknown keys, anchors, aliases, explicit tags, directives, and multiple documents.
Do not coerce numbers, booleans, nulls or mappings into command strings.

## Execution contract for consumers

1. Validate the complete document and the selected shell/privilege policy **before** queueing.
2. Execute steps **in source order**, each as one shell invocation with the
   same requested `cwd`. Do not parse a step as a shell-independent command
   argument vector; the step remains source for the selected shell.
3. After step *i* exits 0, start step *i+1*. If step *i* exits nonzero or
   cannot spawn, **do not start subsequent steps**. Report the failed
   1-based step index and its exit status or spawn error.
4. Enforce the single execution timeout across the **whole sequence**, from
   the first shell launch until the final step exits. It does not reset on
   each step. On expiration, stop the active process group and mark the whole
   job TimedOut; do not launch pending steps.
5. Cancellation similarly stops the active step and prevents later steps.
6. Shell process state (such as transient `cd`, local variables, and
   functions) is **not shared** between steps. Put shared context in the
   structured `cwd`; use a schema-1 `command: |` script for commands that
   intentionally require a shared shell context.
7. A step is defined by its shell process's final exit status. For example,
   `false; true` is one step with a successful final status; schema 2
   is not a substitute for shell-level `and` validation inside a step.

Execarium itself **never executes** commands, accesses the filesystem, grants
privileges, or enforces timeouts. A consumer that cannot uphold these semantics
must reject schema 2, not silently treat the sequence as a schema-1 script.

## Versioning

Schema 1 is frozen and remains a valid six-field `command:` contract.
Schema 2 is a distinct six-field `commands:` contract; there is no
implicit upgrade or downgrade and no `commands:` key added to schema 1.
Crate SemVer is independent of the YAML schema integer. Use
`execarium::v2::Validator` or multi-schema `parse_document` for version 2.
