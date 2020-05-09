use log::*;
use serde::{ Serialize, Deserialize };

use crate::common::{ WgInterface, WgMaestro };

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerConfig {
    interface_name: String,
    private_key: String,
    links: Vec<ServerLink>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerLink {
    host: String,
    port: u16,
    pre_shared_key: String
}

pub struct Server<'a> {
    config: ServerConfig,
    wg: WgInterface<'a>
}

impl<'a> WgMaestro for Server<'a> {
    fn start(&mut self) {
        info!("Starting server...");

        let device = self.wg.get_device();
        debug!("Read Wireguard interface data: {:?}", device);
    }
}

impl<'a> Server<'a> {
    pub fn new(config: ServerConfig) -> Self {
        debug!("Setting up server...");
        let wg = WgInterface::from_name(config.interface_name.clone())
            .expect("Failed to connect to Wireguard interface, do we have permission?");
        Self {
            wg,
            config
        }
    }
}
