//! Stable, language-neutral conformance corpus for YAML schema 1.
//! Other implementations can reuse tests/fixtures/v1/{valid,invalid}.
use execarium::v1;

#[test]
fn all_valid_v1_corpus_files_are_accepted_and_round_trip() {
    let valid: [(&str, &str); 6] = [
        ("fish-basic", include_str!("fixtures/v1/valid/fish-basic.yaml")),
        ("bash-multiline", include_str!("fixtures/v1/valid/bash-multiline.yaml")),
        ("doas-manual-approval", include_str!("fixtures/v1/valid/doas-manual-approval.yaml")),
        ("quoted-command", include_str!("fixtures/v1/valid/quoted-command.yaml")),
        ("unicode", include_str!("fixtures/v1/valid/unicode.yaml")),
        ("quoted-scalar", include_str!("fixtures/v1/valid/quoted-scalar.yaml")),
    ];
    for (name, input) in valid {
        let job = v1::Validator::parse(input)
            .unwrap_or_else(|err| panic!("valid fixture {name} rejected: {err}"));
        let output = job.to_yaml().unwrap();
        assert_eq!(v1::Validator::parse(&output).unwrap(), job, "{name}");
    }
}

#[test]
fn all_invalid_v1_corpus_files_are_rejected() {
    let invalid: [(&str, &str); 14] = [
        ("missing-timeout", include_str!("fixtures/v1/invalid/missing-timeout.yaml")),
        ("unknown-field", include_str!("fixtures/v1/invalid/unknown-field.yaml")),
        ("duplicate-key", include_str!("fixtures/v1/invalid/duplicate-key.yaml")),
        ("unsupported-schema", include_str!("fixtures/v1/invalid/unsupported-schema.yaml")),
        ("quoted-schema", include_str!("fixtures/v1/invalid/quoted-schema.yaml")),
        ("bad-shell", include_str!("fixtures/v1/invalid/bad-shell.yaml")),
        ("bad-privilege", include_str!("fixtures/v1/invalid/bad-privilege.yaml")),
        ("relative-cwd", include_str!("fixtures/v1/invalid/relative-cwd.yaml")),
        ("zero-timeout", include_str!("fixtures/v1/invalid/zero-timeout.yaml")),
        ("quoted-timeout", include_str!("fixtures/v1/invalid/quoted-timeout.yaml")),
        ("null-command", include_str!("fixtures/v1/invalid/null-command.yaml")),
        ("alias", include_str!("fixtures/v1/invalid/alias.yaml")),
        ("tag", include_str!("fixtures/v1/invalid/tag.yaml")),
        ("multidocument", include_str!("fixtures/v1/invalid/multidocument.yaml")),
    ];
    for (name, input) in invalid {
        assert!(v1::Validator::parse(input).is_err(), "invalid fixture {name} accepted");
        assert!(v1::Validator::validate(input).is_err(), "validator accepted {name}");
    }
}
