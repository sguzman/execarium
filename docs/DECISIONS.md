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

**Decision:** All five keys mandatory, exact `schema: 1`, no unknown fields,
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
