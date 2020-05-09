use failure;
use wireguard_uapi::{ DeviceInterface, WgSocket };
use wireguard_uapi::set::Device;

pub trait WgMaestro {
    fn start(&mut self);
}

pub struct WgInterface<'a> {
    wg_socket: WgSocket,
    wg_device_interface: DeviceInterface<'a>
}

impl<'a> WgInterface<'a> {
    pub fn new(interface_name: String) -> Result<Self, failure::Error> {
        let wg_socket = WgSocket::connect()?;
        let wg_device_interface = DeviceInterface::from_name(interface_name);

        Ok(Self {
            wg_socket,
            wg_device_interface
        })
    }
}
