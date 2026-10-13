//! Real executable CLI contract checks using subprocesses and stdin/files.
//! These tests do not execute the described shell commands.
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

const VALID: &str =
    "schema: 1\nshell: fish\ncwd: /tmp\nprivilege: none\ntimeout_seconds: 600\ncommand: echo ok\n";
static NEXT: AtomicUsize = AtomicUsize::new(0);

struct TestDir(PathBuf);

impl TestDir {
    fn new() -> Self {
        let ordinal = NEXT.fetch_add(1, Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("execarium-cli-{}-{ordinal}", std::process::id()));
        fs::create_dir(&dir).unwrap();
        Self(dir)
    }

    fn file(&self, name: &str, content: &[u8]) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, content).unwrap();
        path
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_execarium"))
}

fn run_stdin(input: &[u8]) -> Output {
    let mut child = binary()
        .args(["validate", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn accepts_stdin_and_reports_only_safe_metadata() {
    let sensitive = VALID.replace("echo ok", "echo private-password-should-not-appear");
    let output = run_stdin(sensitive.as_bytes());
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Valid Execarium schema 1"));
    assert!(stdout.contains("shell=fish"));
    assert!(stdout.contains("timeout_seconds=600"));
    assert!(!stdout.contains("private-password"));
    assert!(output.stderr.is_empty());
}

#[test]
fn reads_valid_yaml_files_without_launching_the_command() {
    let tmp = TestDir::new();
    let sentinel = tmp.0.join("must-not-be-created-by-Execarium");
    let doc = VALID.replace("echo ok", &format!("touch {}", sentinel.display()));
    let path = tmp.file("safe.yaml", doc.as_bytes());
    let out = binary().arg("validate").arg(&path).output().unwrap();
    assert!(out.status.success());
    assert!(!sentinel.exists());
}

#[test]
fn rejects_unsupported_versions_and_malformed_file_inputs() {
    let tmp = TestDir::new();
    for (name, doc) in [
        ("unsupported.yaml", VALID.replace("schema: 1", "schema: 2")),
        ("duplicate.yaml", format!("{VALID}command: override\n")),
        ("extra.yaml", format!("{VALID}executor: local\n")),
        (
            "timeout.yaml",
            VALID.replace("timeout_seconds: 600", "timeout_seconds: 0"),
        ),
        (
            "nonnumeric.yaml",
            VALID.replace("timeout_seconds: 600", "timeout_seconds: \"600\""),
        ),
    ] {
        let path = tmp.file(name, doc.as_bytes());
        let out = binary().arg("validate").arg(&path).output().unwrap();
        assert!(!out.status.success(), "accepted {name}");
        assert!(
            out.stdout.is_empty(),
            "{name} unexpectedly produced success output"
        );
        assert!(!out.stderr.is_empty(), "missing error detail for {name}");
    }
}

#[test]
fn rejects_oversized_files_and_oversized_stdin() {
    let tmp = TestDir::new();
    let input = vec![b'#'; execarium::MAX_DOCUMENT_BYTES + 1];
    let path = tmp.file("too-large.yaml", &input);
    let file_output = binary().arg("validate").arg(&path).output().unwrap();
    assert!(!file_output.status.success());
    let stdin_output = run_stdin(&input);
    assert!(!stdin_output.status.success());
    let msg = String::from_utf8(stdin_output.stderr).unwrap();
    assert!(msg.contains("exceeds"));
}

#[test]
fn invalid_utf8_from_stdin_and_files_is_rejected() {
    let tmp = TestDir::new();
    let path = tmp.file("invalid-utf8.yaml", &[b'a', 0xff, b'b']);
    assert!(!binary()
        .arg("validate")
        .arg(path)
        .output()
        .unwrap()
        .status
        .success());
    assert!(!run_stdin(&[b'a', 0xff, b'b']).status.success());
}

#[test]
fn rejects_wrong_argument_counts_and_unknown_subcommands() {
    for arguments in [
        vec![],
        vec!["validate"],
        vec!["run", "-"],
        vec!["submit", "-"],
        vec!["validate", "-", "extra"],
        vec!["--version", "extra"],
    ] {
        let output = binary().args(arguments).output().unwrap();
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
    }
}

#[test]
fn executable_reports_actual_crate_and_schema_versions() {
    let out = binary().arg("--version").output().unwrap();
    assert!(out.status.success());
    let value = String::from_utf8(out.stdout).unwrap();
    assert_eq!(
        value.trim(),
        format!("execarium {} (schemas: 1,2,3)", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn validator_never_creates_a_file_based_on_attacker_controlled_shell_text() {
    let tmp = TestDir::new();
    let sentinel = tmp.0.join("danger");
    let payload = format!(
        "schema: 1\nshell: sh\ncwd: /tmp\nprivilege: doas\ntimeout_seconds: 30\ncommand: 'touch {}'\n",
        sentinel.display()
    );
    let input = tmp.file("untrusted.yaml", payload.as_bytes());
    let out = binary().arg("validate").arg(input).output().unwrap();
    assert!(out.status.success());
    assert!(!sentinel.exists());
}
