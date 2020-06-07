use std::fs;

use anyhow::{Error, Result};
use clap::Clap;
use crossbeam_channel::unbounded;
use log::LevelFilter;
use log::*;
use pretty_env_logger;
use serde::de::DeserializeOwned;
use serde_yaml;
use tokio::signal::unix::{signal, SignalKind};

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
    Client(Client),
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
    maestro: Box<dyn WgMaestro>,
}

impl Application {
    pub fn new() -> Result<Self, Error> {
        let opts = Opts::parse();
        {
            let filter_level = match opts.verbose {
                0 => LevelFilter::Info,
                1 => LevelFilter::Debug,
                2 | _ => LevelFilter::Trace,
            };
            pretty_env_logger::formatted_builder()
                .filter(None, filter_level)
                .init()
        };

        info!("Initializing wg-maestro v{:}", VERSION);

        let maestro: Box<dyn WgMaestro>;
        match &opts.subcmd {
            SubCommand::Server(t) => {
                use crate::server::{Server, ServerConfig};
                let config: ServerConfig = Self::load_config(&t.config);
                maestro = Box::new(Server::new(config)?);
            }
            SubCommand::Client(t) => {
                use crate::client::{Client, ClientConfig};
                let config: ClientConfig = Self::load_config(&t.config);
                maestro = Box::new(Client::new(config)?);
            }
        }

        Ok(Self { opts, maestro })
    }

    fn load_config<T: DeserializeOwned + std::fmt::Debug>(config_path: &str) -> T {
        let config_file = fs::read_to_string(config_path).expect("Failed to open config file.");
        let config = serde_yaml::from_str(&config_file).expect("Failed to parse YAML config.");
        debug!("Loaded config from {:?}", config_path);
        trace!("Config data: {:?}", config);
        config
    }

    pub async fn start(&mut self) -> Result<()> {
        let (s, r) = unbounded::<SignalKind>();

        let mut signal_stream = signal(SignalKind::interrupt())?;
        tokio::spawn(async move {
            loop {
                match signal_stream.recv().await {
                    Some(_) => s.send(SignalKind::interrupt()).ok().unwrap(),
                    None => (),
                };
            }
        });

        match self.maestro.run(r).await {
            Err(err) => {
                self.maestro.cleanup().await?;
                Err(err)
            }
            Ok(_) => Ok(()),
        }
    }
}
