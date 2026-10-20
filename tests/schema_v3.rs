use execarium::{
    parse_document, parse_document_for_schema, v1, v2, v3, validate_for_schema, Document, Error,
    Privilege, Shell, MAX_COMMAND_BYTES, MAX_DOCUMENT_BYTES,
};

const SCRIPT: &str = include_str!("../examples/hyprland-fish-v3.yaml");
const STEPS: &str = include_str!("../examples/steps-wayland-v3.yaml");

#[test]
fn examples_are_exact_version_three_and_preserve_source_shape() {
    let script = v3::parse(SCRIPT).unwrap();
    assert_eq!(script.schema(), 3);
    assert_eq!(script.environment().session(), v3::Session::Hyprland);
    assert_eq!(script.environment().gpu(), v3::Gpu::Nvidia);
    assert!(script.command().unwrap().contains("hyprctl activewindow"));
    assert!(script.commands().is_none());
    assert!(matches!(parse_document(SCRIPT).unwrap(), Document::V3(_)));
    let steps = v3::parse(STEPS).unwrap();
    assert_eq!(steps.environment().session(), v3::Session::Wayland);
    assert_eq!(steps.environment().gpu(), v3::Gpu::Integrated);
    assert_eq!(steps.commands().unwrap().len(), 3);
    assert!(steps.command().is_none());
    assert!(matches!(
        parse_document_for_schema(3, STEPS).unwrap(),
        Document::V3(_)
    ));
    validate_for_schema(3, SCRIPT).unwrap();
    validate_for_schema(3, STEPS).unwrap();
    assert!(validate_for_schema(2, SCRIPT).is_err());
    assert!(v1::parse(SCRIPT).is_err());
    assert!(v2::parse(SCRIPT).is_err());
}

#[test]
fn roundtrip_all_execution_contexts_and_both_command_shapes() {
    for session in [
        v3::Session::Inherit,
        v3::Session::Wayland,
        v3::Session::Hyprland,
    ] {
        for gpu in [v3::Gpu::Inherit, v3::Gpu::Integrated, v3::Gpu::Nvidia] {
            let env = v3::Environment::new(session, gpu);
            for source in [
                v3::CommandSource::Script("printf 'hello: world'\nand echo done\n".into()),
                v3::CommandSource::Steps(vec!["printf ok".into(), "printf next".into()]),
            ] {
                let job =
                    v3::construct(Shell::Fish, "/tmp", Privilege::None, 600, env, source).unwrap();
                let yaml = job.to_yaml().unwrap();
                assert_eq!(v3::parse(&yaml).unwrap(), job);
                assert_eq!(v3::Validator::render(&job).unwrap(), yaml);
                assert!(yaml.contains(&format!("session: {session}")));
                assert!(yaml.contains(&format!("gpu: {gpu}")));
                assert_eq!(yaml.contains("\ncommand:"), job.command().is_some());
                assert_eq!(yaml.contains("\ncommands:"), job.commands().is_some());
            }
        }
    }
}

#[test]
fn invalid_sources_and_environments_fail_closed() {
    let replacements = [
        ("schema: 3", "schema: 2"),
        ("schema: 3", "schema: '3'"),
        ("session: hyprland", "session: gnome"),
        ("session: hyprland", "session: Hyprland"),
        ("gpu: nvidia", "gpu: cuda"),
        ("gpu: nvidia", "gpu: 42"),
        ("gpu: nvidia", "gpu: null"),
        (
            "environment:\n  session: hyprland\n  gpu: nvidia",
            "environment: inherit",
        ),
        (
            "environment:\n  session: hyprland\n  gpu: nvidia",
            "environment:\n  session: hyprland",
        ),
        ("timeout_seconds: 600", "timeout_seconds: '600'"),
        ("timeout_seconds: 600", "timeout_seconds: 0"),
        ("cwd: /home/user/Projects/example", "cwd: ../example"),
        ("command: |", "other: |"),
        ("command: |", "commands: []\nother: |"),
    ];
    for (old, new) in replacements {
        assert!(SCRIPT.contains(old));
        let malformed = SCRIPT.replacen(old, new, 1);
        assert!(
            v3::parse(&malformed).is_err(),
            "unexpectedly accepted: {malformed}"
        );
    }
    for suffix in [
        "command: echo extra\n",
        "commands:\n  - echo extra\n",
        "environment:\n  session: inherit\n  gpu: inherit\n",
        "executor: local\n",
    ] {
        let malformed = format!("{SCRIPT}{suffix}");
        assert!(
            v3::parse(&malformed).is_err(),
            "unexpectedly accepted: {malformed}"
        );
    }
    for bad_environment in [
        "environment:\n  session: hyprland\n  gpu: nvidia\n  extra: unknown",
        "environment:\n  session: hyprland\n  gpu: nvidia\n  session: wayland",
        "environment:\n  session: [hyprland]\n  gpu: nvidia",
    ] {
        let malformed = SCRIPT.replace(
            "environment:\n  session: hyprland\n  gpu: nvidia",
            bad_environment,
        );
        assert!(
            v3::parse(&malformed).is_err(),
            "unexpectedly accepted: {malformed}"
        );
    }
    for source in [
        v3::CommandSource::Script("".into()),
        v3::CommandSource::Script("\0".into()),
        v3::CommandSource::Steps(vec![]),
        v3::CommandSource::Steps(vec!["  ".into()]),
        v3::CommandSource::Steps(vec!["ok".into(); 257]),
        v3::CommandSource::Steps(vec!["x".repeat(MAX_COMMAND_BYTES / 2); 3]),
    ] {
        assert!(v3::construct(
            Shell::Fish,
            "/tmp",
            Privilege::None,
            60,
            v3::Environment::new(v3::Session::Inherit, v3::Gpu::Inherit),
            source
        )
        .is_err());
    }
    assert!(matches!(
        v3::parse(&format!("{SCRIPT}#{}", "x".repeat(MAX_DOCUMENT_BYTES))),
        Err(Error::DocumentTooLarge)
    ));
    assert!(v3::parse(&SCRIPT.replace("gpu: nvidia", "gpu: &gpu nvidia")).is_err());
    assert!(v3::parse(&format!("{SCRIPT}---\nfoo: bar\n")).is_err());
}

#[test]
fn preserves_v1_v2_contracts_and_legacy_entry_points() {
    assert_eq!(execarium::SUPPORTED_SCHEMAS, &[1, 2, 3, 4]);
    assert!(matches!(
        parse_document(include_str!("../examples/local-fish.yaml")).unwrap(),
        Document::V1(_)
    ));
    assert!(matches!(
        parse_document(include_str!("../examples/sequence-fish.yaml")).unwrap(),
        Document::V2(_)
    ));
    assert!(matches!(
        execarium::parse_for_schema(3, SCRIPT),
        Err(Error::UnsupportedSchema(3))
    ));
    assert!(matches!(
        parse_document_for_schema(1, SCRIPT),
        Err(Error::InvalidYaml(_))
    ));
    assert!(matches!(
        validate_for_schema(4, SCRIPT),
        Err(Error::InvalidYaml(_))
    ));
}

#[test]
fn describes_commands_without_starting_them() {
    let sentinel =
        std::env::temp_dir().join(format!("execarium-v3-no-exec-{}", std::process::id()));
    let job = v3::construct(
        Shell::Fish,
        "/tmp",
        Privilege::None,
        60,
        v3::Environment::new(v3::Session::Hyprland, v3::Gpu::Nvidia),
        v3::CommandSource::Script(format!("touch {}", sentinel.display())),
    )
    .unwrap();
    assert!(v3::parse(&job.to_yaml().unwrap()).is_ok());
    assert!(!sentinel.exists());
}
