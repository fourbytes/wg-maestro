use failure;
use log::*;
use wireguard_uapi::{ DeviceInterface, WgSocket, RouteSocket };
use wireguard_uapi::{ get, set, err };

pub trait WgMaestro {
    fn start(&mut self);
}

pub struct WgInterface<'a> {
    wg_socket: WgSocket,
    route_socket: RouteSocket,
    wg_device_interface: DeviceInterface<'a>
}

impl<'a> WgInterface<'a> {
    pub fn from_name(ifname: String) -> Result<Self, failure::Error> {
        trace!("Connecting to Wireguard and routing sockets.");
        let mut wg_socket = WgSocket::connect()?;
        let mut route_socket = RouteSocket::connect()?;

        route_socket.add_device(&ifname)?;
        debug!("Created device @ {:}", ifname);

        let device = set::Device::from_ifname(ifname.clone())
            .flags(vec![set::WgDeviceF::ReplacePeers]);
        let wg_device_interface = device.interface.clone();

        debug!("Setting up device with {:?}", &device);
        wg_socket.set_device(device)?;

        
        Ok(Self {
            wg_socket,
            route_socket,
            wg_device_interface
        })
    }

    pub fn get_device(&mut self) -> get::Device {
        self.wg_socket
            .get_device(self.wg_device_interface.clone())
            .unwrap()
    }

    pub fn set_port(&mut self, listen_port: Option<u16>) -> Result<(), err::SetDeviceError> {
        let device = set::Device {
            flags: vec![],
            fwmark: None,
            interface: self.wg_device_interface.clone(),
            private_key: None,
            listen_port,
            peers: vec![]
        };
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
