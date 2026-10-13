//! Schema 3: local shell jobs with explicit execution-environment requests.
//! Parsing, construction and rendering have no filesystem or execution effects.

use crate::{Error, Privilege, Shell, MAX_COMMAND_BYTES, MAX_DOCUMENT_BYTES};
use serde::{Deserialize, Serialize};
use std::fmt;

pub const SCHEMA: u64 = 3;
pub const MAX_STEPS: usize = 256;

/// Requested desktop session, not a permission mechanism.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Session {
    Inherit,
    Wayland,
    Hyprland,
}
impl Session {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Inherit => "inherit",
            Self::Wayland => "wayland",
            Self::Hyprland => "hyprland",
        }
    }
}
impl fmt::Display for Session {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Requested GPU preference. Consumers implement API-specific routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Gpu {
    Inherit,
    Integrated,
    Nvidia,
}
impl Gpu {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Inherit => "inherit",
            Self::Integrated => "integrated",
            Self::Nvidia => "nvidia",
        }
    }
}
impl fmt::Display for Gpu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Both context fields are mandatory, even when set to inherit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Environment {
    session: Session,
    gpu: Gpu,
}
impl Environment {
    pub const fn new(session: Session, gpu: Gpu) -> Self {
        Self { session, gpu }
    }
    pub const fn session(&self) -> Session {
        self.session
    }
    pub const fn gpu(&self) -> Gpu {
        self.gpu
    }
}

/// Exactly one: a script sharing shell state, or separate stop-on-failure steps.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandSource {
    Script(String),
    Steps(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    schema: u64,
    shell: Shell,
    cwd: String,
    privilege: Privilege,
    timeout_seconds: u64,
    environment: Environment,
    source: CommandSource,
}

#[derive(Serialize)]
struct RenderJob<'a> {
    schema: u64,
    shell: Shell,
    cwd: &'a str,
    privilege: Privilege,
    timeout_seconds: u64,
    environment: Environment,
    #[serde(skip_serializing_if = "Option::is_none")]
    command: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    commands: Option<&'a [String]>,
}

impl Job {
    pub const fn schema(&self) -> u64 { self.schema }
    pub const fn shell(&self) -> Shell { self.shell }
    pub fn cwd(&self) -> &str { &self.cwd }
    pub const fn privilege(&self) -> Privilege { self.privilege }
    pub const fn timeout_seconds(&self) -> u64 { self.timeout_seconds }
    pub const fn environment(&self) -> Environment { self.environment }
    pub fn source(&self) -> &CommandSource { &self.source }
    pub fn command(&self) -> Option<&str> {
        match &self.source {
            CommandSource::Script(script) => Some(script),
            CommandSource::Steps(_) => None,
        }
    }
    pub fn commands(&self) -> Option<&[String]> {
        match &self.source {
            CommandSource::Script(_) => None,
            CommandSource::Steps(steps) => Some(steps),
        }
    }
    pub fn to_yaml(&self) -> Result<String, Error> {
        let (command, commands) = match &self.source {
            CommandSource::Script(script) => (Some(script.as_str()), None),
            CommandSource::Steps(steps) => (None, Some(steps.as_slice())),
        };
        serde_yaml_ng::to_string(&RenderJob {
            schema: SCHEMA,
            shell: self.shell,
            cwd: &self.cwd,
            privilege: self.privilege,
            timeout_seconds: self.timeout_seconds,
            environment: self.environment,
            command,
            commands,
        })
        .map_err(|e| Error::InvalidYaml(e.to_string()))
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawJob {
    schema: u64,
    shell: Shell,
    cwd: String,
    privilege: Privilege,
    timeout_seconds: u64,
    environment: Environment,
    command: Option<String>,
    commands: Option<Vec<String>>,
}

fn checked(raw: RawJob) -> Result<Job, Error> {
    if raw.schema != SCHEMA {
        return Err(Error::UnsupportedSchema(raw.schema));
    }
    let source = match (raw.command, raw.commands) {
        (Some(script), None) => CommandSource::Script(script),
        (None, Some(steps)) => CommandSource::Steps(steps),
        _ => return Err(Error::InvalidCommand("exactly one of command or commands is required")),
    };
    match &source {
        CommandSource::Script(script) => {
            crate::v1::construct(
                raw.shell, raw.cwd.as_str(), raw.privilege,
                raw.timeout_seconds, script.as_str(),
            )?;
        }
        CommandSource::Steps(steps) => {
            if steps.is_empty() || steps.len() > MAX_STEPS {
                return Err(Error::InvalidCommand("commands must contain 1 to 256 steps"));
            }
            let mut total = 0usize;
            for step in steps {
                crate::v1::construct(
                    raw.shell, raw.cwd.as_str(), raw.privilege,
                    raw.timeout_seconds, step.as_str(),
                )?;
                total = total.saturating_add(step.len());
                if total > MAX_COMMAND_BYTES {
                    return Err(Error::InvalidCommand("total commands exceed 128 KiB"));
                }
            }
        }
    }
    Ok(Job {
        schema: SCHEMA,
        shell: raw.shell,
        cwd: raw.cwd,
        privilege: raw.privilege,
        timeout_seconds: raw.timeout_seconds,
        environment: raw.environment,
        source,
    })
}

pub struct Validator;
impl Validator {
    pub fn parse(input: &str) -> Result<Job, Error> {
        if input.len() > MAX_DOCUMENT_BYTES {
            return Err(Error::DocumentTooLarge);
        }
        crate::reject_unsupported_yaml_features(input)?;
        let value: serde_yaml_ng::Value =
            serde_yaml_ng::from_str(input).map_err(|e| Error::InvalidYaml(e.to_string()))?;
        let mapping = value.as_mapping()
            .ok_or_else(|| Error::InvalidYaml("top level must be a YAML mapping".into()))?;
        let get = |name: &str| mapping.get(serde_yaml_ng::Value::String(name.into()));
        if !matches!(
            get("schema"),
            Some(serde_yaml_ng::Value::Number(n)) if n.as_u64() == Some(SCHEMA)
        ) {
            return Err(Error::InvalidYaml("schema must be the YAML integer 3".into()));
        }
        if !matches!(
            get("timeout_seconds"),
            Some(serde_yaml_ng::Value::Number(n)) if n.as_u64().is_some()
        ) {
            return Err(Error::InvalidYaml("timeout_seconds must be an unsigned YAML integer".into()));
        }
        if !matches!(get("cwd"), Some(serde_yaml_ng::Value::String(_))) {
            return Err(Error::InvalidYaml("cwd must be a YAML string".into()));
        }
        if let Some(command) = get("command") {
            if !matches!(command, serde_yaml_ng::Value::String(_)) {
                return Err(Error::InvalidYaml("command must be a YAML string".into()));
            }
        }
        if let Some(commands) = get("commands") {
            match commands {
                serde_yaml_ng::Value::Sequence(steps)
                    if steps.iter().all(|step| matches!(step, serde_yaml_ng::Value::String(_))) => {}
                _ => return Err(Error::InvalidYaml("commands must be a sequence of YAML strings".into())),
            }
        }
        if !matches!(get("environment"), Some(serde_yaml_ng::Value::Mapping(_))) {
            return Err(Error::InvalidYaml("environment must be a YAML mapping".into()));
        }
        let raw: RawJob =
            serde_yaml_ng::from_str(input).map_err(|e| Error::InvalidYaml(e.to_string()))?;
        checked(raw)
    }
    pub fn validate(input: &str) -> Result<(), Error> {
        Self::parse(input).map(|_| ())
    }
    pub fn construct(
        shell: Shell,
        cwd: impl Into<String>,
        privilege: Privilege,
        timeout_seconds: u64,
        environment: Environment,
        source: CommandSource,
    ) -> Result<Job, Error> {
        let (command, commands) = match source {
            CommandSource::Script(script) => (Some(script), None),
            CommandSource::Steps(steps) => (None, Some(steps)),
        };
        checked(RawJob {
            schema: SCHEMA,
            shell,
            cwd: cwd.into(),
            privilege,
            timeout_seconds,
            environment,
            command,
            commands,
        })
    }
    pub fn render(job: &Job) -> Result<String, Error> { job.to_yaml() }
}
pub fn parse(input: &str) -> Result<Job, Error> { Validator::parse(input) }
pub fn validate(input: &str) -> Result<(), Error> { Validator::validate(input) }
pub fn construct(
    shell: Shell,
    cwd: impl Into<String>,
    privilege: Privilege,
    timeout_seconds: u64,
    environment: Environment,
    source: CommandSource,
) -> Result<Job, Error> {
    Validator::construct(shell, cwd, privilege, timeout_seconds, environment, source)
}
