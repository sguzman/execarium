# Scope and roadmap

## Active work: standalone schema v1

The immediate goal is a small, independently reviewable specification that
multiple projects can consume. Work here, not in Jobflick, until the v1
contract is settled.

* [x] Repository bootstrap and independent project identity
* [x] Document `schema: 1` with all five required fields
* [x] Rust parsing/validation library, no process execution
* [x] Validation-only CLI
* [x] Strictness, security and conformance documentation
* [x] Positive/negative parser tests and CI (subject to CI verification)
* [ ] Review v1 contract and settle unresolved policy questions
* [ ] Declare v1 stable and tag a first release after verification
* [ ] Resume Jobflick and integrate structured submission **explicitly**

## Settled v1 decisions

* Format is project-neutral: `schema: 1`, not `jobflick: 1`.
* Only local shell job description; no `executor` key.
* Five required fields: `schema`, `shell`, `cwd`, `privilege`, `command`.
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
* Dependencies, DAGs, retries, priorities, cron, resources or quotas in YAML.
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

Changes to schema 1's meaning require deliberate review. Incompatible or
additional semantics should become a new schema version rather than altering
the interpretation of existing documents.
