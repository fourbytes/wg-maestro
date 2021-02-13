use anyhow::Result;
use async_trait::async_trait;
use byteorder::{BigEndian, ByteOrder};
use crossbeam_channel::Receiver;
use futures::stream::TryStreamExt;
use ipnet::Ipv6Net;
use ipnetwork::{IpNetwork, Ipv6Network};
use log::*;
use rtnetlink::{new_connection, Error, Handle};
use serde::de::{self, Deserializer};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::net::Ipv6Addr;
use tokio::signal::unix::SignalKind;
use wireguard_uapi::{err, get, set};
use wireguard_uapi::{DeviceInterface, RouteSocket, WgSocket};

pub async fn set_link_up(link_name: &str, handle: Handle) -> Result<(), Error> {
    let mut links = handle
        .link()
        .get()
        .set_name_filter(link_name.to_string())
        .execute();
    if let Some(link) = links.try_next().await? {
        handle.link().set(link.header.index).up().execute().await?
    }
    Ok(())
}

pub async fn add_address(link_name: &str, ip: Ipv6Net, handle: Handle) -> Result<(), Error> {
    let ip = Ipv6Network::from(ip.addr());
    let mut links = handle
        .link()
        .get()
        .set_name_filter(link_name.to_string())
        .execute();
    if let Some(link) = links.try_next().await? {
        handle
            .address()
            .add(
                link.header.index,
                std::net::IpAddr::V6(ip.ip()),
                ip.prefix(),
            )
            .execute()
            .await?
    }
    Ok(())
}

pub type WgKey = [u8; 32];
pub fn base64_to_key<'de, D>(deserializer: D) -> Result<WgKey, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = de::Deserialize::deserialize(deserializer)?;
    match base64::decode(s) {
        Ok(data) => {
            let mut key = [0u8; 32];
            key.copy_from_slice(&data);
            Ok(key)
        }
        Err(err) => Err(de::Error::custom(format!(
            "Failed to decode base64 string: {:?}",
            err
        ))),
    }
}

pub fn address_from_public_key(key: &WgKey) -> Result<Ipv6Addr> {
    // Replace DefaultHasher with a more robust solution.
    let hash = {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
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
    Ok(Ipv6Addr::new(
        0xfe80, 0, 0, 0, data[0], data[1], data[2], data[3],
    ))
}

#[async_trait]
pub trait WgMaestro {
    async fn run(&mut self, signal_receiver: Receiver<SignalKind>) -> anyhow::Result<()>;
    async fn cleanup(&mut self) -> anyhow::Result<()>;
}

pub struct WgInterface<'a> {
    wg_socket: WgSocket,
    route_socket: RouteSocket,
    wg_device_interface: DeviceInterface<'a>,
    device: get::Device,
}

impl<'a> WgInterface<'a> {
    pub fn from_name(ifname: String) -> Result<Self> {
        trace!("Connecting to Wireguard and routing sockets.");
        let mut wg_socket = WgSocket::connect()?;
        let mut route_socket = RouteSocket::connect()?;

        match route_socket.add_device(&ifname) {
            Ok(()) => debug!("Successfuly created device @ {:}", ifname),
            Err(err) => warn!(
                "Failed to create interface @ {:?} ({:?}). Trying to continue anyway...",
                ifname, err
            ),
        };

        let wg_device_interface = DeviceInterface::from_name(ifname);
        let device = wg_socket.get_device(wg_device_interface.clone())?;
        trace!("Retrieved initial device data: {:?}", device);

        let wg_device_interface = DeviceInterface::from_index(device.ifindex);

        Ok(Self {
            wg_socket,
            route_socket,
            wg_device_interface,
            device,
        })
    }

    pub fn get_public_key(&self) -> [u8; 32] {
        self.device.public_key.unwrap()
    }

    pub fn ll_address(&self) -> Result<Ipv6Addr> {
        address_from_public_key(&self.device.public_key.unwrap())
    }

    /* pub async fn setup_address(
        &mut self,
        addr: IpNet,
        scope: WireGuardDeviceAddrScope,
    ) -> Result<()> {
        // self.route_socket.add_addr(&ifname, addr, scope)?;
        let args = &[
            "addr",
            "add",
            &addr.to_string(),
            "dev",
            &self.device.ifname,
            "scope",
            match scope {
                WireGuardDeviceAddrScope::Link => "link",
                WireGuardDeviceAddrScope::Universe => "universe",
                WireGuardDeviceAddrScope::Site => "site",
                WireGuardDeviceAddrScope::Host => "host",
                WireGuardDeviceAddrScope::Nowhere => "nowhere",
            },
        ];
        let status = Command::new("ip").args(args).status().await?;

        // trace!("Spawning command: {}", command);

        Ok(())
    } */

    pub fn cleanup(&mut self) -> Result<()> {
        self.route_socket.del_device(&self.device.ifname)?;
        Ok(())
    }

    pub fn get_device(&mut self) -> Result<&get::Device, err::GetDeviceError> {
        self.device = self
            .wg_socket
            .get_device(self.wg_device_interface.clone())?;
        Ok(&self.device)
    }

    pub fn set_device(&mut self, device: set::Device) -> Result<(), err::SetDeviceError> {
        trace!("Setting Wireguard device: {:?}", device);
        self.wg_socket.set_device(device)
    }

    pub fn build_set_device(&self) -> set::Device<'a> {
        set::Device {
            flags: vec![],
            fwmark: Some(self.device.fwmark.clone()),
            interface: self.wg_device_interface.clone(),
            private_key: None,
            listen_port: Some(self.device.listen_port.clone()),
            peers: vec![],
        }
    }

    pub fn set_port(&mut self, listen_port: u16) -> Result<(), err::SetDeviceError> {
        let device = self.build_set_device().listen_port(listen_port);
        self.wg_socket.set_device(device)
    }

    pub fn set_private_key(&mut self, private_key: &[u8; 32]) -> Result<(), err::SetDeviceError> {
        let device = self.build_set_device().private_key(private_key);
        self.wg_socket.set_device(device)
    }
}
