use std::process::Command;

fn main() {
    compile_typescript().unwrap();
}

fn compile_typescript() -> Result<(), String> {
    let dir = env!("CARGO_MANIFEST_DIR");
    let result = Command::new("npx")
        .current_dir(dir)
        .arg("--yes")
        .arg("tsc")
        .output();

    match result {
        Ok(output) if output.status.success() => Ok(()),
        Ok(output) => {
            let error = format!(
                "Failed to compile TypeScript:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
            Err(error)
        }
        Err(e) => Err(format!("Failed to compile TypeScript: {}", e)),
    }
}
