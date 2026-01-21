fn main() {
    // Compile Tailwind CSS (using shared file from ui package)
    build_utils::compile_tailwind(
        "../ui/assets/input.css",
        "assets/tailwind-output.css",
        "mobile",
    );
}
