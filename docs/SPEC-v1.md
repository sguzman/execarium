# Execarium schema 1: normative specification

**Status:** stable **Execarium v1.0.0**. **Wire schema:** integer `1`.
**Reference crate:** `1.0.0`. Normative schema changes require a new major version.

This document defines the portable wire representation of an Execarium **job
request**. It is not an executable file format and does not authorize the work
it describes. The key words **MUST**, **MUST NOT**, **SHOULD**, and **MAY** are
normative in this document.

## 1. Scope

Schema 1 describes a *single local shell command* and its associated working
directory, shell dialect, requested privilege mode, and execution-duration
limit. There is **no**
executor/backend selector. The format has no concept of a remote target,
container, VM, dependencies, resources, triggers, clipboard events, or secrets.

Consumers MAY support only a subset of the available shells or privilege modes.
They MUST reject unsupported configurations instead of substituting another
shell or privilege mode.

## 2. Minimal complete example

```yaml
schema: 1
shell: fish
cwd: /home/user/Projects/example
privilege: none
timeout_seconds: 600
command: cargo test
```

All six keys are **required**, appear exactly once, and have no defaults. Key
ordering is not significant. Additional keys are invalid. The top level MUST
be one YAML mapping, not a sequence or scalar.

## 3. Field definitions

### `schema` — unsigned integer

The sole implemented version is `1`. It MUST be a YAML integer, not a quoted
string, float, boolean, or null. A consumer MUST NOT infer a version from
missing fields or silently migrate an unsupported version. A parser supporting
schema 2 in the future must still honor the source document's stated version.

### `shell` — enumeration

One of the exact, lowercase strings `fish`, `bash`, or `sh`. The value
specifies the **requested shell family**, not an executable path or arguments.
Availability, dialect-specific syntax validation, and spawning belong to the
consumer. The Execarium parser never invokes a shell.

A consumer that only runs Fish MUST reject `bash` and `sh` rather than
silently running those commands in Fish.

### `cwd` — absolute POSIX path string

The requested working directory. The parser enforces:

* 1–4096 bytes of UTF-8;
* starts with `/` (root `/` is valid);
* contains no NUL, carriage return, or newline;
* contains no path segment equal to `.` or `..`.

The parser does **not** expand `~`, `$HOME`, variables, globbing, or symbolic
links, and does **not** access the filesystem. The consumer MUST decide whether
the path exists, is accessible, and is allowed by its execution policy. Path
traversal via symlinks is not ruled out by the lexical validator; consumers
requiring confinement must enforce it separately.

### `privilege` — enumeration

One of the exact, lowercase strings:

| Value | Requested mode | Authorization effect |
| --- | --- | --- |
| `none` | No privilege wrapper requested | **None** |
| `sudo` | Consumer may support a sudo workflow | **None** |
| `doas` | Consumer may support a doas workflow | **None** |

This field is a **description** and MUST NOT be interpreted as permission.
`none` is not a security sandbox; the command text could itself contain
`sudo`, `doas`, or another elevation mechanism. Consumers MUST make their own
approval and OS authentication decisions, and MUST NOT derive authorization
solely from a YAML document.

A future Jobflick integration may hold privileged jobs for explicit review and
out-of-band authentication. That is *not* a schema 1 parser feature.

### `timeout_seconds` — unsigned integer

The maximum **elapsed wall-clock running time** requested for the local shell
job, in **whole seconds**. It MUST be an unquoted YAML integer in the inclusive
range **1–604800** (seven days). Zero, negative integers, fractions,
floating-point values, strings (including `"60"`), booleans, and null are
invalid. No default or unlimited sentinel exists in schema 1.

The requested timeout starts **when the shell process is launched**. Time
spent waiting in a queue, waiting for human approval, or waiting for OS
authentication does not count. A consumer MUST enforce the limit with its
own runtime/process supervision, or **reject the job** if it cannot. It MAY
apply a stricter documented policy by rejecting the request; it MUST NOT
silently change its timeout. On expiration, the consumer SHOULD terminate
the job's process group, reap processes it owns, and report a distinct timeout
outcome. A requested timeout does not guarantee that detached descendants or
external side effects can be stopped.

**Execarium only validates and returns the timeout.** It does not start a timer,
kill processes, or guarantee that another application will enforce the limit.

### `command` — UTF-8 string

Nonempty after trimming whitespace, with a maximum of 131072 UTF-8 bytes
(128 KiB). NUL bytes are invalid. Newlines, whitespace, shell punctuation,
quotes, and ordinary Unicode are accepted. A plain scalar or a YAML literal
block scalar (`|`, `|-`, `|+`) may be used. YAML's block chomping rules
determine whether the final newline is included in the exact parsed command.

The parser MUST preserve the decoded command string; it MUST NOT perform
shell interpolation, command substitution, syntax checks, templating, whitespace
normalization, or execution.

```yaml
schema: 1
shell: fish
cwd: /home/user/Projects/example
privilege: none
timeout_seconds: 600
command: |
  cargo fmt --check
  and cargo test
```

## 4. YAML subset and transport

* Input is UTF-8 text.
* A complete document has an upper limit of 262144 bytes (256 KiB), checked
  **before** parsing.
* Exactly one YAML document is accepted.
* Duplicate keys are rejected, including duplicates of recognized keys.
* Unknown keys are rejected.
* YAML anchors, aliases, custom/standard explicit tags, version directives, and
  tag directives are rejected. They must not be used to introduce indirection.
* YAML comments and ordinary quoted scalars are allowed.
* Numeric, boolean, null, sequence, or mapping values are invalid for string
  fields even if a shell or YAML implementation could coerce them.
* `timeout_seconds` MUST be a YAML unsigned integer in range, never a
  coerced string or floating-point scalar.
* Successful parse returns a typed job description and **has no side effects**.

YAML has scalar inference and quoting rules. Producers SHOULD quote ambiguous
values, especially strings that could be interpreted as booleans or numbers,
and SHOULD use a literal block scalar for multi-line shell source.

Consumers MUST NOT use a generic YAML loader with duplicate-key overwriting
or unknown-field tolerance and call the result conformant.

## 5. Validation versus execution

A conforming parser validates **shape, types, enumeration values, byte limits,
lexical path constraints, and the requested timeout bound only**. Successful parsing is not evidence that
the command is safe, meaningful, runnable, syntactically valid for the chosen
shell, authorized, or guaranteed to finish. The consumer MUST explicitly choose
an execution policy and enforce it independently of this document.

## 6. Schema version evolution

The `schema` integer controls the *wire format*, not the library's Cargo
version. For example, a library release 0.2.0 could still support only
`schema: 1`. A producer must emit only the version agreed with its consumer.
Consumers SHOULD pin a reviewed parser revision and MUST explicitly allow
supported schema versions. Unknown versions are rejected by default. Schema 1
will not acquire new required or optional keys by silent interpretation:
materially different semantics belong in a new schema version.

**Release history:** The sixth required field, `timeout_seconds`, was added
during the pre-release drafting stage. The five-field prototype is not a
released version and is rejected by the stable v1 parser. Effective with
`v1.0.0`, schema 1 is frozen: breaking or semantic changes MUST use a new
integer `schema` and the matching crate major version. Consult
[VERSIONING.md](VERSIONING.md).

## 7. Compatibility checklist

A conforming implementation MUST accept the valid fixture files in
`examples/` and reject invalid cases documented in
[CONFORMANCE.md](CONFORMANCE.md). See [SECURITY.md](SECURITY.md) for the
authorization and clipboard threat model.
