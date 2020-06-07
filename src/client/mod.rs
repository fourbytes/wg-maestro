use anyhow::{Error, Result};
use async_trait::async_trait;
use crossbeam_channel::Receiver;
use log::*;
use serde::{Deserialize, Serialize};
use tokio::signal::unix::SignalKind;

use crate::common::{base64_to_key, WgInterface, WgKey, WgMaestro};

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientConfig {
    interface_name: String,
    #[serde(deserialize_with = "base64_to_key")]
    private_key: WgKey,
    peers: Vec<ClientPeer>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientPeer {
    host: Option<String>,
    port: Option<u16>,
    #[serde(deserialize_with = "base64_to_key")]
    public_key: WgKey,
    // #[serde(deserialize_with = "base64_to_key")]
    // pre_shared_key: Option<WgKey>
}

pub struct Client<'a> {
    config: ClientConfig,
    wg: WgInterface<'a>,
}

#[async_trait]
impl<'a> WgMaestro for Client<'a> {
    async fn run(&mut self, signal_receiver: Receiver<SignalKind>) -> Result<()> {
        info!("Starting client...");
        Ok(())
    }

    async fn cleanup(&mut self) -> Result<()> {
        debug!("Cleaning up...");
        Ok(())
    }
}

impl<'a> Client<'a> {
    pub fn new(config: ClientConfig) -> Result<Self, Error> {
        debug!("Setting up client...");
        let wg = WgInterface::from_name(config.interface_name.clone())?;
        Ok(Self { config, wg })
    }
}
