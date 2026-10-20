# Execarium schema 4: named jobs

Schema 4 retains the command, environment, and sequential execution semantics of schema 3 but requires an explicit human-readable name. All earlier schemas remain valid and unchanged.

~~~yaml
schema: 4
name: Verify Rust project
shell: fish
cwd: /home/user/projects/example
privilege: none
timeout_seconds: 600
environment:
  session: inherit
  gpu: inherit
commands:
  - cargo fmt --check
  - cargo test
~~~

The name is a required YAML string, 1 to 120 UTF-8 bytes, trimmed, and free of control characters. It must not be used as executable source. Other fields follow schema 3, including the requirement for exactly one of command or commands, and the mandatory session and GPU environment map. Unknown or duplicate keys fail validation.

A browser tab identifier is **not** part of the YAML contract. Clients send origin metadata separately with Jobflick submissions. Jobflick generates visual colors itself; neither the author nor this schema assigns a job color.
