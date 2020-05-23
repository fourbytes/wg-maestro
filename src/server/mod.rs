use std::net::Ipv6Addr;
use std::collections::hash_map::DefaultHasher;
use std::hash::{ Hash, Hasher };
use byteorder::{ ByteOrder, BigEndian };
use async_trait::async_trait;
use anyhow::{ Error, Result };
use log::*;
use serde::{ Serialize, Deserialize };
use wireguard_uapi::set::WgDeviceF;
use crossbeam_channel::Receiver;
use tokio::net::TcpListener;
use tokio::prelude::*;
use tokio::signal::unix::SignalKind;

use crate::common::{ WgInterface, WgMaestro, WgKey, base64_to_key };

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerConfig {
    interface_name: String,
    listen_port: u16,
    fwmark: Option<u32>,
    #[serde(deserialize_with = "base64_to_key")]
    private_key: WgKey,
    peers: Vec<ServerPeer>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerPeer {
    host: Option<String>,
    port: Option<u16>,
    #[serde(deserialize_with = "base64_to_key")]
    public_key: WgKey,
    // #[serde(deserialize_with = "base64_to_key")]
    // pre_shared_key: Option<WgKey>
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
        let device = self.wg.get_device();
        let address = self.get_address()?;
        debug!("Setting Wireguard link-local address to {}", address);

        let server_addr = format!("127.0.0.1:{}", self.config.listen_port);
        info!("Starting server loop on {}", server_addr);
        self.listener = Some(TcpListener::bind(server_addr).await?);

        loop {
            match signal_receiver.recv() {
                Ok(signal) => {
                    info!("Received signal: {:?}", signal);
                    self.should_exit = true;
                }
                Err(_) => ()
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
                _ => ()
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
        let mut device = wg.build_set_device()
            .flags(vec![WgDeviceF::ReplacePeers])
            .private_key(&config.private_key)
            .listen_port(config.listen_port);

        match config.fwmark {
            Some(fwmark) => device = device.fwmark(fwmark),
            _ => ()
        }

        wg.set_device(device)?;

        Ok(Self {
            config,
            wg,
            listener: None,
            should_exit: false
        })
    }

    fn do_loop(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    fn get_address(&self) -> Result<Ipv6Addr> {
        // Replace DefaultHasher with a more robust solution.
        let hash = {
            let mut hasher = DefaultHasher::new();
            self.wg.get_public_key().hash(&mut hasher);
            hasher.finish()
        };
        let data = {
            let mut buf = [0u8; 8];
            BigEndian::write_u64(&mut buf, hash);
            let mut data = [0u16; 4];
            for (i, item) in buf.chunks(2).enumerate() {
                data[i] = BigEndian::read_u16(item)
            }
            data
        };
        Ok(Ipv6Addr::new(0xfe80, 0, 0, 0, data[0], data[1], data[2], data[3]))
    }
}
