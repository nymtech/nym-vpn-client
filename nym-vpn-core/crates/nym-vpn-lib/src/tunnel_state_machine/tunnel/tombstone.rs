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
    /// Convenience initializer that creates a tombstone with a single tunnel device.
    pub fn with_tun_device(tun_device: AsyncDevice) -> Self {
        Self::with_tun_devices(vec![tun_device])
    }

    /// Creates a tombstone with tunnel devices.
    pub fn with_tun_devices(tun_devices: Vec<AsyncDevice>) -> Self {
        Self {
            tun_devices,
            #[cfg(windows)]
            wg_instances: Vec::new(),
        }
    }

    /// Creates a tombstone with wireguard tunnel instances.
    #[cfg(windows)]
    pub fn with_wg_instances(wg_instances: Vec<nym_wg_go::wireguard_go::Tunnel>) -> Self {
        Self {
            tun_devices: Vec::new(),
            wg_instances,
        }
    }
}
