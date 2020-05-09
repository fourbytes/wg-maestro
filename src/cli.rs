use std::fs;

use serde_yaml;
use serde::de::DeserializeOwned;
use pretty_env_logger;
use log::LevelFilter;
use log::*;
use clap::Clap;

use crate::common::WgMaestro;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

/// This doc string acts as a help message when the user runs '--help'
/// as do all doc strings on fields
#[derive(Clap)]
#[clap(version = VERSION, author = "Oscar R. <oscar@fourbs.com.au>")]
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
    #[clap(default_value = "server.yaml")]
    config: String,
    
}

#[derive(Clap)]
struct Client {
    /// Set config file location. 
    #[clap(default_value = "client.yaml")]
    config: String,
}

pub struct Application {
    opts: Opts,
    maestro: Box<dyn WgMaestro>
}

impl Application {

    pub fn new() -> Self {
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

        let maestro: Box<dyn WgMaestro>;
        match &opts.subcmd {
            SubCommand::Server(t) => {
                use crate::server::{ ServerConfig, Server };
                let config: ServerConfig = Self::load_config(&t.config);
                maestro = Box::new(Server::new(config));
            }
            SubCommand::Client(t) => {
                use crate::client::{ ClientConfig, Client };
                let config: ClientConfig = Self::load_config(&t.config);
                maestro = Box::new(Client::new(config));
            }
        }

        Self {
            opts,
            maestro
        }
    }

    fn load_config<T: DeserializeOwned>(config_path: &str) -> T {
        let config_file = fs::read_to_string(config_path).expect("Failed to open config file.");
        serde_yaml::from_str(&config_file).expect("Failed to parse YAML config.")
    }

    pub fn start(&mut self) {
        info!("Initializing wg-maestro v{:}", VERSION);
        self.maestro.start()
    }
}
