# Execarium v1 conformance and testing

## Accepted documents

The reference parser MUST accept:
* `examples/local-fish.yaml`: minimal single-command local Fish job.
* `examples/multiline-fish.yaml`: literal multi-line Fish command.
* `examples/privileged-doas.yaml`: syntactically valid metadata that still
  **requires separate user authorization before any execution**.
* In-memory documents using Bash or POSIX sh enum values.
* Quoted literal shell operators `!`, `&`, and `*` (not YAML tags/aliases).
* Comments, arbitrary key order, a single optional `---`, and Unicode text.
* Integer `timeout_seconds` boundaries 1 and 604800, and intermediate values.

## Rejected documents

| Condition | Required result |
| --- | --- |
| Missing any required field | Error |
| Duplicate key | Error, never last-write-wins |
| Unknown or extra key (including `executor`) | Error |
| `schema: 2`, `schema: 0` | Unsupported schema |
| Quoted `schema: "1"`, null or boolean schema | Type error |
| Unknown shell or privilege mode | Error |
| Missing or duplicate `timeout_seconds` | Error |
| `timeout_seconds: 0`, `604801`, negative integers | Error |
| Quoted, fractional, boolean, null, list or mapping timeout | Error |
| Nonmapping document or a YAML sequence | Error |
| Multiple documents separated by `---` | Error |
| YAML anchor, alias, tag or directive | Error |
| Relative or tilde-expanded cwd | Error |
| Dot or parent-directory segment in cwd | Error |
| NUL/newline in cwd | Error |
| Empty or whitespace-only command | Error |
| NUL in command | Error |
| Command over 128 KiB | Error |
| Document over 256 KiB | Error |
| Malformed YAML | Error |

Rejected documents MUST NOT launch or enqueue a job (the reference library
is incapable of either). A validation-only CLI MUST exit nonzero.

## Running checks

```sh
cargo fmt --all -- --check
cargo test --all-targets
cargo test --doc
cargo clippy --all-targets -- -D warnings
cargo run -- validate examples/local-fish.yaml
cargo run -- validate examples/multiline-fish.yaml
cargo run -- validate examples/privileged-doas.yaml
```

CLI smoke tests in `.github/workflows/ci.yml` assert the expected acceptance
and rejection behavior for file input and stdin. The test suite exercises each
field, key strictness, YAML indirection, timeout integer boundaries, and byte limits.

## Test scope and limitations

Passing these tests means **the parser conforms to its documented v1 rules on
tested cases**. It does not prove a job safe, confer authority, or test
the behavior of any eventual command runner. No actual shell execution is
part of these tests.
