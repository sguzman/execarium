# Writing Execarium YAML with ChatGPT or other AI systems

This is an **authoring guide**, not a command-execution prompt. The normative
source of truth is [SPEC-v1.md](SPEC-v1.md). These instructions are suitable
for any AI system, developer, or script that *produces* Execarium documents.

## Target

**Release:** Execarium v1.0.0.
**YAML:** `schema: 1` (integer). Six required keys and no others.

```yaml
schema: 1
shell: fish
cwd: /home/user/Projects/example
privilege: none
timeout_seconds: 600
command: cargo test
```

## Authoring checklist

1. Emit exactly one YAML mapping with exactly six keys: `schema`,
   `shell`, `cwd`, `privilege`, `timeout_seconds`, `command`.
2. Write `schema: 1` as an **integer**, not `1.0` or `"1"`.
3. Choose `shell` from `fish`, `bash`, `sh`; don't improvise an
   executable path or assume dialects are interchangeable.
4. Supply an **absolute POSIX** `cwd` appropriate for the recipient's host,
   without `.` or `..` path segments. Do not fabricate the user's path.
5. Write `privilege: none` unless the user specifically requests a
   privileged operation and the intended consumer supports an approval flow.
   `doas` and `sudo` are requests, not approval or authentication.
6. Supply `timeout_seconds` as an unquoted whole integer **1–604800**,
   reflecting the expected job duration. Its timer starts at shell launch,
   not while the job is waiting for user approval or a queue slot.
7. Write nonempty literal shell source under `command`. Use a YAML
   `|` block scalar for multiple shell lines. A multiline `command` is
   still one **shell-specific script**, not a shell-independent step list.
8. Do not add `executor`, `commands`, `env`, `description`, or other
   fields: schema 1 rejects all unknown keys.
9. Never include passwords or other secrets in the document.
10. Avoid treating a valid document as authorization to execute it. The
    consumer must verify user intent and enforce its execution policy.

If the working directory, privilege mode, or other required information
cannot be determined reliably, ask for the missing detail rather than
inventing a plausible-looking value.

## Valid multiline example

```yaml
schema: 1
shell: fish
cwd: /home/user/Projects/example
privilege: none
timeout_seconds: 1200
command: |
  cargo fmt --check
  and cargo test
  and cargo build
```

Note that `and` is **Fish shell syntax**, not an Execarium sequencing
primitive. v1 intentionally leaves script semantics to the selected shell.

## Using the Rust library (optional)

Human and AI producers need not use Rust to write valid schema-1 YAML.
For programs that need to construct typed jobs or validate untrusted YAML,
crate 1.1.0 adds `v1::Validator::{construct,render,validate,parse}`.
Use these APIs without invoking a shell; see [RUST-API.md](RUST-API.md).

## Validation before delivery

```sh
execarium --version
execarium validate job.yaml
```

The CLI confirms conformance only; it never queues or executes anything.
If the validator is unavailable, check the document against SPEC-v1.md;
do not claim machine validation that was not performed.

## What's deferred

Automatic clipboard scanning and enrollment, containers, jails, Docker,
Firecracker, microVMs, remote/CI execution, automatic sudo/doas
authentication, and independent `commands:` lists are **not** part of
the stable v1 specification.

## Schema-2 sequences (independent shell steps)

Use this separate version only when the task calls for distinct ordered steps
that must stop on the first failure:

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

The single `command` field is **not** accepted in schema 2; `commands`
is **not** accepted in schema 1. Every step is a nonempty shell string;
the list contains 1–256 steps, 128 KiB total command source. A consuming
executor must run steps one at a time, stop on the first nonzero status,
report the failed step, and enforce one timeout for the whole sequence.
Steps do not share transient shell state. Use schema-1 `command: |` for
a script intentionally sharing shell state.

Refer to [SPEC-v2.md](SPEC-v2.md), not the v1 authoring checklist, for
all normative schema-2 details. Execarium still performs **no execution**.

## Consumer warning

Generated YAML is untrusted input. Reading it, validating it, or copying
it to a clipboard must never automatically authorize its execution.

## Schema-3 execution environments (post-MVP)

Use schema 3 if the intended executor supports live desktop/GPU context
resolution. A complete example:

```yaml
schema: 3
shell: fish
cwd: /home/user/Projects/example
privilege: none
timeout_seconds: 600
environment:
  session: hyprland
  gpu: nvidia
command: |
  hyprctl activewindow
```

Always declare both `environment.session` (`inherit`, `wayland`, `hyprland`)
and `environment.gpu` (`inherit`, `integrated`, `nvidia`). Declare exactly
one `command` script or `commands` list. Hyprland includes the Wayland
connection; GPU preference is independent. Never omit fields or silently
downgrade an unsupported session or GPU. Use `inherit` explicitly when no
override is requested. See [SPEC-v3.md](SPEC-v3.md) for the complete contract.

Execarium never executes the command or resolves environments. The human
decides which submitted YAML to execute; consumers must implement the
requested context before accepting schema 3.
