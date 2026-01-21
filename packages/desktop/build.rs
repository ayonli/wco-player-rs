fn main() {
    // Compile TypeScript to JavaScript (using shared file from web package, output to desktop assets)
    if let Err(e) =
        build_utils::compile_typescript_to("../web/assets/video_player.ts", Some("assets"))
    {
        eprintln!("❌ {}", e);
        panic!("TypeScript compilation failed");
    }

    // Compile Tailwind CSS (using shared file from ui package)
    build_utils::compile_tailwind(
        "../ui/assets/input.css",
        "assets/tailwind-output.css",
        "desktop",
    );
}
