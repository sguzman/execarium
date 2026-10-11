//! Execarium schema 2: a bounded, ordered list of independent shell steps.
//! Parsing and construction have no filesystem, authorization, or execution effects.
use crate::{Error, Privilege, Shell, MAX_COMMAND_BYTES, MAX_DOCUMENT_BYTES};
use serde::{Deserialize, Serialize};

pub const SCHEMA: u64 = 2;
pub const MAX_STEPS: usize = 256;

/// A validated schema-2 sequence. Each item is a separate shell invocation.
/// Consumers must stop on the first nonzero exit and enforce one overall timeout.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Job {
    schema: u64,
    shell: Shell,
    cwd: String,
    privilege: Privilege,
    timeout_seconds: u64,
    commands: Vec<String>,
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
    pub const fn timeout_seconds(&self) -> u64 {
        self.timeout_seconds
    }
    pub fn commands(&self) -> &[String] {
        &self.commands
    }
    pub fn to_yaml(&self) -> Result<String, Error> {
        serde_yaml_ng::to_string(self).map_err(|e| Error::InvalidYaml(e.to_string()))
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
    commands: Vec<String>,
}

fn checked(raw: RawJob) -> Result<Job, Error> {
    if raw.schema != SCHEMA {
        return Err(Error::UnsupportedSchema(raw.schema));
    }
    if raw.commands.is_empty() || raw.commands.len() > MAX_STEPS {
        return Err(Error::InvalidCommand(
            "commands must contain 1 to 256 steps",
        ));
    }
    let mut total_bytes = 0usize;
    for command in &raw.commands {
        // Schema 1 and 2 share exact command, cwd and timeout field bounds.
        // This construct operation only validates and never executes.
        crate::v1::Validator::construct(
            raw.shell,
            raw.cwd.as_str(),
            raw.privilege,
            raw.timeout_seconds,
            command.as_str(),
        )?;
        total_bytes = total_bytes.saturating_add(command.len());
        if total_bytes > MAX_COMMAND_BYTES {
            return Err(Error::InvalidCommand("total commands exceed 128 KiB"));
        }
    }
    Ok(Job {
        schema: SCHEMA,
        shell: raw.shell,
        cwd: raw.cwd,
        privilege: raw.privilege,
        timeout_seconds: raw.timeout_seconds,
        commands: raw.commands,
    })
}

pub struct Validator;

impl Validator {
    pub fn parse(input: &str) -> Result<Job, Error> {
        if input.len() > MAX_DOCUMENT_BYTES {
            return Err(Error::DocumentTooLarge);
        }
        crate::reject_unsupported_yaml_features(input)?;
        let generic: serde_yaml_ng::Value =
            serde_yaml_ng::from_str(input).map_err(|e| Error::InvalidYaml(e.to_string()))?;
        let mapping = generic
            .as_mapping()
            .ok_or_else(|| Error::InvalidYaml("top level must be a YAML mapping".into()))?;
        let get = |key: &str| mapping.get(serde_yaml_ng::Value::String(key.to_owned()));
        if !matches!(
            get("schema"),
            Some(serde_yaml_ng::Value::Number(n)) if n.as_u64() == Some(SCHEMA)
        ) {
            return Err(Error::InvalidYaml(
                "schema must be the YAML integer 2".into(),
            ));
        }
        if !matches!(
            get("timeout_seconds"),
            Some(serde_yaml_ng::Value::Number(n)) if n.as_u64().is_some()
        ) {
            return Err(Error::InvalidYaml(
                "timeout_seconds must be an unsigned YAML integer".into(),
            ));
        }
        if !matches!(get("cwd"), Some(serde_yaml_ng::Value::String(_))) {
            return Err(Error::InvalidYaml("cwd must be a YAML string".into()));
        }
        let items = match get("commands") {
            Some(serde_yaml_ng::Value::Sequence(items)) => items,
            _ => {
                return Err(Error::InvalidYaml(
                    "commands must be a YAML sequence".into(),
                ))
            }
        };
        if items
            .iter()
            .any(|item| !matches!(item, serde_yaml_ng::Value::String(_)))
        {
            return Err(Error::InvalidYaml(
                "every command must be a YAML string".into(),
            ));
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
        commands: Vec<String>,
    ) -> Result<Job, Error> {
        checked(RawJob {
            schema: SCHEMA,
            shell,
            cwd: cwd.into(),
            privilege,
            timeout_seconds,
            commands,
        })
    }

    pub fn render(job: &Job) -> Result<String, Error> {
        job.to_yaml()
    }
}

pub fn parse(input: &str) -> Result<Job, Error> {
    Validator::parse(input)
}
pub fn validate(input: &str) -> Result<(), Error> {
    Validator::validate(input)
}
pub fn construct(
    shell: Shell,
    cwd: impl Into<String>,
    privilege: Privilege,
    timeout_seconds: u64,
    commands: Vec<String>,
) -> Result<Job, Error> {
    Validator::construct(shell, cwd, privilege, timeout_seconds, commands)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn good() -> &'static str {
        concat!(
            "schema: 2\n",
            "shell: fish\n",
            "cwd: /tmp\n",
            "privilege: none\n",
            "timeout_seconds: 60\n",
            "commands:\n",
            "  - 'printf one'\n",
            "  - 'printf two'\n",
        )
    }

    #[test]
    fn strict_valid_sequence_round_trip() {
        let job = Validator::parse(good()).unwrap();
        assert!(crate::validate_for_schema(2, good()).is_ok());
        assert!(crate::validate_for_schema(1, good()).is_err());
        assert!(matches!(
            crate::parse_document_for_schema(2, good()).unwrap(),
            crate::Document::V2(_)
        ));
        assert_eq!(job.schema(), 2);
        assert_eq!(
            job.commands()
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            vec!["printf one", "printf two"]
        );
        assert_eq!(
            Validator::parse(&Validator::render(&job).unwrap()).unwrap(),
            job
        );
        assert_eq!(
            Validator::construct(
                Shell::Fish,
                "/tmp",
                Privilege::None,
                60,
                vec!["printf one".into(), "printf two".into()],
            )
            .unwrap(),
            job
        );
    }

    #[test]
    fn rejects_incomplete_wrongly_typed_and_ambiguous_sequences() {
        for bad in [
            good().replace("schema: 2", "schema: 1"),
            good().replace("schema: 2", "schema: '2'"),
            good().replace("timeout_seconds: 60", "timeout_seconds: '60'"),
            good().replace(
                "commands:\n  - 'printf one'\n  - 'printf two'",
                "command: printf one",
            ),
            good().replace(
                "commands:\n  - 'printf one'\n  - 'printf two'",
                "commands: []",
            ),
            good().replace("  - 'printf two'", "  - 42"),
            good().replace("  - 'printf two'", "  - ''"),
            good().replace("  - 'printf two'", "  - null"),
            good().replace(
                "  - 'printf two'",
                "  - 'printf two'\n  - 'printf three'\ncommand: forbidden",
            ),
            good().replace("shell: fish", "shell: unknown"),
            good().replace("cwd: /tmp", "cwd: ../tmp"),
            good().replace("commands:", "commands: &steps"),
            format!("{}commands:\n  - echo duplicate\n", good()),
        ] {
            assert!(
                Validator::parse(&bad).is_err(),
                "unexpected acceptance: {bad}"
            );
        }
        let legacy = concat!(
            "schema: 1\n",
            "shell: fish\n",
            "cwd: /tmp\n",
            "privilege: none\n",
            "timeout_seconds: 60\n",
            "command: printf ok\n",
        );
        assert!(crate::v1::Validator::parse(legacy).is_ok());
        assert!(Validator::parse(legacy).is_err());
    }

    #[test]
    fn enforces_limits_and_never_executes() {
        let big = vec!["x".repeat(MAX_COMMAND_BYTES / 2); 3];
        assert!(Validator::construct(Shell::Fish, "/", Privilege::None, 60, big).is_err());
        assert!(Validator::construct(Shell::Fish, "/", Privilege::None, 60, vec![]).is_err());
        assert!(Validator::construct(
            Shell::Fish,
            "/",
            Privilege::None,
            60,
            vec!["echo x".into(); MAX_STEPS + 1]
        )
        .is_err());
    }
}
