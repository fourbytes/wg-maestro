pub mod common;
pub mod client;
pub mod server;
mod cli;

fn main() {
    let mut app = cli::Application::new();
    app.start()
}
