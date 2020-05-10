use anyhow::Error;
use log::*;
use wireguard_uapi::{ DeviceInterface, WgSocket, RouteSocket };
use wireguard_uapi::{ get, set, err };

pub trait WgMaestro {
    fn start(&mut self);
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

        route_socket.add_device(&ifname)?;
        debug!("Successfuly created device @ {:}", ifname);

        let wg_device_interface = DeviceInterface::from_name(ifname);
        let device = wg_socket.get_device(wg_device_interface.clone())?;
        debug!("Retrieved device data: {:?}", device);
        
        Ok(Self {
            wg_socket,
            route_socket,
            wg_device_interface,
            device
        })
    }

    pub fn get_device(&mut self) -> Result<get::Device, err::GetDeviceError> {
        self.wg_socket
            .get_device(self.wg_device_interface.clone())
    }

    fn build_set_device(wg_device_interface: DeviceInterface) -> set::Device {
        set::Device {
            flags: vec![],
            fwmark: None,
            interface: wg_device_interface,
            private_key: None,
            listen_port: None,
            peers: vec![]
        }
    }

    pub fn set_port(&mut self, listen_port: u16) -> Result<(), err::SetDeviceError> {
        let device = Self::build_set_device(self.wg_device_interface.clone())
            .listen_port(listen_port);
        self.wg_socket.set_device(device)
    }

    pub fn set_private_key(&mut self, private_key: Option<&[u8; 32]>) -> Result<(), err::SetDeviceError> {
        let device = set::Device {
            flags: vec![],
            fwmark: None,
            interface: self.wg_device_interface.clone(),
            private_key,
            listen_port: None,
            peers: vec![]
        };
        self.wg_socket.set_device(device)
    }
}
