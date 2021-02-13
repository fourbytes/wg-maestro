use anyhow::{Error, Result};
use async_trait::async_trait;
use crossbeam_channel::Receiver;
use ipnet::Ipv6Net;
use log::*;
use rtnetlink::new_connection;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use tokio::signal::unix::SignalKind;

use wireguard_uapi::set::{AllowedIp, Peer, WgDeviceF};

use crate::common::{
    add_address, address_from_public_key, base64_to_key, set_link_up, WgInterface, WgKey, WgMaestro,
};

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
        let ll_address = self.wg.ll_address()?;
        let device = self.wg.get_device().ok().unwrap();
        info!(
            "Configured Wireguard interface (received public key {})",
            base64::encode(device.public_key.unwrap())
        );

        info!("Setting up netlink route connection...");
        let (connection, handle, _) = new_connection().unwrap();
        tokio::spawn(connection);

        debug!("Setting Wireguard link-local address to {}", ll_address);
        let ll_net = Ipv6Net::new(ll_address, 64)?;
        add_address(&device.ifname, ll_net, handle.clone()).await?;
        set_link_up(&device.ifname, handle).await?;

        info!("Starting client...");
        debug!("Updating peers...");
        let server = &self.config.server;
        let addrs = format!("{}:{}", server.host, server.wireguard_port)
            .to_socket_addrs()?
            .collect::<Vec<SocketAddr>>();
        let server_ll_address = IpAddr::V6(address_from_public_key(&server.public_key).unwrap());
        let device =
            self.wg
                .build_set_device()
                .peers(vec![Peer::from_public_key(&server.public_key)
                    .endpoint(addrs.first().expect("Couldn't find host."))
                    .allowed_ips(vec![AllowedIp::from_ipaddr(&server_ll_address)])
                    .persistent_keepalive_interval(10)]);
        self.wg.set_device(device)?;

        loop {
            if let Ok(signal) = signal_receiver.recv() {
                info!("Received signal: {:?}", signal);
                self.should_exit = true;
            }
            if self.should_exit {
                debug!("Exiting...");
                return self.cleanup().await;
            }
            self.do_loop()?;
        }
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

        if let Some(fwmark) = config.fwmark {
            device = device.fwmark(fwmark)
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
