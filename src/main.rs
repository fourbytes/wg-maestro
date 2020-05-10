use anyhow::Error;
use log::*;

pub mod common;
pub mod client;
pub mod server;
mod cli;

fn main() {
    match cli::Application::new() {
        Ok(mut app) => {
            app.start()
        }
        Err(err) => {
            error!("Failed to start application: {:?}", err)
        }
    }
}
