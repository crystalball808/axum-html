use std::process::Command;

pub fn generate_styles() -> String {
    let output = Command::new("sh")
        .arg("-c")
        .arg("pnpm tailwind:build")
        .output()
        .expect("Failed to run style generation");

    String::from_utf8(output.stdout).expect("Failed to represent stdout as String")
}
