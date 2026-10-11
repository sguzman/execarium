# Security model and trust boundaries

**Execarium is a data parser, not a security boundary around shell processes.**
Treat every document as potentially adversarial input, including documents
copied from websites, terminal output, chat messages, files, or other software.

## Threat model

An attacker may control an entire YAML document. They may attempt to:
* smuggle executable shell text into a harmless-looking job;
* use unknown keys, duplicate keys, tags, anchors or aliases to change meaning;
* exhaust CPU or memory with giant YAML inputs;
* supply a working directory that points to untrusted or sensitive resources;
* assert `privilege: none` while embedding `sudo` in `command`;
* request a timeout the consuming executor does not honor;
* trick a consumer into using automatic acceptance or silently overriding its
  configured shell and authorization policy.

### What Execarium does

* Limits the entire document and each command before retaining unbounded data.
* Requires a fixed set of six keys and exact schema version.
* Rejects tags/aliases/anchors/directives, unknown keys, duplicate keys, bad
  field types, invalid enum values, and malformed inputs.
* Rejects relative directories, parent-directory path segments, line breaks,
  and embedded NUL in `cwd`.
* Validates `timeout_seconds` as an exact integer between 1 and 604800.
* Produces typed immutable-accessor data instead of executing a shell.
* Never reads clipboard data, polls changes, launches a daemon, or transmits
  information over the network.

These are **parser properties**, not a sandbox. The library does not validate
shell command syntax or perform a safety assessment.

### What Execarium does not do

* Prove a command safe or grant permission to run it.
* Guarantee a working directory stays within a filesystem boundary.
* Enforce the absence of privilege escalation when `privilege: none`.
* Prevent shell injection if a consumer interpolates untrusted strings.
* Encrypt or hide commands, passwords, command output, or filesystem paths.
* Require human approval or authenticate via `sudo` / `doas`.
* Prevent a separate application from reading a clipboard or launching a job.
* Enforce or guarantee the requested timeout, kill detached descendants, or
  undo side effects produced before the timeout.

## Required responsibilities of a consuming executor

1. **Deliberate intake:** Accept jobs only through a user-approved source. A
   valid schema is **not** consent to schedule or execute.
2. **Policy check:** Compare the entire parsed job against the consumer's
   allowed schema versions, shells, paths, privilege modes, and quotas.
3. **Approval binding:** For elevated jobs, bind any explicit human approval
   to the exact immutable job description; edits invalidate approval.
4. **Credential isolation:** Never embed a password in YAML, job metadata, IPC
   payloads, logs or desktop notifications. Use a trusted OS authentication
   channel/TTY at execution time; do not assume cached authentication remains
   valid while queued.
5. **No silent substitutions:** Unsupported shell or privilege modes must be
   rejected or remain held; never downgrade or upgrade them implicitly.
6. **Process policy:** Enforce the requested `timeout_seconds` as an elapsed
   execution-time limit using a monotonic clock; start the timer at process
   launch (not at submission or authorization). Reject unsupported limits,
   record expiration distinctly, terminate and reap owned processes, and
   apply additional isolation/cancellation rules as appropriate. A parser
   cannot confine a shell or ensure detached processes terminate.
7. **Observability:** Distinguish rejected jobs, pending approval, queued work,
   running work, and uncertain crash recovery outcomes.
8. **Secret hygiene:** Treat even ordinary shell text and error messages as
   potentially sensitive. Consumers should not put raw commands on lock-screen
   notifications or public issue trackers.

## Privileged work in a future consumer

`privilege: doas` and `privilege: sudo` are request metadata. They should
normally enter a **held / authorization-required state**, with no worker slot
occupied. The user reviews exactly what would run, grants job-specific
authorization, and performs OS authentication when needed. **No password
belongs inside an Execarium document.** The schema does not define this queue
state machine; each consumer must implement it.

## Clipboard policy (deliberately deferred)

There is **no automatic clipboard monitor, parser loop, collection, or command
dispatch** in this project. Even strict YAML syntax is not a secure trigger:
a malicious website can supply perfectly valid YAML. Future clipboard
integration requires a separate, explicitly authorized design and opt-in
configuration. The format MUST NOT silently enable it.

## Reporting vulnerabilities

Open a GitHub issue for non-sensitive defects. For potential secrets or
exploitable vulnerabilities, use an appropriate private disclosure channel
rather than posting exploit details or private user data in a public issue.
