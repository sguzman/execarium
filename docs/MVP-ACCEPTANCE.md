# Execarium MVP acceptance gate

**Scope:** Released YAML **schema 1** and the current Rust reference
implementation. This gate occurs **before** declaring the Rust library
sufficiently tested for routine consumption by Jobflick.

Execarium is a specification, constructor, serializer, and validator; it is
**not** a command executor. Consequently, these tests do not demonstrate
Jobflick execution, privileged authentication, sandboxing, clipboard policies,
or process timeout enforcement. Those require independent consumer tests.

## Requirements for MVP

All of the following must be true on the *same main commit*:

1. Rust formatting and Clippy (warnings denied) pass.
2. All pre-existing unit, documentation, and schema-1 tests pass.
3. Valid and invalid corpus files match their expected outcomes for both
   the Rust API and the real `execarium validate` CLI.
4. Construction rejects illegal directories, commands, and timeouts with the
   same field constraints as source parsing.
5. Constructed values render to YAML and reparse identically, including
   shell punctuation, ambiguous scalars, whitespace and Unicode.
6. The parser accepts every documented v1 enum/timeout combination and
   validates boundaries as byte counts, not character counts.
7. Fixed-seed randomized round-trip and malformed-input stress suites pass.
8. The real CLI rejects missing/unknown arguments, bad UTF-8, unsupported
   versions and oversized stdin/files; it never invokes shell commands.
9. The YAML contract remains frozen: **schema: 1**, exactly six required
   fields, no compatibility fallback, new fields, or execution backends.
10. GitHub Actions passes and a distinct immutable release tag points to the
    tested commit (not an unverified future commit).

## Test inventory

| Test source | Target |
| --- | --- |
| `tests/conformance.rs` | Original normative v1 conformance cases |
| `tests/versioned_api.rs` | v1 validator dispatch, typed builder, YAML rendering |
| `tests/mvp_acceptance.rs` | Boundary matrices, schema mutation, deterministic stress |
| `tests/cli_acceptance.rs` | Actual binary, stdin/files, exit codes and no-execution |
| `tests/conformance_corpus.rs` | Portable valid/invalid YAML files |
| `tests/fixtures/v1/` | 6 valid and 14 rejected standalone YAML documents |
| `cargo test --doc` | Public Rust API documentation examples |
| `cargo clippy --all-targets -- -D warnings` | Static lint baseline |
| `.github/workflows/ci.yml` | Automated complete acceptance and corpus CLI checks |

The randomized suite uses a **fixed seed** so a failure is reproducible.
It includes at least 2,000 generated valid-job round trips and 3,000
untrusted malformed-input parsing attempts, plus all 54 shell/privilege/
representative-timeout combinations. These tests supplement the
document corpus, but are **not** a proof of memory safety or an exhaustive
fuzzer. Production fuzzing and dependency auditing can be added separately.

## Commands

```sh
cargo fmt --all -- --check
cargo test --all-targets
cargo test --doc
cargo clippy --all-targets -- -D warnings
cargo build --quiet
for yaml in tests/fixtures/v1/valid/*.yaml; do
    ./target/debug/execarium validate "$yaml"
done
for yaml in tests/fixtures/v1/invalid/*.yaml; do
    if ./target/debug/execarium validate "$yaml"; then
        echo "Invalid fixture accepted: $yaml" >&2
        exit 1
    fi
done
```

## Post-acceptance scope

A successful gate permits formalizing **Execarium as MVP**. The stable
wire schema will still be `schema: 1`, and Rust library improvements only
require a **patch** release. Jobflick remains an independent project: it
must deliberately integrate and test the parser, queue, authorization holds,
per-job launch, timeout enforcement, and copy/report behavior.

The full status and exact CI run must be recorded only **after** results
have been verified. Do not mark a gate green merely because files exist.
