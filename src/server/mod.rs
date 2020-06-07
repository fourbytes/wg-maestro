use anyhow::{Error, Result};
use async_trait::async_trait;
use crossbeam_channel::Receiver;
use ipnet::Ipv6Net;
use log::*;
use serde::{Deserialize, Serialize};
use std::net::Ipv6Addr;
use tokio::net::TcpListener;
use tokio::prelude::*;
use tokio::signal::unix::SignalKind;
use wireguard_uapi::set::WgDeviceF;

use crate::common::{base64_to_key, WgInterface, WgKey, WgMaestro};

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerConfig {
    interface_name: String,
    wireguard_port: u16,
    maestro_port: u16,
    fwmark: Option<u32>,
    #[serde(deserialize_with = "base64_to_key")]
    private_key: WgKey,
    addresses: Vec<Address>,
    clients: Vec<Client>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Address {
    prefix: Ipv6Net,
    #[serde(deserialize_with = "base64_to_key")]
    public_key: WgKey,
    pre_shared_key: Option<WgKey>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Client {
    #[serde(deserialize_with = "base64_to_key")]
    public_key: WgKey,
    pre_shared_key: Option<WgKey>,
    hostname: Option<String>,
}

pub struct Server<'a> {
    config: ServerConfig,
    wg: WgInterface<'a>,
    listener: Option<TcpListener>,
    should_exit: bool,
}

#[async_trait]
impl<'a> WgMaestro for Server<'a> {
    async fn run(&mut self, signal_receiver: Receiver<SignalKind>) -> anyhow::Result<()> {
        self.wg.get_device().ok();
        let address = self.wg.get_ll_address()?;
        debug!("Setting Wireguard link-local address to {}", address);

        let server_addr = format!("127.0.0.1:{}", self.config.maestro_port);
        info!("Starting server loop on {}", server_addr);
        self.listener = Some(TcpListener::bind(server_addr).await?);

        loop {
            match signal_receiver.recv() {
                Ok(signal) => {
                    info!("Received signal: {:?}", signal);
                    self.should_exit = true;
                }
                Err(_) => (),
            }
            if self.should_exit {
                debug!("Exiting...");
                return self.cleanup().await;
            }
            self.do_loop()?;
        }
    }

    async fn cleanup(&mut self) -> anyhow::Result<()> {
        log::debug!("Cleaning up...");
        {
            // Shutdown the TCP stream
            match self.listener.take() {
                Some(_) => (),
                _ => (),
            }
        }
        {
            // Remove the wireguard interface
            self.wg.cleanup()?;
        }
        Ok(())
    }
}

impl<'a> Server<'a> {
    pub fn new(config: ServerConfig) -> Result<Self, Error> {
        debug!("Setting up server...");
        let mut wg = WgInterface::from_name(config.interface_name.clone())?;
        let mut device = wg
            .build_set_device()
            .flags(vec![WgDeviceF::ReplacePeers])
            .private_key(&config.private_key)
            .listen_port(config.wireguard_port);

        match config.fwmark {
            Some(fwmark) => device = device.fwmark(fwmark),
            _ => (),
        }

        wg.set_device(device)?;

        Ok(Self {
            config,
            wg,
            listener: None,
            should_exit: false,
        })
    }

    fn do_loop(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}
