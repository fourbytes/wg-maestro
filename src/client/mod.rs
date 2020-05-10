use anyhow::Error;
use log::*;
use serde::{ Serialize, Deserialize };

use crate::common::{ WgInterface, WgMaestro };

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientConfig {
    interface_name: String,
    private_key: String,
    links: Vec<ClientLink>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientLink {
    host: String,
    port: u16,
    pre_shared_key: String
}

pub struct Client<'a> {
    config: ClientConfig,
    wg: WgInterface<'a>
}

impl<'a> WgMaestro for Client<'a> {
    fn start(&mut self) {
        info!("Starting client...");
    }
}

impl<'a> Client<'a> {
    pub fn new(config: ClientConfig) -> Result<Self, Error> {
        debug!("Setting up client...");
        match WgInterface::from_name(config.interface_name.clone()) {
            Ok(wg) => Ok(Self { wg, config }),
            Err(err) => Err(err)
        }
    }
}
