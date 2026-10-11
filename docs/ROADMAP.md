# Scope and roadmap

## Status: MVP — stable schema 1, Rust crate v1.1.1

The standalone six-field specification and Rust reference parser are stable
as v1.0.0. Jobflick is a separate project; integration remains a deliberate,
separately authorized next step.

* [x] Repository bootstrap and independent project identity
* [x] Document stable `schema: 1` with all six required fields
* [x] Rust parsing/validation library, no process execution
* [x] Validation-only CLI
* [x] Strictness, security and conformance documentation
* [x] Positive/negative parser tests and CI
* [x] Accept v1.0.0 as the stable six-field baseline
* [x] Align crate 1.0.0 with schema 1
* [x] Add version-specific `v1::Validator`, typed construction, rendering and
  shared validation in compatible crate 1.1.0
* [x] Add versioning policy, release history and AI authoring instructions
* [x] Configure CI to publish immutable crate tags after verification
* [x] Add 20 portable YAML fixtures and robust CLI validation acceptance tests
* [x] Add 2,000 deterministic YAML round trips and 3,000 malformed-input
  parser attempts, plus 54 enum/timeout combinations
* [x] Run and pass the complete acceptance gate before promoting MVP
* [x] Mark independent Execarium specification/library as MVP (crate 1.1.1)
* [ ] Resume Jobflick and integrate structured submission **explicitly**

## Settled v1 decisions

* Format is project-neutral: `schema: 1`, not `jobflick: 1`.
* Only local shell job description; no `executor` key.
* Six required fields: `schema`, `shell`, `cwd`, `privilege`,
  `timeout_seconds`, `command`.
* `timeout_seconds`: exact integer 1–604800, measured from execution start;
  each consumer enforces its runtime limit, not the parser.
* Shell enum includes Fish, Bash and POSIX sh as descriptions; not all
  consumers are required to implement every dialect.
* Privilege enum is `none`/`sudo`/`doas`; none grants authority.
* No implicit version migrations, unknown-field tolerance, or defaults.
* Execarium validates data; consumers own scheduling and OS interaction.
* Jobflick remains paused while the shared contract matures.

## Explicitly deferred — not to be implemented opportunistically

* Automatic clipboard reading, monitoring, scanning, auto-enqueue or triggers.
* Docker, Podman, jails, container orchestration.
* Firecracker, microVMs, hypervisors or VMs.
* Remote machines, SSH, GitHub Actions, GitLab CI, cloud runners.
* Dependencies, DAGs, retries, priorities, cron, additional resources
  (CPU/memory) or quotas in YAML.
* Credential stores, password prompts, approval HUDs, privilege escalation.
* Shell execution of any kind inside the Execarium library or CLI.
* Altering the Jobflick repository, systemd, Hyprland, or user workstation.

The blocked-authorization workflow discussed for Jobflick is a **future
consumer feature**, not part of schema 1 or this parser. A consumer may hold a
privileged document until the user approves it and supplies credentials via a
trusted OS channel.

## Open review questions (do not silently resolve via new fields)

* Should the supported shell enum remain `fish`/`bash`/`sh` long-term?
* Is the mandatory `cwd` appropriate for every future producer?
* Should schema version allowlists remain entirely consumer-side?
* Should path syntax tighten in a future version (e.g., repeated separators)?
* Does a future v2 need separate command arguments or environment variables?

The timeout addition occurred in the **unreleased prototype**, and the
earlier five-field draft is intentionally rejected. Version 1.0.0 freezes
the six-field contract. Future incompatible changes require a new schema
integer and matching Rust crate major version. See [VERSIONING.md](VERSIONING.md).
