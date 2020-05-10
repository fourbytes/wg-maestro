use async_trait::async_trait;
use anyhow::Error;
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

#[async_trait]
impl<'a> WgMaestro for Server<'a> {
    async fn run(&mut self) {
        info!("Starting server loop...");

        let device = self.wg.get_device();
        debug!("Read Wireguard interface data: {:?}", device);
    }
}

impl<'a> Server<'a> {
    pub fn new(config: ServerConfig) -> Result<Self, Error> {
        debug!("Setting up server...");
        match WgInterface::from_name(config.interface_name.clone()) {
            Ok(wg) => Ok(Self { wg, config }),
            Err(err) => Err(err)
        }
    }
}
