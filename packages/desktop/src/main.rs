use web::{ServerPort, start_video_server};

fn main() {
    use dioxus::desktop::tao::dpi::LogicalSize;
    use dioxus::desktop::{Config, WindowBuilder};

    let window = WindowBuilder::new()
        .with_title("WCO Player")
        .with_always_on_top(false)
        .with_resizable(true)
        .with_inner_size(LogicalSize::new(1280.0, 800.0));

    let server_port = start_video_server();

    dioxus::LaunchBuilder::new()
        .with_cfg(Config::new().with_window(window))
        .with_context(ServerPort(server_port))
        .launch(web::App);
}
