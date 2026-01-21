fn main() {
    // Compile TypeScript to JavaScript (using shared file from web package, output to desktop assets)
    if let Err(e) =
        build_utils::compile_typescript_to("../web/assets/video_player.ts", Some("assets"))
    {
        eprintln!("❌ {}", e);
        panic!("TypeScript compilation failed");
    }
}
