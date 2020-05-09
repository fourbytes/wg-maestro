use std::fs;

use toml;
use serde::de::DeserializeOwned;
use pretty_env_logger;
use log::LevelFilter;
use log::*;
use clap::Clap;

use crate::client::ClientConfig;
use crate::server::ServerConfig;

/// This doc string acts as a help message when the user runs '--help'
/// as do all doc strings on fields
#[derive(Clap)]
#[clap(version = "1.0", author = "Oscar R. <oscar@fourbs.com.au>")]
struct Opts {
    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    Server(Server),
    Client(Client)
}

/// A subcommand for starting the server.
#[derive(Clap)]
struct Server {
    /// Set config file location.
    #[clap(default_value = "server.toml")]
    config: String,
    
}

#[derive(Clap)]
struct Client {
    /// Set config file location. 
    #[clap(default_value = "client.toml")]
    config: String,
}

pub struct Maestro {
    opts: Opts,
}


impl Maestro {

    pub fn new() -> Maestro {
        let opts = Opts::parse();
        {
            let filter_level = match opts.verbose {
                0 => LevelFilter::Info,
                1 => LevelFilter::Debug,
                2 | _ => LevelFilter::Trace
            };
            pretty_env_logger::formatted_builder()
                .filter(None, filter_level)
                .init()
        };

        Maestro {
            opts
        }
    }

    fn load_config<T: DeserializeOwned>(config_path: &str) -> T {
        let config_file = fs::read_to_string(config_path).expect("Failed to open config file.");
        toml::from_str(&config_file).expect("Failed to parse toml.")
    }

    pub fn start(&mut self) {
        match &self.opts.subcmd {
            SubCommand::Server(t) => {
                let config: ServerConfig = Self::load_config(&t.config);
                debug!("Loaded config: {:?}", config);
                self.start_server(config)
            }
            SubCommand::Client(t) => {
                let config: ClientConfig = Self::load_config(&t.config);
                debug!("Loaded config: {:?}", config);
                self.start_client(config)
            }
        }
    }

    fn start_server(&self, server_config: ServerConfig) {
        info!("Starting server...");
    }

    fn start_client(&self, client_config: ClientConfig) {
        info!("Starting client...")
    }
}
