use tun::AsyncDevice;

/// Holds the remains of the mixnet or wireguard tunnel.
#[derive(Default)]
pub struct Tombstone {
    /// Tunnel devices that are no longer in use by the tunnel.
    pub tun_devices: Vec<tun::AsyncDevice>,

    /// Wireguard tunnels that have not been shutdown yet because they own the tunnel device.
    /// These tunnels are kept around until after the routing table is reset. Windows only.
    #[cfg(windows)]
    pub wg_instances: Vec<nym_wg_go::wireguard_go::Tunnel>,
}

impl Tombstone {
    pub fn with_tun_device(tun_device: AsyncDevice) -> Self {
        Self {
            tun_devices: vec![tun_device],
            ..Default::default()
        }
    }
}
