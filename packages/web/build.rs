fn main() {
    // Compile TypeScript to JavaScript
    if let Err(e) = build_utils::compile_typescript("assets/video_player.ts") {
        eprintln!("❌ {}", e);
        panic!("TypeScript compilation failed");
    }

    // Compile Tailwind CSS (using shared file from ui package)
    build_utils::compile_tailwind(
        "../ui/assets/input.css",
        "assets/tailwind-output.css",
        "web",
    );
}
