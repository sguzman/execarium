//! # Execarium
//!
//! Declarative, versioned descriptions of *local shell jobs*.
//!
//! Execarium is a **pure parser and validator**. It does not spawn a shell,
//! access the clipboard, authenticate users, interpret privilege requests as
//! grants, inspect working directories, or execute any operation.
//!
//! ## Example
//!
//! ```
//! let yaml = "schema: 1\nshell: fish\ncwd: /tmp\nprivilege: none\ntimeout_seconds: 600\ncommand: echo ok\n";
//! let job = execarium::parse(yaml).unwrap();
//! assert_eq!(job.schema(), 1);
//! assert_eq!(job.shell(), execarium::Shell::Fish);
//! assert_eq!(job.privilege(), execarium::Privilege::None);
//! assert_eq!(job.cwd(), "/tmp");
//! assert_eq!(job.timeout_seconds(), 600);
//! assert_eq!(job.command(), "echo ok");
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;
use yaml_rust2::scanner::{Scanner, TokenType};

/// The sole supported YAML schema major version in this release.
/// Crate major version and schema major version MUST match.
pub const SCHEMA_VERSION: u64 = 1;
/// Only supported stable YAML wire schema. New major versions add validators
/// explicitly rather than reinterpreting an existing document.
pub const SUPPORTED_SCHEMAS: &[u64] = &[SCHEMA_VERSION];
/// Reference implementation version, independent of the YAML `schema` integer.
pub const IMPLEMENTATION_VERSION: &str = env!("CARGO_PKG_VERSION");
/// Maximum UTF-8 byte length of a complete YAML document.
pub const MAX_DOCUMENT_BYTES: usize = 256 * 1024;
/// Maximum UTF-8 byte length of the command source string.
pub const MAX_COMMAND_BYTES: usize = 128 * 1024;
/// Maximum UTF-8 byte length of the literal working directory path.
pub const MAX_CWD_BYTES: usize = 4096;
/// Minimum allowed execution-time deadline (one second).
pub const MIN_TIMEOUT_SECONDS: u64 = 1;
/// Maximum allowed execution-time deadline (seven days).
pub const MAX_TIMEOUT_SECONDS: u64 = 7 * 24 * 60 * 60;

/// The requested local shell *label*. Availability and permission to invoke
/// that shell are the consumer's responsibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Shell {
    Fish,
    Bash,
    Sh,
}

impl Shell {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fish => "fish",
            Self::Bash => "bash",
            Self::Sh => "sh",
        }
    }
}

impl fmt::Display for Shell {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Requested privilege mode; it is **metadata only**, never permission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Privilege {
    None,
    Sudo,
    Doas,
}

impl Privilege {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Sudo => "sudo",
            Self::Doas => "doas",
        }
    }
}

impl fmt::Display for Privilege {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// A validated specification, *not* a queued or executable job.
///
/// This type is constructible only through the strict parser. Other software
/// must separately authorize and schedule it before performing any action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Job {
    schema: u64,
    shell: Shell,
    cwd: String,
    privilege: Privilege,
    timeout_seconds: u64,
    command: String,
}

impl Job {
    pub const fn schema(&self) -> u64 {
        self.schema
    }
    pub const fn shell(&self) -> Shell {
        self.shell
    }
    pub fn cwd(&self) -> &str {
        &self.cwd
    }
    pub const fn privilege(&self) -> Privilege {
        self.privilege
    }
    /// Requested maximum running time, excluding queue and approval waits.
    /// Enforcement is the responsibility of the consuming executor.
    pub const fn timeout_seconds(&self) -> u64 {
        self.timeout_seconds
    }
    pub fn command(&self) -> &str {
        &self.command
    }

    /// Serialize to YAML preserving all six schema-1 fields. Does not execute.
    /// The output can be validated again by `v1::Validator`.
    pub fn to_yaml(&self) -> Result<String, Error> {
        serde_yaml_ng::to_string(self).map_err(|e| Error::InvalidYaml(e.to_string()))
    }
}

/// Validation errors never make a document runnable.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Execarium document exceeds 262144 bytes")]
    DocumentTooLarge,
    #[error("invalid YAML: {0}")]
    InvalidYaml(String),
    #[error("unsupported YAML feature: {0}")]
    UnsupportedYaml(&'static str),
    #[error("unsupported Execarium schema version {0}; only schema 1 is supported")]
    UnsupportedSchema(u64),
    #[error("invalid working directory: {0}")]
    InvalidCwd(&'static str),
    #[error("invalid timeout: {0}")]
    InvalidTimeout(&'static str),
    #[error("invalid command: {0}")]
    InvalidCommand(&'static str),
}

/// The untrusted wire document. Every field is required, and both duplicate
/// known fields and unknown fields are rejected by serde's struct visitor.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawJob {
    schema: u64,
    shell: Shell,
    cwd: String,
    privilege: Privilege,
    timeout_seconds: u64,
    command: String,
}

/// Reject YAML mechanisms that introduce indirection or type overrides.
///
/// This checks *scanner tokens*, not substrings, so literal `&`, `*`, `!`
/// within command text or quoted scalars are not incorrectly rejected.
fn reject_unsupported_yaml_features(input: &str) -> Result<(), Error> {
    let mut scanner = Scanner::new(input.chars());
    let mut explicit_documents = 0;
    while let Some(token) = scanner
        .next_token()
        .map_err(|error| Error::InvalidYaml(error.to_string()))?
    {
        match token.1 {
            TokenType::Anchor(_) | TokenType::Alias(_) => {
                return Err(Error::UnsupportedYaml("anchors and aliases"));
            }
            TokenType::Tag(_, _) | TokenType::TagDirective(_, _) => {
                return Err(Error::UnsupportedYaml("YAML tags and tag directives"));
            }
            TokenType::VersionDirective(_, _) => {
                return Err(Error::UnsupportedYaml("YAML version directives"));
            }
            TokenType::DocumentStart => {
                explicit_documents += 1;
                if explicit_documents > 1 {
                    return Err(Error::UnsupportedYaml("multiple YAML documents"));
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// Parse one strict v1 document. This function only handles data.
///
/// * All six fields are mandatory; extra and duplicate fields are errors.
/// * There are no defaults, expressions, environment interpolation or
///   filesystem checks.
/// * `privilege` is a requested mode, **not an authorization**.
/// * The function is bounded by `MAX_DOCUMENT_BYTES`.
///
/// Use `schema()` on the returned job for consumer-side version allowlists.
pub fn parse(input: &str) -> Result<Job, Error> {
    if input.len() > MAX_DOCUMENT_BYTES {
        return Err(Error::DocumentTooLarge);
    }
    reject_unsupported_yaml_features(input)?;

    // Serde can coerce some plain YAML scalars into Rust String, e.g.
    // 'command: null'. Check the original YAML value categories first so
    // string fields must actually be YAML strings rather than coerced types.
    let generic: serde_yaml_ng::Value =
        serde_yaml_ng::from_str(input).map_err(|e| Error::InvalidYaml(e.to_string()))?;
    if let Some(mapping) = generic.as_mapping() {
        for name in ["cwd", "command"] {
            if let Some(value) = mapping.get(serde_yaml_ng::Value::String(name.to_owned())) {
                if !matches!(value, serde_yaml_ng::Value::String(_)) {
                    return Err(Error::InvalidYaml(format!(
                        "field {name} must be a YAML string"
                    )));
                }
            }
        }
        // Do not accept "300", 300.0, booleans, or null as an integer.
        if let Some(value) = mapping.get(serde_yaml_ng::Value::String("timeout_seconds".into())) {
            if !matches!(value, serde_yaml_ng::Value::Number(n) if n.as_u64().is_some()) {
                return Err(Error::InvalidYaml(
                    "field timeout_seconds must be an unsigned YAML integer".into(),
                ));
            }
        }
    }
    let raw: RawJob =
        serde_yaml_ng::from_str(input).map_err(|e| Error::InvalidYaml(e.to_string()))?;
    validated_job(raw)
}

/// One shared field validator for parsed YAML and constructed typed jobs.
/// There is no way to construct a public Job through this API without
/// applying the exact same value constraints as YAML parsing.
fn validated_job(raw: RawJob) -> Result<Job, Error> {
    if raw.schema != SCHEMA_VERSION {
        return Err(Error::UnsupportedSchema(raw.schema));
    }
    if raw.cwd.is_empty() || raw.cwd.len() > MAX_CWD_BYTES {
        return Err(Error::InvalidCwd("must contain 1 to 4096 UTF-8 bytes"));
    }
    if !raw.cwd.starts_with('/') {
        return Err(Error::InvalidCwd("must be an absolute POSIX path"));
    }
    if raw.cwd.chars().any(|ch| matches!(ch, '\0' | '\r' | '\n')) {
        return Err(Error::InvalidCwd("must not contain NUL or line breaks"));
    }
    if raw.cwd.split('/').any(|part| part == "." || part == "..") {
        return Err(Error::InvalidCwd(
            "dot and parent-directory segments are not allowed",
        ));
    }
    if !(MIN_TIMEOUT_SECONDS..=MAX_TIMEOUT_SECONDS).contains(&raw.timeout_seconds) {
        return Err(Error::InvalidTimeout(
            "must be between 1 and 604800 seconds",
        ));
    }
    if raw.command.trim().is_empty() {
        return Err(Error::InvalidCommand(
            "must not be empty or whitespace only",
        ));
    }
    if raw.command.len() > MAX_COMMAND_BYTES {
        return Err(Error::InvalidCommand("exceeds 128 KiB"));
    }
    if raw.command.contains('\0') {
        return Err(Error::InvalidCommand("contains NUL"));
    }

    Ok(Job {
        schema: raw.schema,
        shell: raw.shell,
        cwd: raw.cwd,
        privilege: raw.privilege,
        timeout_seconds: raw.timeout_seconds,
        command: raw.command,
    })
}

/// Check validity without returning a job. Never interprets, schedules, or
/// executes the document.
pub fn validate(input: &str) -> Result<(), Error> {
    parse(input).map(|_| ())
}

/// Select an exact schema validator. Reject unsupported versions, and still
/// require the YAML document's own `schema` to match the selected version.
pub fn parse_for_schema(schema: u64, input: &str) -> Result<Job, Error> {
    match schema {
        v1::SCHEMA => v1::Validator::parse(input),
        _ => Err(Error::UnsupportedSchema(schema)),
    }
}

/// Validate using a caller-selected schema version, never a guessed fallback.
pub fn validate_for_schema(schema: u64, input: &str) -> Result<(), Error> {
    parse_for_schema(schema, input).map(|_| ())
}

/// Construct an independently validated schema-1 job from typed Rust values.
///
/// This performs all field validation but does not read the filesystem or
/// authorize, queue, or execute a command. Prefer `v1::construct` when
/// writing a version-specific consumer.
pub fn construct(
    shell: Shell,
    cwd: impl Into<String>,
    privilege: Privilege,
    timeout_seconds: u64,
    command: impl Into<String>,
) -> Result<Job, Error> {
    validated_job(RawJob {
        schema: SCHEMA_VERSION,
        shell,
        cwd: cwd.into(),
        privilege,
        timeout_seconds,
        command: command.into(),
    })
}

/// Stable module namespace for schema 1. Future format versions get separate
/// validators, avoiding accidental silent changes to the v1 contract.
pub mod v1 {
    use crate::{Error, SCHEMA_VERSION};
    pub use crate::{Job, Privilege, Shell};

    /// Exact YAML integer supported by this validator.
    pub const SCHEMA: u64 = SCHEMA_VERSION;

    /// Explicit version-1 validator. No filesystem or process side effects.
    pub struct Validator;

    impl Validator {
        /// Parse and validate a complete schema-1 YAML document.
        pub fn parse(input: &str) -> Result<Job, Error> {
            crate::parse(input)
        }

        /// Validate without returning a job.
        pub fn validate(input: &str) -> Result<(), Error> {
            crate::validate(input)
        }

        /// Construct a validated job from Rust data.
        pub fn construct(
            shell: Shell,
            cwd: impl Into<String>,
            privilege: Privilege,
            timeout_seconds: u64,
            command: impl Into<String>,
        ) -> Result<Job, Error> {
            crate::construct(shell, cwd, privilege, timeout_seconds, command)
        }

        /// Serialize a validated job to YAML. Does not enqueue or execute.
        pub fn render(job: &Job) -> Result<String, Error> {
            job.to_yaml()
        }
    }

    /// Construct a validated v1 job from typed Rust fields.
    pub fn construct(
        shell: Shell,
        cwd: impl Into<String>,
        privilege: Privilege,
        timeout_seconds: u64,
        command: impl Into<String>,
    ) -> Result<Job, Error> {
        Validator::construct(shell, cwd, privilege, timeout_seconds, command)
    }

    /// Parse an exact schema-1 YAML document.
    pub fn parse(input: &str) -> Result<Job, Error> {
        Validator::parse(input)
    }

    /// Validate an exact schema-1 YAML document.
    pub fn validate(input: &str) -> Result<(), Error> {
        Validator::validate(input)
    }
}
