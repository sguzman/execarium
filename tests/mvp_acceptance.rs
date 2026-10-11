//! Acceptance suite: schema-1 MUST remain stable across Rust library releases.
//! The pseudo-random cases use fixed seeds so CI failures are reproducible.
use execarium::{
    construct, parse, parse_for_schema, v1, validate_for_schema, Error, Privilege, Shell,
    MAX_COMMAND_BYTES, MAX_CWD_BYTES, MAX_DOCUMENT_BYTES, MAX_TIMEOUT_SECONDS,
    MIN_TIMEOUT_SECONDS, SCHEMA_VERSION, SUPPORTED_SCHEMAS,
};

const BASE: &str = "schema: 1\nshell: fish\ncwd: /tmp\nprivilege: none\ntimeout_seconds: 600\ncommand: echo hello\n";

fn modified(key: &str, value: &str) -> String {
    let mut out = String::new();
    let mut found = 0;
    for line in BASE.lines() {
        if line.starts_with(&format!("{key}:")) {
            out.push_str(key);
            out.push_str(": ");
            out.push_str(value);
            out.push('\n');
            found += 1;
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    assert_eq!(found, 1, "unknown field {key}");
    out
}

fn reject_document(input: &str) {
    let result = v1::Validator::parse(input);
    assert!(result.is_err(), "accepted invalid document: {input:?}");
    assert!(v1::Validator::validate(input).is_err());
    assert!(validate_for_schema(1, input).is_err());
}

#[test]
fn v1_released_examples_remain_exactly_compatible() {
    assert_eq!(SUPPORTED_SCHEMAS, &[1]);
    assert_eq!(SCHEMA_VERSION, v1::SCHEMA);
    for document in [
        BASE,
        include_str!("../examples/local-fish.yaml"),
        include_str!("../examples/multiline-fish.yaml"),
        include_str!("../examples/privileged-doas.yaml"),
    ] {
        let a = v1::Validator::parse(document).unwrap();
        assert_eq!(a, parse_for_schema(1, document).unwrap());
        assert_eq!(a, parse(document).unwrap());
        assert_eq!(a.schema(), 1);
        let yaml = a.to_yaml().unwrap();
        assert_eq!(v1::Validator::parse(&yaml).unwrap(), a);
    }
}

#[test]
fn exhaustive_shell_privilege_and_timeout_boundary_matrix() {
    let mut cases = 0;
    for shell in [Shell::Fish, Shell::Bash, Shell::Sh] {
        for privilege in [Privilege::None, Privilege::Sudo, Privilege::Doas] {
            for timeout in [MIN_TIMEOUT_SECONDS, 2, 60, 600, 3600, MAX_TIMEOUT_SECONDS] {
                let constructed = construct(
                    shell,
                    "/workspace",
                    privilege,
                    timeout,
                    "echo carefully",
                )
                .unwrap();
                let yaml = v1::Validator::render(&constructed).unwrap();
                let parsed = v1::Validator::parse(&yaml).unwrap();
                assert_eq!(parsed, constructed);
                assert_eq!(parsed.shell(), shell);
                assert_eq!(parsed.privilege(), privilege);
                assert_eq!(parsed.timeout_seconds(), timeout);
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 54);
}

#[test]
fn all_six_fields_are_mandatory_and_must_not_repeat() {
    for key in ["schema", "shell", "cwd", "privilege", "timeout_seconds", "command"] {
        let missing = BASE
            .lines()
            .filter(|line| !line.starts_with(&format!("{key}:")))
            .collect::<Vec<_>>()
            .join("\n");
        reject_document(&missing);
        let repeated = BASE
            .lines()
            .find(|line| line.starts_with(&format!("{key}:")))
            .unwrap();
        reject_document(&format!("{BASE}{repeated}\n"));
    }
}

#[test]
fn closed_field_set_cannot_grow_accidentally() {
    for field in [
        "executor: local",
        "commands: [echo, hello]",
        "env: {}",
        "description: test",
        "trigger: clipboard",
        "schema_version: 1",
        "timeout: 10",
        "sudo: true",
        "resources: {}",
    ] {
        reject_document(&format!("{BASE}{field}\n"));
    }
}

#[test]
fn strict_type_rejection_covers_all_fields() {
    for value in ["\"1\"", "1.0", "true", "null", "-1", "2", "[]", "{}"] {
        reject_document(&modified("schema", value));
    }
    for value in ["fishy", "zsh", "FISH", "null", "true", "99", "[]", "{}"] {
        reject_document(&modified("shell", value));
    }
    for value in ["nonee", "root", "DOAS", "null", "true", "10", "[]", "{}"] {
        reject_document(&modified("privilege", value));
    }
    for value in ["null", "true", "0", "[]", "{}", "\"\"", "\" \""] {
        reject_document(&modified("cwd", value));
    }
    for value in ["null", "true", "12", "[]", "{}", "\"\"", "\"   \""] {
        reject_document(&modified("command", value));
    }
    for value in ["null", "true", "-1", "0", "604801", "1.5", "\"600\"", "[]", "{}"] {
        reject_document(&modified("timeout_seconds", value));
    }
}

#[test]
fn exact_timeout_boundaries_and_no_infinite_sentinel() {
    for timeout in [0, MAX_TIMEOUT_SECONDS + 1, u64::MAX] {
        assert!(matches!(
            v1::construct(Shell::Fish, "/tmp", Privilege::None, timeout, "true"),
            Err(Error::InvalidTimeout(_))
        ));
    }
    for timeout in [MIN_TIMEOUT_SECONDS, MAX_TIMEOUT_SECONDS] {
        assert_eq!(
            v1::construct(Shell::Fish, "/tmp", Privilege::None, timeout, "true")
                .unwrap()
                .timeout_seconds(),
            timeout
        );
    }
}

#[test]
fn working_directory_lexical_boundaries() {
    for cwd in [
        "relative",
        "~",
        "~/work",
        "$HOME/work",
        "",
        "/tmp/.",
        "/tmp/..",
        "/tmp/../private",
        "/./tmp",
        "/../tmp",
        "/tmp/\nother",
        "/tmp/\rfoo",
        "/tmp/\0hi",
    ] {
        assert!(
            matches!(
                v1::construct(Shell::Fish, cwd, Privilege::None, 600, "echo ok"),
                Err(Error::InvalidCwd(_))
            ),
            "accepted {cwd:?}"
        );
    }
    for cwd in ["/", "/work", "//tmp", "/working directory", "/日本語/🙂"] {
        let job = v1::construct(Shell::Fish, cwd, Privilege::None, 600, "echo ok").unwrap();
        let decoded = v1::parse(&job.to_yaml().unwrap()).unwrap();
        assert_eq!(decoded.cwd(), cwd);
    }
    let at_limit = format!("/{}", "x".repeat(MAX_CWD_BYTES - 1));
    assert_eq!(at_limit.len(), MAX_CWD_BYTES);
    assert!(v1::construct(Shell::Sh, at_limit, Privilege::None, 1, "true").is_ok());
    let over_limit = format!("/{}", "x".repeat(MAX_CWD_BYTES));
    assert!(matches!(
        v1::construct(Shell::Sh, over_limit, Privilege::None, 1, "true"),
        Err(Error::InvalidCwd(_))
    ));
}

#[test]
fn command_byte_limits_are_not_character_limits() {
    let max_ascii = "x".repeat(MAX_COMMAND_BYTES);
    let max_unicode = "🙂".repeat(MAX_COMMAND_BYTES / 4);
    for source in [max_ascii.as_str(), max_unicode.as_str()] {
        assert_eq!(source.len(), MAX_COMMAND_BYTES);
        let job = v1::construct(Shell::Fish, "/", Privilege::None, 60, source).unwrap();
        assert_eq!(job.command(), source);
        assert_eq!(v1::parse(&job.to_yaml().unwrap()).unwrap(), job);
    }
    assert!(v1::construct(Shell::Fish, "/", Privilege::None, 60, format!("{max_ascii}x")).is_err());
    assert!(v1::construct(Shell::Fish, "/", Privilege::None, 60, format!("{max_unicode}x")).is_err());
    for bad in ["", " ", "\n  \r", "\0", "safe\0unsafe"] {
        assert!(matches!(
            v1::construct(Shell::Sh, "/", Privilege::None, 600, bad),
            Err(Error::InvalidCommand(_))
        ));
    }
}

#[test]
fn complete_input_byte_limit_uses_exact_threshold() {
    let comment_prefix = format!("{BASE}#");
    let valid = format!(
        "{comment_prefix}{}",
        "a".repeat(MAX_DOCUMENT_BYTES - comment_prefix.len())
    );
    assert_eq!(valid.len(), MAX_DOCUMENT_BYTES);
    assert!(v1::parse(&valid).is_ok());
    assert!(matches!(
        v1::parse(&(valid + "x")),
        Err(Error::DocumentTooLarge)
    ));
}

#[test]
fn yaml_indirection_and_multi_document_streams_are_rejected() {
    for invalid in [
        "---\nschema: 1\n---\nschema: 1\n",
        "schema: 1\n...\n---\nschema: 1\n",
        "schema: &version 1\nshell: fish\ncwd: /tmp\nprivilege: none\ntimeout_seconds: 60\ncommand: echo ok\n",
        "schema: 1\nshell: *shell\ncwd: /tmp\nprivilege: none\ntimeout_seconds: 60\ncommand: echo ok\n",
        "schema: 1\nshell: !fish fish\ncwd: /tmp\nprivilege: none\ntimeout_seconds: 60\ncommand: echo ok\n",
        "%TAG !e! tag:example.com,2026:\n---\nschema: 1\n",
        "%YAML 1.2\n---\nschema: 1\n",
        "{schema: 1, shell: fish, cwd: /tmp, privilege: none, timeout_seconds: 60, command: a, command: b}",
        "- [1, 2, 3]",
        "null",
        "",
    ] {
        reject_document(invalid);
    }
}

#[test]
fn line_endings_unicode_and_quoted_yaml_metacharacters() {
    let line_endings = BASE.replace('\n', "\r\n");
    let decoded = v1::parse(&line_endings).unwrap();
    assert_eq!(decoded.command(), "echo hello");
    for command in [
        "null",
        "true",
        "false",
        "1234",
        "0x10",
        "#comment",
        "*alias",
        "&anchor",
        "!tag",
        "---",
        "hello: world",
        "echo '* & ! : #'",
        "printf '\"quoted\"'",
        "echo 🐈‍⬛ مرحبا 世界",
        "first\nsecond\n",
        "first\r\nsecond",
        "a\\b\\c",
        "a\tb",
    ] {
        let job = v1::construct(Shell::Bash, "/tmp", Privilege::None, 60, command).unwrap();
        let round_trip = v1::parse(&job.to_yaml().unwrap()).unwrap();
        assert_eq!(round_trip.command(), command, "{command:?}");
    }
}

#[test]
fn independent_version_selection_never_falls_back() {
    for chosen in [0, 2, 3, 100, u64::MAX] {
        assert!(matches!(
            parse_for_schema(chosen, BASE),
            Err(Error::UnsupportedSchema(v)) if v == chosen
        ));
    }
    let mismatched = modified("schema", "2");
    assert!(matches!(parse_for_schema(1, &mismatched), Err(Error::UnsupportedSchema(2))));
    assert_eq!(parse_for_schema(1, BASE).unwrap(), parse(BASE).unwrap());
}

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn index(&mut self, len: usize) -> usize {
        (self.next() as usize) % len
    }
}

#[test]
fn seeded_randomized_round_trips_across_thousands_of_jobs() {
    let mut rng = Rng(0x5EED_1234_907A_BEEF);
    let shells = [Shell::Fish, Shell::Bash, Shell::Sh];
    let privileges = [Privilege::None, Privilege::Sudo, Privilege::Doas];
    let snippets = [
        "echo hello",
        "null",
        "true",
        "#literal",
        "echo 'quoted'",
        "echo one\nand echo two",
        "printf '%s' 'x: y'",
        "!symbol &symbol *symbol",
        "printf 🐈",
        "a\\b\tc",
        "echo \"ok\"",
    ];
    for case in 0..2000 {
        let cmd = format!("{}\n# seed {case}", snippets[rng.index(snippets.len())]);
        let dir = format!("/work_{}/📁", rng.index(1000));
        let secs = (rng.next() % MAX_TIMEOUT_SECONDS) + 1;
        let job = construct(
            shells[rng.index(shells.len())],
            dir,
            privileges[rng.index(privileges.len())],
            secs,
            cmd,
        )
        .unwrap();
        let encoded = job.to_yaml().unwrap();
        let decoded = v1::Validator::parse(&encoded).unwrap();
        assert_eq!(decoded, job, "case {case}");
        assert_eq!(decoded.to_yaml().unwrap(), encoded, "case {case}");
    }
}

#[test]
fn randomized_untrusted_inputs_never_panic() {
    let mut rng = Rng(0xBAD5_EED1_2345_6789);
    let alphabet: &[u8] = b" abcdefghijklmnopqrstuvwxyz0123456789{}[]-:!&*#'\"\\\n\r\t\0%?.";
    for case in 0..3000 {
        let length = rng.index(128);
        let input: String = (0..length)
            .map(|_| alphabet[rng.index(alphabet.len())] as char)
            .collect();
        let first = std::panic::catch_unwind(|| v1::Validator::parse(&input));
        assert!(first.is_ok(), "parser panicked in case {case}: {input:?}");
        let second = std::panic::catch_unwind(|| v1::Validator::validate(&input));
        assert!(second.is_ok(), "validator panicked in case {case}: {input:?}");
    }
}
