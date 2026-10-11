//! Construct and serialize a validated Execarium schema-1 request.
//! This example does NOT execute a shell or schedule any work.
use execarium::{v1, Privilege, Shell};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let job = v1::Validator::construct(
        Shell::Fish,
        "/home/user/Projects/example",
        Privilege::None,
        600,
        "cargo test",
    )?;
    let yaml = v1::Validator::render(&job)?;
    v1::Validator::validate(&yaml)?;
    print!("{yaml}");
    Ok(())
}
