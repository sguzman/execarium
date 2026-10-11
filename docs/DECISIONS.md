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

## D9 — Historical decision: initial alignment of v1.0.0 and schema 1

**Historical decision (superseded by D10):** Declare the current six-field
YAML contract as stable schema 1 and the existing Rust library as crate
v1.0.0. At the time, we also required equal crate and schema major numbers.
The alignment was convenient at bootstrap, but this restriction was later
withdrawn in D10.

**Reason:** Gives human, ChatGPT, and programmatic consumers a clear stable
reference while distinguishing library maintenance from changes to the YAML
language. The Rust library remains optional: a conforming document need
not be authored by Rust software.

## D10 — Independent specification and reference-library versions

**Decision:** The YAML specification uses its own schema integer; the Rust
library uses independent Semantic Versioning. Neither is calculated from the
other. The reference library declares its exact implemented schema versions
via `SUPPORTED_SCHEMAS` and explicit per-version validators.

**Reason:** Library API compatibility and wire-format compatibility have
different boundaries. A breaking Rust API redesign may leave existing YAML
valid, while additive Rust support for a newer YAML schema need not break
existing library clients. Consumers must explicitly choose allowed schemas.

**Migration:** D10 supersedes D9's major-number equality constraint.
The six-field stable YAML schema 1, existing version-1 validation semantics,
and previously tagged Rust releases remain unchanged. Crate 1.1.2 updates
policy, documentation and regression tests only.
