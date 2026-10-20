//! Schema 4: a named job with the same execution semantics as schema 3.
//! The name is presentation metadata, not an instruction to the executor.
use crate::{v3, Error, Privilege, Shell, MAX_DOCUMENT_BYTES};
use serde::{Deserialize, Serialize};

pub const SCHEMA: u64 = 4;
pub const MAX_NAME_BYTES: usize = 120;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    name: String,
    inner: v3::Job,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawJob {
    schema: u64,
    name: String,
    shell: Shell,
    cwd: String,
    privilege: Privilege,
    timeout_seconds: u64,
    environment: v3::Environment,
    command: Option<String>,
    commands: Option<Vec<String>>,
}
#[derive(Serialize)]
struct RenderJob<'a> {
    schema: u64,
    name: &'a str,
    shell: Shell,
    cwd: &'a str,
    privilege: Privilege,
    timeout_seconds: u64,
    environment: v3::Environment,
    #[serde(skip_serializing_if = "Option::is_none")]
    command: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    commands: Option<&'a [String]>,
}
impl Job {
    pub const fn schema(&self) -> u64 { SCHEMA }
    pub fn name(&self) -> &str { &self.name }
    pub const fn shell(&self) -> Shell { self.inner.shell() }
    pub fn cwd(&self) -> &str { self.inner.cwd() }
    pub const fn privilege(&self) -> Privilege { self.inner.privilege() }
    pub const fn timeout_seconds(&self) -> u64 { self.inner.timeout_seconds() }
    pub const fn environment(&self) -> v3::Environment { self.inner.environment() }
    pub fn source(&self) -> &v3::CommandSource { self.inner.source() }
    pub fn command(&self) -> Option<&str> { self.inner.command() }
    pub fn commands(&self) -> Option<&[String]> { self.inner.commands() }
    pub fn to_yaml(&self) -> Result<String, Error> {
        serde_yaml_ng::to_string(&RenderJob {
            schema: SCHEMA, name: &self.name, shell: self.shell(), cwd: self.cwd(),
            privilege: self.privilege(), timeout_seconds: self.timeout_seconds(),
            environment: self.environment(), command: self.command(), commands: self.commands(),
        }).map_err(|error| Error::InvalidYaml(error.to_string()))
    }
}
fn checked(raw: RawJob) -> Result<Job, Error> {
    if raw.schema != SCHEMA { return Err(Error::UnsupportedSchema(raw.schema)); }
    if raw.name.len() > MAX_NAME_BYTES || raw.name.trim().is_empty() ||
        raw.name.trim() != raw.name ||
        raw.name.chars().any(|ch| ch.is_control())
    {
        return Err(Error::InvalidYaml("name must be 1-120 UTF-8 bytes, trimmed, with no control characters".into()));
    }
    let source = match (raw.command, raw.commands) {
        (Some(script), None) => v3::CommandSource::Script(script),
        (None, Some(steps)) => v3::CommandSource::Steps(steps),
        _ => return Err(Error::InvalidCommand("exactly one of command or commands is required")),
    };
    let inner = v3::Validator::construct(raw.shell, raw.cwd, raw.privilege,
        raw.timeout_seconds, raw.environment, source)?;
    Ok(Job { name: raw.name, inner })
}
pub struct Validator;
impl Validator {
    pub fn parse(input: &str) -> Result<Job, Error> {
        if input.len() > MAX_DOCUMENT_BYTES { return Err(Error::DocumentTooLarge); }
        crate::reject_unsupported_yaml_features(input)?;
        let generic: serde_yaml_ng::Value = serde_yaml_ng::from_str(input)
            .map_err(|e| Error::InvalidYaml(e.to_string()))?;
        let map = generic.as_mapping()
            .ok_or_else(|| Error::InvalidYaml("top level must be a YAML mapping".into()))?;
        let get = |key: &str| map.get(serde_yaml_ng::Value::String(key.to_owned()));
        if !matches!(get("schema"), Some(serde_yaml_ng::Value::Number(n)) if n.as_u64() == Some(SCHEMA)) {
            return Err(Error::InvalidYaml("schema must be the YAML integer 4".into()));
        }
        if !matches!(get("name"), Some(serde_yaml_ng::Value::String(_))) {
            return Err(Error::InvalidYaml("name must be a YAML string".into()));
        }
        if !matches!(get("timeout_seconds"), Some(serde_yaml_ng::Value::Number(n)) if n.as_u64().is_some()) {
            return Err(Error::InvalidYaml("timeout_seconds must be an unsigned YAML integer".into()));
        }
        if !matches!(get("cwd"), Some(serde_yaml_ng::Value::String(_))) {
            return Err(Error::InvalidYaml("cwd must be a YAML string".into()));
        }
        checked(serde_yaml_ng::from_str(input).map_err(|e| Error::InvalidYaml(e.to_string()))?)
    }
    pub fn validate(input: &str) -> Result<(), Error> { Self::parse(input).map(|_| ()) }
    pub fn construct(name: impl Into<String>, shell: Shell, cwd: impl Into<String>,
        privilege: Privilege, timeout_seconds: u64, environment: v3::Environment,
        source: v3::CommandSource) -> Result<Job, Error>
    {
        let (command, commands) = match source {
            v3::CommandSource::Script(script) => (Some(script), None),
            v3::CommandSource::Steps(steps) => (None, Some(steps)),
        };
        checked(RawJob { schema: SCHEMA, name: name.into(), shell, cwd: cwd.into(),
            privilege, timeout_seconds, environment, command, commands })
    }
    pub fn render(job: &Job) -> Result<String, Error> { job.to_yaml() }
}
pub fn parse(input: &str) -> Result<Job, Error> { Validator::parse(input) }
pub fn validate(input: &str) -> Result<(), Error> { Validator::validate(input) }

#[cfg(test)]
mod tests {
    use super::*;
    fn good() -> &'static str {
        "schema: 4\nname: Verify project\nshell: fish\ncwd: /tmp\nprivilege: none\ntimeout_seconds: 60\nenvironment:\n  session: inherit\n  gpu: inherit\ncommands:\n  - echo one\n  - echo two\n"
    }
    #[test]
    fn round_trip_and_dispatch() {
        let job = Validator::parse(good()).unwrap();
        assert_eq!(job.name(), "Verify project");
        assert!(matches!(crate::parse_document(good()).unwrap(), crate::Document::V4(_)));
        assert_eq!(Validator::parse(&job.to_yaml().unwrap()).unwrap(), job);
        assert_eq!(job.commands().unwrap().len(), 2);
    }
    #[test]
    fn validates_name_and_keeps_v3_strict() {
        for input in [
            good().replace("Verify project", ""),
            good().replace("Verify project", "  Verify project  "),
            good().replace("Verify project", "4"),
            good().replace("name: Verify project\n", ""),
            good().replace("name: Verify project", "name: Verify project\nname: Duplicate"),
            good().replace("schema: 4", "schema: 3"),
            good().replace("  gpu: inherit\n", ""),
            good().replace("  - echo two", "  - echo two\nunknown: key"),
        ] {
            assert!(Validator::parse(&input).is_err(), "accepted invalid YAML: {input}");
        }
        assert!(Validator::construct(" ", Shell::Fish, "/tmp", Privilege::None, 30,
            v3::Environment::new(v3::Session::Inherit, v3::Gpu::Inherit),
            v3::CommandSource::Script("echo ok".into())).is_err());
    }
}
