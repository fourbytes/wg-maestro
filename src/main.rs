pub mod client;
pub mod server;
mod cli;

fn main() {
    let mut maestro = cli::Maestro::new();
    maestro.start()
}
