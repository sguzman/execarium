//! Stable, language-neutral schema-1 YAML fixtures, reusable by other parsers.
use execarium::v1;
use std::fs;
use std::path::Path;

fn verify_directory(name: &str, expected_count: usize, valid: bool) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("v1")
        .join(name);
    let mut count = 0;
    for entry in fs::read_dir(root).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        assert_eq!(path.extension().and_then(|s| s.to_str()), Some("yaml"));
        let input = fs::read_to_string(&path).unwrap();
        let result = v1::Validator::parse(&input);
        if valid {
            let job = result.unwrap_or_else(|err| panic!("{}: {err}", path.display()));
            let encoded = job.to_yaml().unwrap();
            assert_eq!(v1::Validator::parse(&encoded).unwrap(), job);
        } else {
            assert!(
                result.is_err(),
                "invalid fixture accepted: {}",
                path.display()
            );
            assert!(v1::Validator::validate(&input).is_err());
        }
        count += 1;
    }
    assert_eq!(count, expected_count, "fixture corpus unexpectedly changed");
}

#[test]
fn valid_v1_fixtures_are_accepted_and_round_trip() {
    verify_directory("valid", 6, true);
}

#[test]
fn invalid_v1_fixtures_are_all_rejected() {
    verify_directory("invalid", 14, false);
}
