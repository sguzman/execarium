//! Validation-only CLI; intentionally no run, enqueue or approval command.
use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, Read};
use std::process::ExitCode;

fn read_bounded(mut reader: impl Read) -> Result<String, Box<dyn std::error::Error>> {
    let mut bytes = Vec::new();
    reader
        .by_ref()
        .take((execarium::MAX_DOCUMENT_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > execarium::MAX_DOCUMENT_BYTES {
        return Err(Box::new(execarium::Error::DocumentTooLarge));
    }
    Ok(String::from_utf8(bytes)?)
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let instruction = args.next();
    if instruction.as_deref() == Some(OsStr::new("--version")) {
        if args.next().is_some() {
            return Err("Usage: execarium --version".into());
        }
        println!(
            "execarium {} (schemas: 1,2,3)",
            execarium::IMPLEMENTATION_VERSION
        );
        return Ok(());
    }
    if instruction.as_deref() != Some(OsStr::new("validate")) {
        return Err("Usage: execarium validate <file-or-> | execarium --version".into());
    }
    let source = args.next().ok_or("Usage: execarium validate <file-or->")?;
    if args.next().is_some() {
        return Err("Usage: execarium validate <file-or->".into());
    }
    let document = if source == OsStr::new("-") {
        read_bounded(io::stdin().lock())?
    } else {
        read_bounded(File::open(source)?)?
    };
    let job = execarium::parse_document(&document)?;
    match job {
        execarium::Document::V1(spec) => println!(
            "Valid Execarium schema {} (shell={}, privilege={}, timeout_seconds={})",
            spec.schema(),
            spec.shell(),
            spec.privilege(),
            spec.timeout_seconds()
        ),
        execarium::Document::V2(spec) => println!(
            "Valid Execarium schema {} (shell={}, privilege={}, timeout_seconds={}, steps={})",
            spec.schema(),
            spec.shell(),
            spec.privilege(),
            spec.timeout_seconds(),
            spec.commands().len()
        ),
        execarium::Document::V3(spec) => println!(
            "Valid Execarium schema {} (shell={}, privilege={}, timeout_seconds={}, session={}, gpu={}, steps={})",
            spec.schema(),
            spec.shell(),
            spec.privilege(),
            spec.timeout_seconds(),
            spec.environment().session(),
            spec.environment().gpu(),
            spec.commands().map_or(1, |steps| steps.len())
        ),
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("execarium: {error}");
            ExitCode::FAILURE
        }
    }
}
