use desktop::start_video_server;
use web::ServerPort;

fn main() {
    let server_port = start_video_server();

    dioxus::LaunchBuilder::new()
        .with_context(ServerPort(server_port))
        .launch(web::App);
}
