use dioxus::desktop::tao::dpi::LogicalSize;
use dioxus::desktop::tao::window::Icon;
use dioxus::desktop::{Config, WindowBuilder};
use image::GenericImageView;
use web::{ServerPort, start_video_server};

fn load_icon(bytes: &[u8]) -> Icon {
    let img = image::load_from_memory(bytes).unwrap();
    let (width, height) = img.dimensions();

    // Manually create a flat RGBA buffer to ensure 4 bytes per pixel
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for (_, _, pixel) in img.pixels() {
        rgba.extend_from_slice(&pixel.0);
        // pixel.0 is [r, g, b, a] - even if source is RGB,
        // the image crate's .pixels() iterator for DynamicImage
        // typically yields RGBA pixels.
    }

    Icon::from_rgba(rgba, width, height).unwrap()
}

fn main() {
    let icon = load_icon(include_bytes!("../assets/icon.png"));
    let window = WindowBuilder::new()
        .with_window_icon(Some(icon.clone()))
        .with_title("WCO Player")
        .with_always_on_top(false)
        .with_resizable(true)
        .with_inner_size(LogicalSize::new(1280.0, 800.0));

    let server_port = start_video_server();
    let cfg = Config::new().with_window(window).with_icon(icon);

    dioxus::LaunchBuilder::new()
        .with_cfg(cfg)
        .with_context(ServerPort(server_port))
        .launch(web::App);
}
