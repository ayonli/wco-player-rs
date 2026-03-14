use std::process::Command;

fn main() {
    compile_typescript().unwrap();
}

fn compile_typescript() -> Result<(), String> {
    let dir = env!("CARGO_MANIFEST_DIR");
    let cmd = if cfg!(target_os = "windows") {
        "npx.cmd"
    } else {
        "npx"
    };
    let result = Command::new(cmd)
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
