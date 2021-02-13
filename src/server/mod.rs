use std::net::{IpAddr, SocketAddr};

use anyhow::Result;
use async_trait::async_trait;
use crossbeam_channel::Receiver;
use ipnet::{IpNet, Ipv6Net};
use ipnetwork::Ipv6Network;
use log::*;
use rtnetlink::{new_connection, Error, Handle};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio::signal::unix::SignalKind;

use wireguard_uapi::{
    set::Peer,
    set::{AllowedIp, WgDeviceF},
};

use crate::common::{
    add_address, address_from_public_key, base64_to_key, set_link_up, WgInterface, WgKey, WgMaestro,
};

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
    // #[serde(deserialize_with = "base64_to_key")]
    // pre_shared_key: Option<WgKey>,
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

        debug!("Updating peers...");
        let clients: Vec<_> = self
            .config
            .clients
            .iter()
            .map(|client| {
                (
                    client.public_key,
                    IpAddr::V6(address_from_public_key(&client.public_key).unwrap()),
                )
            })
            .collect();
        let device = self
            .wg
            .build_set_device()
            .peers(vec![Peer::from_public_key(&clients[0].0)
                .allowed_ips(vec![AllowedIp::from_ipaddr(&clients[0].1)])
                .persistent_keepalive_interval(10)]);
        self.wg.set_device(device)?;

        let server_addr = format!("127.0.0.1:{}", self.config.maestro_port);
        info!("Starting server loop on {}", server_addr);
        self.listener = Some(TcpListener::bind(server_addr).await?);

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
        log::debug!("Cleaning up...");
        {
            // Shutdown the TCP stream
            if self.listener.take().is_some() {}
        }
        {
            // Remove the wireguard interface
            self.wg.cleanup()?;
        }
        Ok(())
    }
}

impl<'a> Server<'a> {
    pub fn new(config: ServerConfig) -> Result<Self> {
        debug!("Setting up server...");
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
            listener: None,
            should_exit: false,
        })
    }

    fn do_loop(&mut self) -> Result<()> {
        Ok(())
    }
}
