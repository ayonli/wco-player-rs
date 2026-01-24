use web::{ServerPort, start_video_server};

fn main() {
    let server_port = start_video_server();

    dioxus::LaunchBuilder::new()
        .with_context(ServerPort(server_port))
        .launch(web::App);
}
