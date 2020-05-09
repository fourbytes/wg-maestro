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
    pub fn new(config: ClientConfig) -> Self {
        debug!("Setting up client...");
        let wg = WgInterface::from_name(config.interface_name.clone())
            .expect("Failed to connect to Wireguard interface, do we have permission?");
        Self {
            wg,
            config
        }
    }
}
