use async_trait::async_trait;
use anyhow::{ Error, Result };
use log::*;
use base64;
use serde::de::{self, Deserializer};
use crossbeam_channel::Receiver;
use tokio::signal::unix::SignalKind;
use wireguard_uapi::{ DeviceInterface, WgSocket, RouteSocket };
use wireguard_uapi::{ get, set, err };

pub type WgKey = [u8; 32];
pub fn base64_to_key<'de, D>(deserializer: D) -> Result<WgKey, D::Error>
where D: Deserializer<'de> {
    let s: String = de::Deserialize::deserialize(deserializer)?;
    match base64::decode(s) {
        Ok(data) => {
            let mut key = [0u8; 32];
            key.copy_from_slice(&data);
            Ok(key)
        },
        Err(err) => Err(de::Error::custom(format!("Failed to decode base64 string: {:?}", err)))
    }
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
    device: get::Device
}

impl<'a> WgInterface<'a> {
    pub fn from_name(ifname: String) -> Result<Self, Error> {
        trace!("Connecting to Wireguard and routing sockets.");
        let mut wg_socket = WgSocket::connect()?;
        let mut route_socket = RouteSocket::connect()?;

        match route_socket.add_device(&ifname) {
            Ok(()) => debug!("Successfuly created device @ {:}", ifname),
            Err(err) => warn!("Failed to create interface @ {:?} ({:?}). Trying to continue anyway...", ifname, err)
        };

        let wg_device_interface = DeviceInterface::from_name(ifname);
        let device = wg_socket.get_device(wg_device_interface.clone())?;
        trace!("Retrieved initial device data: {:?}", device);
        
        let wg_device_interface = DeviceInterface::from_index(device.ifindex);

        Ok(Self {
            wg_socket,
            route_socket,
            wg_device_interface,
            device
        })
    }

    pub fn get_public_key(&self) -> [u8; 32] {
        self.device.public_key.unwrap()
    }

    pub fn cleanup(&mut self) -> Result<()> {
        self.route_socket.del_device(&self.device.ifname)?;
        Ok(())
    }

    pub fn get_device(&mut self) -> Result<&get::Device, err::GetDeviceError> {
        self.device = self.wg_socket
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
            peers: vec![]
        }
    }

    pub fn set_port(&mut self, listen_port: u16) -> Result<(), err::SetDeviceError> {
        let device = self.build_set_device()
            .listen_port(listen_port);
        self.wg_socket.set_device(device)
    }

    pub fn set_private_key(&mut self, private_key: &[u8; 32]) -> Result<(), err::SetDeviceError> {
        let device = self.build_set_device()
            .private_key(private_key);
        self.wg_socket.set_device(device)
    }
}
