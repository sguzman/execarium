# Decision record

This is the compact rationale for design choices recorded during bootstrap.

## D1 — Independent repository

**Decision:** Execarium owns the specification, separate from Jobflick.
**Reason:** Jobflick is the first intended consumer, not the permanent owner.
Other Rust programs should be able to validate the same documents without
depending on a GUI, daemon, clipboard service, or scheduler.

## D2 — Explicit user submission only

**Decision:** No clipboard monitoring or automatic ingestion.
**Reason:** Merely encountering valid syntax must not confer execution
authority. The automatic path needs a separate future threat-model review.

## D3 — Local shell only

**Decision:** No executor/backend selector in v1.
**Reason:** Containers, microVMs, remote CI and other backends have
incompatible capabilities and security boundaries. The initial contract is
narrowly about describing ordinary local shell work.

## D4 — Privilege as request metadata

**Decision:** `none`, `sudo`, and `doas` are descriptive values only.
**Reason:** Approval, authentication, and credential lifetimes belong to the
consumer and operating system. A privileged job may be held in Jobflick
without occupying a worker slot; that behavior must not be assumed of
other Execarium consumers.

## D5 — Version and field strictness

**Decision:** All six keys mandatory, exact `schema: 1`, no unknown fields,
no silent migrations, no aliases/tags, bounded input size.
**Reason:** Consumers need a stable, independently reviewable contract with
predictable interpretation. Unknown features fail closed.

## D6 — Parser has no execution side effects

**Decision:** Both Rust API and command-line utility validate only.
**Reason:** Safe reuse requires separating a data description from a user's
authorization and any application's execution policy.

## D7 — Raw Jobflick commands preserved

**Decision:** Future Execarium ingestion adds a separate explicit structured
input path in Jobflick rather than silently replacing raw Fish submission.
**Reason:** Avoid breaking the existing mature queue and its users.

## D8 — Required bounded execution timeout (draft v1 amendment)

**Decision:** Require `timeout_seconds` as a positive integer (1–604800),
representing the maximum running time after the shell is launched. Exclude
queue waiting, pending user approval, and OS authentication from this duration.
There is no implicit or unlimited timeout.

**Reason:** Every consumer should receive an explicit, machine-checkable
execution-duration request without conflating it with user approval or scheduler
waiting time. The parser validates only; the consumer must enforce or reject
the request and report expiry as a distinct outcome.

**Version note:** This change explicitly amends the **unreleased draft** of
schema 1; it is not a silent change to a stable format. Earlier five-key
drafts are intentionally rejected. After schema 1 is released, similar
breaking changes must bump the schema version.

## D9 — Stable v1.0.0 and shared major versioning

**Decision:** Declare the current six-field YAML contract as stable schema 1
and the existing Rust library as crate v1.0.0. The wire `schema` stays the
integer 1. Crate **major** versions track schema integers; crate minor/patch
releases may change only compatible implementation/API details. Incompatible
YAML changes must bump both schema integer and crate major version.

**Reason:** Gives human, ChatGPT, and programmatic consumers a clear stable
reference while distinguishing library maintenance from changes to the YAML
language. The Rust library remains optional: a conforming document need
not be authored by Rust software.
