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
//! let yaml = "schema: 1\nshell: fish\ncwd: /tmp\nprivilege: none\ncommand: echo ok\n";
//! let job = execarium::parse(yaml).unwrap();
//! assert_eq!(job.schema(), 1);
//! assert_eq!(job.shell(), execarium::Shell::Fish);
//! assert_eq!(job.privilege(), execarium::Privilege::None);
//! assert_eq!(job.cwd(), "/tmp");
//! assert_eq!(job.command(), "echo ok");
//! ```

use serde::Deserialize;
use std::fmt;
use yaml_rust2::scanner::{Scanner, TokenType};

/// The sole supported schema version in this crate release.
pub const SCHEMA_VERSION: u64 = 1;
/// Maximum UTF-8 byte length of a complete YAML document.
pub const MAX_DOCUMENT_BYTES: usize = 256 * 1024;
/// Maximum UTF-8 byte length of the command source string.
pub const MAX_COMMAND_BYTES: usize = 128 * 1024;
/// Maximum UTF-8 byte length of the literal working directory path.
pub const MAX_CWD_BYTES: usize = 4096;

/// The requested local shell *label*. Availability and permission to invoke
/// that shell are the consumer's responsibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    schema: u64,
    shell: Shell,
    cwd: String,
    privilege: Privilege,
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
    pub fn command(&self) -> &str {
        &self.command
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
/// * All five fields are mandatory; extra and duplicate fields are errors.
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
    }
    let raw: RawJob =
        serde_yaml_ng::from_str(input).map_err(|e| Error::InvalidYaml(e.to_string()))?;

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
        command: raw.command,
    })
}

/// Check validity without returning a job. Never interprets, schedules, or
/// executes the document.
pub fn validate(input: &str) -> Result<(), Error> {
    parse(input).map(|_| ())
}
