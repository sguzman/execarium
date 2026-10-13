use execarium::{
    construct, parse, parse_for_schema, v1, validate_for_schema, Error, Privilege, Shell,
    IMPLEMENTATION_VERSION, SCHEMA_VERSION, SUPPORTED_SCHEMAS,
};

#[test]
fn legacy_parser_remains_schema_one_only() {
    assert_eq!(SUPPORTED_SCHEMAS, &[1, 2, 3]);
    assert_eq!(v1::SCHEMA, SCHEMA_VERSION);
    assert_eq!(IMPLEMENTATION_VERSION, env!("CARGO_PKG_VERSION"));
    let input = include_str!("../examples/local-fish.yaml");
    assert_eq!(parse_for_schema(1, input).unwrap(), parse(input).unwrap());
    v1::Validator::validate(input).unwrap();
    v1::validate(input).unwrap();
    validate_for_schema(1, input).unwrap();
    assert!(matches!(
        parse_for_schema(2, input),
        Err(Error::UnsupportedSchema(2))
    ));
    assert!(matches!(
        validate_for_schema(0, input),
        Err(Error::UnsupportedSchema(0))
    ));
    let mismatched = input.replace("schema: 1", "schema: 2");
    assert!(matches!(
        parse_for_schema(1, &mismatched),
        Err(Error::UnsupportedSchema(2))
    ));
}

#[test]
fn typed_construction_uses_the_same_v1_field_constraints() {
    let job = v1::construct(Shell::Fish, "/tmp", Privilege::None, 600, "echo ok").unwrap();
    let through_validator =
        v1::Validator::construct(Shell::Fish, "/tmp", Privilege::None, 600, "echo ok").unwrap();
    let through_top_level =
        construct(Shell::Fish, "/tmp", Privilege::None, 600, "echo ok").unwrap();
    assert_eq!(job, through_validator);
    assert_eq!(job, through_top_level);
    assert_eq!(job.schema(), 1);

    for cwd in ["", "relative", "/tmp/../escape", "/tmp/.", "/line\nbreak"] {
        assert!(v1::construct(Shell::Fish, cwd, Privilege::None, 600, "echo ok").is_err());
    }
    for timeout in [0, 604801, u64::MAX] {
        assert!(matches!(
            v1::construct(Shell::Fish, "/", Privilege::None, timeout, "echo ok"),
            Err(Error::InvalidTimeout(_))
        ));
    }
    for command in ["", "   ", "\0", "x\0y"] {
        assert!(matches!(
            v1::construct(Shell::Bash, "/", Privilege::Sudo, 20, command),
            Err(Error::InvalidCommand(_))
        ));
    }
    assert!(v1::construct(Shell::Fish, "/", Privilege::None, 1, "x".repeat(131073)).is_err());
    assert!(v1::construct(
        Shell::Sh,
        "/".to_owned(),
        Privilege::Doas,
        604800,
        "printf ok"
    )
    .is_ok());
}

#[test]
fn typed_round_trip_renders_documents_with_the_same_exact_data() {
    let examples = [
        ("echo hi", "/tmp", Shell::Fish, Privilege::None, 600),
        ("null", "/tmp", Shell::Bash, Privilege::Doas, 1),
        ("true", "/tmp", Shell::Sh, Privilege::Sudo, 604800),
        (
            "echo '* & !' && printf '%s' 'hello: world'",
            "/a folder",
            Shell::Fish,
            Privilege::None,
            16,
        ),
        (
            "echo one\nand echo two\n",
            "/",
            Shell::Fish,
            Privilege::None,
            30,
        ),
        (
            "printf 'colon: yes'\n",
            "/tmp",
            Shell::Sh,
            Privilege::None,
            30,
        ),
    ];

    for (command, cwd, shell, privilege, timeout) in examples {
        let job = v1::Validator::construct(shell, cwd, privilege, timeout, command).unwrap();
        let rendered = v1::Validator::render(&job).unwrap();
        assert!(rendered.contains("schema: 1"));
        assert!(rendered.contains("timeout_seconds:"));
        assert!(!rendered.contains("executor:"));
        v1::Validator::validate(&rendered).unwrap();
        assert_eq!(v1::Validator::parse(&rendered).unwrap(), job);
        assert_eq!(parse_for_schema(1, &rendered).unwrap(), job);
        assert_eq!(job.to_yaml().unwrap(), rendered);
    }
}

#[test]
fn released_schema_one_files_still_parse_with_the_versioned_validator() {
    for fixture in [
        include_str!("../examples/local-fish.yaml"),
        include_str!("../examples/multiline-fish.yaml"),
        include_str!("../examples/privileged-doas.yaml"),
    ] {
        let parsed = v1::parse(fixture).unwrap();
        let rendered = parsed.to_yaml().unwrap();
        assert_eq!(v1::parse(&rendered).unwrap(), parsed);
    }
}

#[test]
fn building_a_document_does_not_execute_shell_source() {
    let sentinel = format!("/tmp/execarium-library-must-not-run-{}", std::process::id());
    // Merely expressing malicious-looking shell source does not run it.
    let job = v1::construct(
        Shell::Fish,
        "/tmp",
        Privilege::None,
        30,
        format!("touch '{}'", sentinel),
    )
    .unwrap();
    let rendered = job.to_yaml().unwrap();
    assert!(rendered.contains("touch"));
    assert!(!std::path::Path::new(&sentinel).exists());
}
