use anyhow::{Error, Result};
use async_trait::async_trait;
use crossbeam_channel::Receiver;
use ipnet::{IpNet, Ipv6Net};
use log::*;
use serde::{Deserialize, Serialize};
use tokio::signal::unix::SignalKind;

use wireguard_uapi::set::WgDeviceF;
use wireguard_uapi::WireGuardDeviceAddrScope;

use crate::common::{base64_to_key, WgInterface, WgKey, WgMaestro};

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientConfig {
    interface_name: String,
    wireguard_port: u16,
    maestro_port: u16,
    fwmark: Option<u32>,
    #[serde(deserialize_with = "base64_to_key")]
    private_key: WgKey,
    server: ClientConfigServer,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientConfigServer {
    host: String,
    wireguard_port: u16,
    maestro_port: u16,
    #[serde(deserialize_with = "base64_to_key")]
    public_key: WgKey,
}

pub struct Client<'a> {
    config: ClientConfig,
    wg: WgInterface<'a>,
    should_exit: bool,
}

#[async_trait]
impl<'a> WgMaestro for Client<'a> {
    async fn run(&mut self, signal_receiver: Receiver<SignalKind>) -> Result<()> {
        let device = self.wg.get_device().ok().unwrap();
        info!(
            "Configured Wireguard interface (received public key {})",
            base64::encode(device.public_key.unwrap())
        );

        let ll_address = self.wg.get_ll_address()?;
        debug!("Setting Wireguard link-local address to {}", ll_address);
        let ll_net = Ipv6Net::new(ll_address, 64)?;
        self.wg
            .setup_address(IpNet::V6(ll_net), WireGuardDeviceAddrScope::Link)
            .await?;

        info!("Starting client...");

        // self.wg.build_set_device().peers(Peer::new())

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

        Ok(())
    }

    async fn cleanup(&mut self) -> Result<()> {
        debug!("Cleaning up...");
        {
            // Remove the wireguard interface
            self.wg.cleanup()?;
        }
        Ok(())
    }
}

impl<'a> Client<'a> {
    pub fn new(config: ClientConfig) -> Result<Self, Error> {
        debug!("Setting up client...");
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
            should_exit: false,
        })
    }

    fn do_loop(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}
