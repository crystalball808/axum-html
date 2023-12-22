use anyhow::Result;
use std::process::Command;

pub fn generate_styles() -> Result<String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg("pnpm tailwind:build")
        .output()?;

    Ok(String::from_utf8(output.stdout)?)
}
