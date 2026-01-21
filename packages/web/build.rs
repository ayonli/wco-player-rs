fn main() {
    // Compile TypeScript to JavaScript
    if let Err(e) = build_utils::compile_typescript("assets/video_player.ts") {
        eprintln!("❌ {}", e);
        panic!("TypeScript compilation failed");
    }

}
