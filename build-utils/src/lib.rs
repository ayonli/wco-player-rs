//! Shared build utilities for compiling TypeScript and Tailwind CSS

use std::process::Command;

/// Compile TypeScript to JavaScript using tsc
/// Output file is placed in the same directory with .js extension, or in output_dir if specified
pub fn compile_typescript(ts_file: &str) -> Result<(), String> {
    compile_typescript_to(ts_file, None)
}

/// Compile TypeScript to JavaScript using tsc with custom output directory
pub fn compile_typescript_to(ts_file: &str, output_dir: Option<&str>) -> Result<(), String> {
    println!("cargo:rerun-if-changed={}", ts_file);

    // Get current working directory (build.rs runs from package root)
    let current_dir =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;

    // Get the directory of the TypeScript file (resolve relative to current dir)
    let ts_path = current_dir.join(ts_file);
    let ts_dir = ts_path.parent().ok_or("Invalid TypeScript file path")?;
    let ts_filename = ts_path
        .file_name()
        .ok_or("Invalid TypeScript file path")?
        .to_str()
        .ok_or("Invalid TypeScript file name")?;

    // Determine output directory (resolve relative to current dir)
    let out_dir = if let Some(output) = output_dir {
        current_dir.join(output)
    } else {
        ts_dir.to_path_buf()
    };

    // Ensure output directory exists
    std::fs::create_dir_all(&out_dir)
        .map_err(|e| format!("Failed to create output directory: {}", e))?;

    // Generate output file path
    let js_filename = ts_filename.replace(".ts", ".js");
    let js_file = out_dir.join(&js_filename);

    // Create a temporary tsconfig.json in the same directory as the TypeScript file
    let tsconfig_path = ts_dir.join("tsconfig.temp.json");

    // Convert out_dir to absolute path string for tsconfig
    // Use absolute path to avoid issues with cross-directory compilation
    let out_dir_str = out_dir
        .canonicalize()
        .unwrap_or_else(|_| out_dir.to_path_buf())
        .to_str()
        .ok_or("Invalid output directory path")?
        .to_string();

    let tsconfig_content = format!(
        r#"{{
  "compilerOptions": {{
    "target": "ES2020",
    "module": "ES2020",
    "lib": ["ES2020", "DOM"],
    "moduleResolution": "node",
    "outDir": "{}",
    "rootDir": ".",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "declaration": false,
    "sourceMap": false
  }},
  "include": ["{}"],
  "exclude": ["node_modules", "dist", "target"]
}}
"#,
        out_dir_str, ts_filename
    );

    std::fs::write(&tsconfig_path, tsconfig_content)
        .map_err(|e| format!("Failed to create temporary tsconfig.json: {}", e))?;

    // Run tsc with the temporary tsconfig (use npx to ensure tsc is available)
    let output = Command::new("npx")
        .arg("--yes")
        .arg("tsc")
        .arg("--project")
        .arg(&tsconfig_path)
        .output();

    // Clean up temporary tsconfig
    let _ = std::fs::remove_file(&tsconfig_path);

    match output {
        Ok(output) if output.status.success() => {
            // Verify output file exists
            if !js_file.exists() {
                return Err(format!(
                    "TypeScript compilation succeeded but output file not found: {}",
                    js_file.display()
                ));
            }
            println!("✅ Successfully compiled TypeScript to JavaScript");
            Ok(())
        }
        Ok(output) => {
            let error = format!(
                "Failed to compile TypeScript:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
            Err(error)
        }
        Err(e) => Err(format!("Failed to run tsc (via npx): {}", e)),
    }
}

/// Compile Tailwind CSS
pub fn compile_tailwind(input_css: &str, output_css: &str, platform: &str) {
    println!("cargo:rerun-if-changed={}", input_css);

    let output = Command::new("npx")
        .arg("@tailwindcss/cli")
        .arg("-i")
        .arg(input_css)
        .arg("-o")
        .arg(output_css)
        .output();

    match output {
        Ok(output) if output.status.success() => {
            println!("✅ Successfully compiled Tailwind CSS for {}", platform);
        }
        Ok(output) => {
            eprintln!("❌ Failed to compile Tailwind CSS:");
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
            eprintln!("⚠️ Continuing without Tailwind CSS");
        }
        Err(e) => {
            eprintln!("⚠️ Failed to run tailwindcss CLI: {}", e);
            eprintln!("⚠️ Continuing without Tailwind CSS");
        }
    }
}
