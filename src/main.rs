use anyhow::Error;
use log::*;
use async_std::task;

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
            error!("Encountered failure: {:?}", err)
        }
    }
}
