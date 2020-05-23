use log::*;

pub mod common;
pub mod client;
pub mod server;
mod cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    match cli::Application::new() {
        Ok(mut app) => {
            app.start().await
        }
        Err(err) => {
            error!("Encountered failure: {:?}", err);
            Err(err)
        }
    }
}
