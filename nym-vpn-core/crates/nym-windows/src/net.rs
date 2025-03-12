// Copyright 2016-2024 Mullvad VPN AB. All Rights Reserved.
// Copyright 2024 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    ffi::{OsStr, OsString},
    fmt, io,
    mem::{self, MaybeUninit},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    os::windows::ffi::OsStringExt,
    sync::Mutex,
    time::{Duration, Instant},
};

use windows::{
    core::{GUID, HSTRING},
    Win32::{
        Foundation::{ERROR_NOT_FOUND, HANDLE},
        NetworkManagement::{
            IpHelper::{
                CancelMibChangeNotify2, ConvertInterfaceAliasToLuid, ConvertInterfaceLuidToAlias,
                ConvertInterfaceLuidToGuid, ConvertInterfaceLuidToIndex, CreateIpForwardEntry2,
                CreateUnicastIpAddressEntry, FreeMibTable, GetIpInterfaceEntry,
                GetUnicastIpAddressEntry, GetUnicastIpAddressTable, InitializeIpForwardEntry,
                InitializeUnicastIpAddressEntry, MibAddInstance, NotifyIpInterfaceChange,
                SetIpInterfaceEntry, MIB_IPINTERFACE_ROW, MIB_NOTIFICATION_TYPE,
                MIB_UNICASTIPADDRESS_ROW, MIB_UNICASTIPADDRESS_TABLE,
            },
            Ndis::{IF_MAX_STRING_SIZE, NET_LUID_LH},
        },
        Networking::WinSock::{
            IpDadStateDeprecated, IpDadStateDuplicate, IpDadStateInvalid, IpDadStatePreferred,
            IpDadStateTentative, NlroManual, ADDRESS_FAMILY, AF_INET, AF_INET6, AF_UNSPEC,
            IN6_ADDR, IN_ADDR, MIB_IPPROTO_NT_STATIC, NL_DAD_STATE, SOCKADDR_INET,
        },
    },
};

/// Result type for this module.
pub type Result<T> = std::result::Result<T, Error>;

const DAD_CHECK_TIMEOUT: Duration = Duration::from_secs(5);
const DAD_CHECK_INTERVAL: Duration = Duration::from_millis(100);

/// Errors returned by some functions in this module.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Error returned from `ConvertInterfaceAliasToLuid`
    #[cfg(windows)]
    #[error("Cannot find LUID for virtual adapter")]
    NoDeviceLuid(#[source] io::Error),

    /// Error returned from `GetUnicastIpAddressTable`/`GetUnicastIpAddressEntry`
    #[cfg(windows)]
    #[error("Failed to obtain unicast IP address table")]
    ObtainUnicastAddress(#[source] windows::core::Error),

    /// `GetUnicastIpAddressTable` contained no addresses for the interface
    #[cfg(windows)]
    #[error("Found no addresses for the given adapter")]
    NoUnicastAddress,

    /// Error returned from `CreateUnicastIpAddressEntry`
    #[cfg(windows)]
    #[error("Failed to create unicast IP address")]
    CreateUnicastEntry(#[source] windows::core::Error),

    /// Error returned from `CreateIpForwardEntry2`
    #[cfg(windows)]
    #[error("Failed to create IP forwarding entry")]
    CreateForwardEntry(#[source] windows::core::Error),

    /// Unexpected DAD state returned for a unicast address
    #[cfg(windows)]
    #[error("Unexpected DAD state")]
    DadStateError(#[source] DadStateError),

    /// DAD check failed.
    #[cfg(windows)]
    #[error("Timed out waiting on tunnel device")]
    DeviceReadyTimeout,

    /// Unicast DAD check fail.
    #[cfg(windows)]
    #[error("Unicast channel sender was unexpectedly dropped")]
    UnicastSenderDropped,

    /// Unknown address family
    #[error("Unknown address family: {0}")]
    UnknownAddressFamily(u16),
}

/// Handles cases where there DAD state is neither tentative nor preferred.
#[derive(thiserror::Error, Debug)]
pub enum DadStateError {
    /// Invalid DAD state.
    #[error("Invalid DAD state")]
    Invalid,

    /// Duplicate unicast address.
    #[error("A duplicate IP address was detected")]
    Duplicate,

    /// Deprecated unicast address.
    #[error("The IP address has been deprecated")]
    Deprecated,

    /// Unknown DAD state constant.
    #[error("Unknown DAD state: {0}")]
    Unknown(i32),
}

#[allow(non_upper_case_globals)]
impl From<NL_DAD_STATE> for DadStateError {
    fn from(state: NL_DAD_STATE) -> DadStateError {
        match state {
            IpDadStateInvalid => DadStateError::Invalid,
            IpDadStateDuplicate => DadStateError::Duplicate,
            IpDadStateDeprecated => DadStateError::Deprecated,
            other => DadStateError::Unknown(other.0),
        }
    }
}

impl AddressFamily {
    /// Convert one of the `AF_*` constants to an [`AddressFamily`].
    pub fn try_from_af_family(family: u16) -> Result<AddressFamily> {
        match ADDRESS_FAMILY(family) {
            AF_INET => Ok(AddressFamily::Ipv4),
            AF_INET6 => Ok(AddressFamily::Ipv6),
            family => Err(Error::UnknownAddressFamily(family.0)),
        }
    }

    /// Convert an [`AddressFamily`] to one of the `AF_*` constants.
    pub fn to_af_family(&self) -> u16 {
        match self {
            Self::Ipv4 => AF_INET,
            Self::Ipv6 => AF_INET6,
        }
        .0
    }
}

/// Context for [`notify_ip_interface_change`]. When it is dropped,
/// the callback is unregistered.
pub struct IpNotifierHandle<'a> {
    #[allow(clippy::type_complexity)]
    callback: Mutex<Box<dyn FnMut(&MIB_IPINTERFACE_ROW, MIB_NOTIFICATION_TYPE) + Send + 'a>>,
    handle: HANDLE,
}

unsafe impl Send for IpNotifierHandle<'_> {}

impl Drop for IpNotifierHandle<'_> {
    fn drop(&mut self) {
        if let Err(e) = unsafe { CancelMibChangeNotify2(self.handle) }.ok() {
            tracing::error!("Failed to cancel ip notifier: {}", e);
        }
    }
}

unsafe extern "system" fn inner_callback(
    context: *const std::ffi::c_void,
    row: *const MIB_IPINTERFACE_ROW,
    notify_type: MIB_NOTIFICATION_TYPE,
) {
    let context = &mut *(context as *mut IpNotifierHandle<'_>);
    context
        .callback
        .lock()
        .expect("NotifyIpInterfaceChange mutex poisoned")(&*row, notify_type);
}

/// Registers a callback function that is invoked when an interface is added, removed,
/// or changed.
pub fn notify_ip_interface_change<
    'a,
    T: FnMut(&MIB_IPINTERFACE_ROW, MIB_NOTIFICATION_TYPE) + Send + 'a,
>(
    callback: T,
    family: Option<AddressFamily>,
) -> windows::core::Result<Box<IpNotifierHandle<'a>>> {
    let mut context = Box::new(IpNotifierHandle {
        callback: Mutex::new(Box::new(callback)),
        handle: HANDLE::default(),
    });

    unsafe {
        NotifyIpInterfaceChange(
            af_family_from_family(family),
            Some(inner_callback),
            Some(&*context as *const _ as *const _),
            false,
            (&mut context.handle) as *mut _,
        )
    }
    .ok()?;

    Ok(context)
}

/// Returns information about a network IP interface.
pub fn get_ip_interface_entry(
    family: AddressFamily,
    luid: &NET_LUID_LH,
) -> windows::core::Result<MIB_IPINTERFACE_ROW> {
    let mut row: MIB_IPINTERFACE_ROW = unsafe { mem::zeroed() };
    row.Family = ADDRESS_FAMILY(family as u16);
    row.InterfaceLuid = *luid;

    unsafe { GetIpInterfaceEntry(&mut row) }.ok()?;

    Ok(row)
}

/// Set the properties of an IP interface.
pub fn set_ip_interface_entry(row: &mut MIB_IPINTERFACE_ROW) -> windows::core::Result<()> {
    unsafe { SetIpInterfaceEntry(row as *mut _) }.ok()
}

fn ip_interface_entry_exists(
    family: AddressFamily,
    luid: &NET_LUID_LH,
) -> windows::core::Result<bool> {
    match get_ip_interface_entry(family, luid) {
        Ok(_) => Ok(true),
        Err(error) if error.code() == ERROR_NOT_FOUND.to_hresult() => Ok(false),
        Err(error) => Err(error),
    }
}

/// Waits until the specified IP interfaces have attached to a given network interface.
pub async fn wait_for_interfaces(luid: NET_LUID_LH, ipv4: bool, ipv6: bool) -> io::Result<()> {
    let (tx, rx) = futures::channel::oneshot::channel();

    let mut found_ipv4 = !ipv4;
    let mut found_ipv6 = !ipv6;

    let mut tx = Some(tx);

    let _handle = notify_ip_interface_change(
        move |row, notification_type| {
            if found_ipv4 && found_ipv6 {
                return;
            }
            if notification_type != MibAddInstance {
                return;
            }
            if unsafe { row.InterfaceLuid.Value != luid.Value } {
                return;
            }
            match row.Family {
                AF_INET => found_ipv4 = true,
                AF_INET6 => found_ipv6 = true,
                _ => (),
            }
            if found_ipv4 && found_ipv6 {
                if let Some(tx) = tx.take() {
                    let _ = tx.send(());
                }
            }
        },
        None,
    )?;

    // Make sure they don't already exist
    if (!ipv4 || ip_interface_entry_exists(AddressFamily::Ipv4, &luid)?)
        && (!ipv6 || ip_interface_entry_exists(AddressFamily::Ipv6, &luid)?)
    {
        return Ok(());
    }

    let _ = rx.await;
    Ok(())
}

/// Wait for addresses to be usable on an network adapter.
pub async fn wait_for_addresses(luid: NET_LUID_LH) -> Result<()> {
    // Obtain unicast IP addresses
    let mut unicast_rows: Vec<MIB_UNICASTIPADDRESS_ROW> = get_unicast_table(None)
        .map_err(Error::ObtainUnicastAddress)?
        .into_iter()
        .filter(|row| unsafe { row.InterfaceLuid.Value == luid.Value })
        .collect();
    if unicast_rows.is_empty() {
        return Err(Error::NoUnicastAddress);
    }

    let (tx, rx) = futures::channel::oneshot::channel();
    let mut addr_check_thread = move || {
        // Poll DAD status using GetUnicastIpAddressEntry
        // https://docs.microsoft.com/en-us/windows/win32/api/netioapi/nf-netioapi-createunicastipaddressentry

        let deadline = Instant::now() + DAD_CHECK_TIMEOUT;
        while Instant::now() < deadline {
            let mut ready = true;

            for row in &mut unicast_rows {
                unsafe { GetUnicastIpAddressEntry(row) }
                    .ok()
                    .map_err(Error::ObtainUnicastAddress)?;
                if row.DadState == IpDadStateTentative {
                    ready = false;
                    break;
                }
                if row.DadState != IpDadStatePreferred {
                    return Err(Error::DadStateError(DadStateError::from(row.DadState)));
                }
            }

            if ready {
                return Ok(());
            }
            std::thread::sleep(DAD_CHECK_INTERVAL);
        }

        Err(Error::DeviceReadyTimeout)
    };
    std::thread::spawn(move || {
        let _ = tx.send(addr_check_thread());
    });
    rx.await.map_err(|_| Error::UnicastSenderDropped)?
}

/// Returns the first unicast IP address for the given interface.
pub fn get_ip_address_for_interface(
    family: AddressFamily,
    luid: NET_LUID_LH,
) -> Result<Option<IpAddr>> {
    match get_unicast_table(Some(family))
        .map_err(Error::ObtainUnicastAddress)?
        .into_iter()
        .find(|row| unsafe { row.InterfaceLuid.Value == luid.Value })
    {
        Some(row) => Ok(Some(try_socketaddr_from_inet_sockaddr(row.Address)?.ip())),
        None => Ok(None),
    }
}

/// Adds a unicast IP address for the given interface.
pub fn add_ip_address_for_interface(luid: NET_LUID_LH, address: IpAddr) -> Result<()> {
    let mut row = unsafe { mem::zeroed() };
    unsafe { InitializeUnicastIpAddressEntry(&mut row) };

    row.InterfaceLuid = luid;
    row.Address = SOCKADDR_INET::from(SocketAddr::new(address, 0));
    row.DadState = IpDadStatePreferred;
    row.OnLinkPrefixLength = 255;

    unsafe { CreateUnicastIpAddressEntry(&row) }
        .ok()
        .map_err(Error::CreateUnicastEntry)
}

/// Add default IPv4 gateway for the given interface.
pub fn add_default_ipv4_gateway_for_interface(luid: NET_LUID_LH, address: Ipv4Addr) -> Result<()> {
    let mut forward_row = unsafe { mem::zeroed() };
    unsafe { InitializeIpForwardEntry(&mut forward_row) };

    forward_row.InterfaceLuid = luid;
    forward_row.DestinationPrefix.Prefix.si_family = AF_INET;
    forward_row.DestinationPrefix.Prefix.Ipv4.sin_family = AF_INET;
    forward_row.NextHop.si_family = AF_INET;
    forward_row.NextHop.Ipv4.sin_family = AF_INET;
    forward_row.NextHop.Ipv4.sin_addr = IN_ADDR::from(address);
    forward_row.SitePrefixLength = 0;
    forward_row.Metric = 1;
    forward_row.Protocol = MIB_IPPROTO_NT_STATIC;
    forward_row.Origin = NlroManual;

    unsafe { CreateIpForwardEntry2(&forward_row) }
        .ok()
        .map_err(Error::CreateForwardEntry)
}

/// Add default IPv6 gateway for the given interface.
pub fn add_default_ipv6_gateway_for_interface(luid: NET_LUID_LH, address: Ipv6Addr) -> Result<()> {
    let mut forward_row = unsafe { mem::zeroed() };
    unsafe { InitializeIpForwardEntry(&mut forward_row) };

    forward_row.InterfaceLuid = luid;
    forward_row.DestinationPrefix.Prefix.si_family = AF_INET6;
    forward_row.DestinationPrefix.Prefix.Ipv6.sin6_family = AF_INET6;
    forward_row.NextHop.si_family = AF_INET6;
    forward_row.NextHop.Ipv6.sin6_family = AF_INET6;
    forward_row.NextHop.Ipv6.sin6_addr = IN6_ADDR::from(address);
    forward_row.SitePrefixLength = 0;
    forward_row.Metric = 1;
    forward_row.Protocol = MIB_IPPROTO_NT_STATIC;
    forward_row.Origin = NlroManual;

    unsafe { CreateIpForwardEntry2(&forward_row) }
        .ok()
        .map_err(Error::CreateForwardEntry)
}

/// Sets MTU on the specified network interface identified by `luid`.
pub fn set_mtu(mtu: u32, luid: NET_LUID_LH, ip_family: AddressFamily) -> windows::core::Result<()> {
    let mut row = get_ip_interface_entry(ip_family, &luid)?;

    row.NlMtu = mtu;

    set_ip_interface_entry(&mut row)
}

/// Returns the unicast IP address table. If `family` is `None`, then addresses for all families are
/// returned.
pub fn get_unicast_table(
    family: Option<AddressFamily>,
) -> windows::core::Result<Vec<MIB_UNICASTIPADDRESS_ROW>> {
    let mut unicast_rows = vec![];
    let mut unicast_table: *mut MIB_UNICASTIPADDRESS_TABLE = std::ptr::null_mut();

    unsafe { GetUnicastIpAddressTable(af_family_from_family(family), &mut unicast_table) }.ok()?;
    let first_row = unsafe { &(*unicast_table).Table[0] } as *const MIB_UNICASTIPADDRESS_ROW;
    for i in 0..unsafe { *unicast_table }.NumEntries {
        unicast_rows.push(unsafe { *(first_row.offset(i as isize)) });
    }
    unsafe { FreeMibTable(unicast_table as *const _) };

    Ok(unicast_rows)
}

/// Returns the index of a network interface given its LUID.
pub fn index_from_luid(luid: &NET_LUID_LH) -> io::Result<u32> {
    let mut index = 0u32;
    unsafe { ConvertInterfaceLuidToIndex(luid, &mut index) }.ok()?;
    Ok(index)
}

/// Returns the GUID of a network interface given its LUID.
pub fn guid_from_luid(luid: &NET_LUID_LH) -> io::Result<GUID> {
    let mut guid = MaybeUninit::zeroed();
    unsafe { ConvertInterfaceLuidToGuid(luid, guid.as_mut_ptr()) }.ok()?;
    Ok(unsafe { guid.assume_init() })
}

/// Returns the LUID of an interface given its alias.
pub fn luid_from_alias<T: AsRef<OsStr>>(alias: T) -> io::Result<NET_LUID_LH> {
    let mut luid: NET_LUID_LH = unsafe { std::mem::zeroed() };
    unsafe { ConvertInterfaceAliasToLuid(&HSTRING::from(alias.as_ref()), &mut luid) }.ok()?;
    Ok(luid)
}

/// Returns the alias of an interface given its LUID.
pub fn alias_from_luid(luid: &NET_LUID_LH) -> windows::core::Result<OsString> {
    let mut buffer = [0u16; IF_MAX_STRING_SIZE as usize + 1];
    unsafe { ConvertInterfaceLuidToAlias(luid, &mut buffer) }.ok()?;
    let nul = buffer.iter().position(|&c| c == 0u16).unwrap();
    Ok(OsString::from_wide(&buffer[0..nul]))
}

fn af_family_from_family(family: Option<AddressFamily>) -> ADDRESS_FAMILY {
    family
        .map(|family| ADDRESS_FAMILY(family as u16))
        .unwrap_or(AF_UNSPEC)
}

/// Converts a `SOCKADDR_INET` to `SocketAddr`. Returns an error if the address family is invalid.
pub fn try_socketaddr_from_inet_sockaddr(addr: SOCKADDR_INET) -> Result<SocketAddr> {
    let family = unsafe { addr.si_family };

    match family {
        AF_INET => {
            let ipv4_addr = Ipv4Addr::from(unsafe { addr.Ipv4.sin_addr });
            let port = u16::from_be(unsafe { addr.Ipv4.sin_port });

            Ok(SocketAddr::V4(SocketAddrV4::new(ipv4_addr, port)))
        }
        AF_INET6 => {
            let ipv6_addr = Ipv6Addr::from(unsafe { addr.Ipv6.sin6_addr });
            let port = u16::from_be(unsafe { addr.Ipv6.sin6_port });
            let flowinfo = u32::from_be(unsafe { addr.Ipv6.sin6_flowinfo });
            let scope_id = unsafe { addr.Ipv6.Anonymous.sin6_scope_id };

            Ok(SocketAddr::V6(SocketAddrV6::new(
                ipv6_addr, port, flowinfo, scope_id,
            )))
        }
        _ => Err(Error::UnknownAddressFamily(family.0)),
    }
}

/// Address family. These correspond to the `AF_*` constants.
#[derive(Debug, Clone, Copy)]
pub enum AddressFamily {
    /// IPv4 address family
    Ipv4 = AF_INET.0 as isize,
    /// IPv6 address family
    Ipv6 = AF_INET6.0 as isize,
}

impl fmt::Display for AddressFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            AddressFamily::Ipv4 => write!(f, "IPv4 (AF_INET)"),
            AddressFamily::Ipv6 => write!(f, "IPv6 (AF_INET6)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sockaddr_v4() {
        let addr_v4 = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(1, 2, 3, 4), 1234));
        assert_eq!(
            addr_v4,
            try_socketaddr_from_inet_sockaddr(SOCKADDR_INET::from(addr_v4)).unwrap()
        );
    }

    #[test]
    fn test_sockaddr_v6() {
        let addr_v6 = SocketAddr::V6(SocketAddrV6::new(
            Ipv6Addr::new(1, 2, 3, 4, 5, 6, 7, 8),
            1234,
            0xa,
            0xb,
        ));
        assert_eq!(
            addr_v6,
            try_socketaddr_from_inet_sockaddr(SOCKADDR_INET::from(addr_v6)).unwrap()
        );
    }
}
