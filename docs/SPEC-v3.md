# Execarium schema 3: execution context for local jobs

**Status:** strict YAML wire schema 3. Schemas 1 and 2 are frozen and unchanged.
**Purpose:** allow a producer to declare which graphical session and GPU a local command needs. The consumer resolves the request when the job launches. Execarium only parses, validates, constructs and renders.

## Complete script example

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

## Complete ordered-steps example

```yaml
schema: 3
shell: fish
cwd: /home/user/Projects/example
privilege: none
timeout_seconds: 600
environment:
  session: wayland
  gpu: integrated
commands:
  - cargo fmt --check
  - cargo test
  - cargo build
```

Exactly seven top-level keys are mandatory: `schema`, `shell`, `cwd`, `privilege`, `timeout_seconds`, `environment` and exactly **one** of `command` or `commands`. Unknown and duplicate keys are rejected.

`schema` must be the unquoted unsigned YAML integer `3`. `shell` is exactly `fish`, `bash`, or `sh`. `privilege` is exactly `none`, `sudo`, or `doas`, retained as metadata (not an authorization mechanism). `cwd` is a literal absolute POSIX path (1–4096 UTF-8 bytes) without dot or parent components, NUL, carriage return or newline. `timeout_seconds` is an unquoted unsigned integer in 1–604800, measuring elapsed running time only.

## Mandatory environment mapping

`environment` contains **exactly** the two mandatory lowercase, case-sensitive string enum fields below, including when both request inheritance.

| Field | Value | Meaning |
| --- | --- | --- |
| `session` | `inherit` | Preserve the executor's inherited session environment |
| | `wayland` | Resolve an active Wayland graphical session |
| | `hyprland` | Resolve an active Hyprland session **including Wayland** |
| `gpu` | `inherit` | Make no GPU-specific override |
| | `integrated` | Prefer the integrated GPU |
| | `nvidia` | Prefer an NVIDIA GPU |

These dimensions are independent. A headless CUDA job may request `session: inherit` with `gpu: nvidia`; a Hyprland window-control job needs `session: hyprland` but may use `gpu: inherit`.

**Consumer contract:** Resolve requested session/device context when each job **starts**, not when it was submitted or when a long-running daemon originally started. For Wayland this typically includes `XDG_RUNTIME_DIR` and `WAYLAND_DISPLAY`; Hyprland also needs `HYPRLAND_INSTANCE_SIGNATURE` and access to the compositor's IPC socket. GPU preference is API-specific, not a guarantee that any arbitrary process honors a hint. Consumers MUST document supported routing and fail explicitly when a requested session or device cannot be established or supported, without silent fallback. `inherit` means no requested override; it does not imply isolation or guarantee an empty environment.

Execarium does not discover host sessions, authenticate users, grant privileges, configure GPUs, inspect sockets or execute anything.

## Command shapes

* `command`: one nonempty literal UTF-8 script, maximum 131072 bytes and no NUL. The selected shell runs the whole script with shared shell state.
* `commands`: a sequence of 1–256 independent nonempty UTF-8 shell snippets. Maximum 131072 bytes across all elements; each element also observes the single-command constraints. Consumers execute each step in a fresh shell at `cwd`, stop on the first failed step, and enforce one aggregate timeout. Transient shell state is not shared.

Exactly one source shape must be present, and the declared environment applies equally to scripts and steps. The complete YAML document is bounded to 262144 UTF-8 bytes. Reject missing/unknown/duplicate keys at both levels, wrong scalar categories, anchors, aliases, explicit tags/directives, and multiple documents. Never infer a different version or silently convert command shape.

## API, versioning, integration

Rust crate 1.3.0 adds `v3::{Environment,Session,Gpu,CommandSource,Job,Validator}`, and `Document::V3` support in the version-explicit multi-schema parser and validator. Earlier version-1 and version-2 APIs and wire schemas are unchanged; the legacy `parse` and `parse_for_schema` still return only version-1 jobs.

A consumer such as Jobflick must explicitly opt into schema 3 **after** implementing its environment-resolution contract. A successful parse alone never executes a command; the person submitting YAML is responsible for deciding what to run. There are no additional permission prompts or authorization workflows in this schema. Rust crate SemVer is independent of the YAML `schema` integer.
