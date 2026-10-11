use execarium::{
    parse, validate, Error, Privilege, Shell, MAX_COMMAND_BYTES, MAX_CWD_BYTES,
    MAX_DOCUMENT_BYTES, SCHEMA_VERSION,
};

const MINIMAL: &str = "schema: 1\nshell: fish\ncwd: /tmp\nprivilege: none\ncommand: echo ok\n";

fn reject(yaml: &str) {
    assert!(parse(yaml).is_err(), "unexpected acceptance of: {yaml:?}");
}

#[test]
fn accepts_all_example_documents_without_executing_anything() {
    let local = parse(include_str!("../examples/local-fish.yaml")).unwrap();
    assert_eq!(local.schema(), SCHEMA_VERSION);
    assert_eq!(local.shell(), Shell::Fish);
    assert_eq!(local.cwd(), "/home/user/Projects/example");
    assert_eq!(local.privilege(), Privilege::None);
    assert_eq!(local.command(), "cargo test");

    let multi = parse(include_str!("../examples/multiline-fish.yaml")).unwrap();
    assert_eq!(multi.command(), "cargo fmt --check\nand cargo test\nand cargo build\n");

    let elevated = parse(include_str!("../examples/privileged-doas.yaml")).unwrap();
    assert_eq!(elevated.privilege(), Privilege::Doas);
    assert_eq!(elevated.command(), "pacman -Syu");
    assert!(validate(MINIMAL).is_ok());
}

#[test]
fn valid_enumerations_are_exact_and_case_sensitive() {
    for shell in ["fish", "bash", "sh"] {
        let yaml = MINIMAL.replace("shell: fish", &format!("shell: {shell}"));
        assert_eq!(parse(&yaml).unwrap().shell().as_str(), shell);
    }
    for privilege in ["none", "sudo", "doas"] {
        let yaml = MINIMAL.replace("privilege: none", &format!("privilege: {privilege}"));
        assert_eq!(parse(&yaml).unwrap().privilege().as_str(), privilege);
    }
    reject(&MINIMAL.replace("shell: fish", "shell: Fish"));
    reject(&MINIMAL.replace("shell: fish", "shell: zsh"));
    reject(&MINIMAL.replace("privilege: none", "privilege: root"));
}

#[test]
fn versions_must_be_explicit_exact_integer_one() {
    for value in ["0", "2", "1.0", "\"1\"", "true", "null"] {
        reject(&MINIMAL.replace("schema: 1", &format!("schema: {value}")));
    }
    let failure = parse(&MINIMAL.replace("schema: 1", "schema: 2")).unwrap_err();
    assert!(matches!(failure, Error::UnsupportedSchema(2)));
    reject(&MINIMAL.replace("schema: 1\n", ""));
}

#[test]
fn missing_duplicate_and_unknown_keys_fail_closed() {
    for key in ["schema", "shell", "cwd", "privilege", "command"] {
        let filtered = MINIMAL
            .lines()
            .filter(|line| !line.starts_with(&format!("{key}:")))
            .collect::<Vec<_>>()
            .join("\n");
        reject(&filtered);
    }
    reject(&format!("{MINIMAL}shell: bash\n"));
    reject(&format!("{MINIMAL}command: echo second\n"));
    reject(&format!("{MINIMAL}executor: local\n"));
    reject(&format!("{MINIMAL}run_as: root\n"));
}

#[test]
fn top_level_must_be_one_mapping_with_no_yaml_indirection() {
    reject("- schema: 1\n- shell: fish\n");
    reject("foo\n");
    reject(&format!("{MINIMAL}---\nschema: 1\n"));
    reject("schema: 1\nshell: &s fish\ncwd: /tmp\nprivilege: none\ncommand: *s\n");
    reject("schema: 1\nshell: !custom fish\ncwd: /tmp\nprivilege: none\ncommand: echo ok\n");
    reject("%YAML 1.2\n---\nschema: 1\nshell: fish\ncwd: /tmp\nprivilege: none\ncommand: echo ok\n");
    reject("schema: [1]\nshell: fish\ncwd: /tmp\nprivilege: none\ncommand: echo ok\n");
    reject("schema: 1\nshell: fish\ncwd: /tmp\nprivilege: none\ncommand: [echo, ok]\n");
}

#[test]
fn shell_punctuation_is_not_mistaken_for_yaml_tags_or_anchors() {
    let yaml = MINIMAL.replace("command: echo ok", "command: \"printf '%s' '* & !'\"");
    let parsed = parse(&yaml).unwrap();
    assert_eq!(parsed.command(), "printf '%s' '* & !'");
    let with_comment = format!("---\n# ordinary YAML comment\n{MINIMAL}");
    assert!(parse(&with_comment).is_ok());
}

#[test]
fn cwd_is_a_literal_absolute_posix_path_not_a_resolution_request() {
    for bad in [
        "relative/project",
        "~/project",
        "/tmp/../secret",
        "/tmp/./test",
        "/tmp\nother",
        "",
    ] {
        let yaml = MINIMAL.replace("cwd: /tmp", &format!("cwd: \"{bad}\""));
        reject(&yaml);
    }
    let root = parse(&MINIMAL.replace("cwd: /tmp", "cwd: /")).unwrap();
    assert_eq!(root.cwd(), "/");
    let long = format!("/{}", "x".repeat(MAX_CWD_BYTES));
    reject(&MINIMAL.replace("cwd: /tmp", &format!("cwd: {long}")));
}

#[test]
fn command_is_literal_nonempty_bounded_text() {
    reject(&MINIMAL.replace("command: echo ok", "command: \"\""));
    reject(&MINIMAL.replace("command: echo ok", "command: \"   \""));
    reject(&MINIMAL.replace("command: echo ok", "command: null"));
    reject(&MINIMAL.replace("command: echo ok", "command: 99"));
    let nul_command = MINIMAL.replace("command: echo ok", "command: \"a\\0b\"");
    reject(&nul_command);
    let at_limit = "x".repeat(MAX_COMMAND_BYTES);
    let yaml = MINIMAL.replace("command: echo ok", &format!("command: {at_limit}"));
    assert_eq!(parse(&yaml).unwrap().command().len(), MAX_COMMAND_BYTES);
    let oversized = MINIMAL.replace("command: echo ok", &format!("command: {at_limit}x"));
    reject(&oversized);
}

#[test]
fn complete_documents_have_independent_bounded_input_size() {
    let oversized = format!("{MINIMAL}#{}", "x".repeat(MAX_DOCUMENT_BYTES));
    assert!(matches!(parse(&oversized), Err(Error::DocumentTooLarge)));
}

#[test]
fn parser_never_performs_filesystem_validation_or_execution() {
    let yaml = MINIMAL.replace("cwd: /tmp", "cwd: /this/need/not/exist");
    let job = parse(&yaml).unwrap();
    assert_eq!(job.cwd(), "/this/need/not/exist");
    assert_eq!(job.command(), "echo ok");
    // This test deliberately performs no file/process operations: parse is
    // a data operation, not consent to execute this request.
}
