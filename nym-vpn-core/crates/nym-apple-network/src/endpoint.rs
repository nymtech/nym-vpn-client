// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use core::fmt;
use std::{
    ffi::{c_char, CStr, CString},
    net::{SocketAddr, SocketAddrV4, SocketAddrV6},
    path::PathBuf,
    ptr::NonNull,
};

use nix::sys::socket::{
    AddressFamily, SockaddrIn, SockaddrIn6, SockaddrLike, SockaddrStorage, UnixAddr,
};

use crate::sys::OS_nw_endpoint;

use super::{rc::Retained, sys};
pub use sys::nw_endpoint_type_t;

/// A local or remote endpoint in a network connection.
#[derive(Debug)]
pub enum Endpoint {
    /// Invalid endpoint.
    Invalid,

    /// Address endpoint.
    Address(AddressEndpoint),

    /// Host endpoint.
    Host(HostEndpoint),

    /// Bonjour service endpoint.
    BonjourService(BonjourServiceEndpoint),

    /// URL endpoint.
    Url(UrlEndpoint),

    /// An endpoint unknown to the crate.
    Unknown(UnknownEndpoint),
}

unsafe impl Send for Endpoint {}

impl Endpoint {
    /// Create new `Endpoint` retaining the raw pointer that we don't own.
    pub(crate) fn retain(nw_endpoint_ref: NonNull<sys::OS_nw_endpoint>) -> Self {
        match get_endpoint_type(nw_endpoint_ref) {
            EndpointType::Address => Self::Address(AddressEndpoint::retain(nw_endpoint_ref)),
            EndpointType::Host => Self::Host(HostEndpoint::retain(nw_endpoint_ref)),
            EndpointType::Url => Self::Url(UrlEndpoint::retain(nw_endpoint_ref)),
            EndpointType::BonjourService => {
                Self::BonjourService(BonjourServiceEndpoint::retain(nw_endpoint_ref))
            }
            EndpointType::Unknown(_) => Endpoint::Unknown(UnknownEndpoint::retain(nw_endpoint_ref)),
            EndpointType::Invalid => Self::Invalid,
        }
    }
}

/// An endpoint that couldn't be parsed or unknown to the crate.
#[repr(transparent)]
#[derive(Debug)]
#[allow(unused)]
pub struct UnknownEndpoint {
    inner: Retained<sys::OS_nw_endpoint>,
}

unsafe impl Send for UnknownEndpoint {}

impl UnknownEndpoint {
    pub(crate) fn retain(nw_endpoint_ref: NonNull<sys::OS_nw_endpoint>) -> Self {
        Self {
            inner: unsafe { Retained::retain(nw_endpoint_ref.as_ptr()) }
                .expect("failed to retain unknown endpoint"),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum EndpointType {
    Invalid,
    Address,
    Host,
    BonjourService,
    Url,
    Unknown(nw_endpoint_type_t),
}

impl From<nw_endpoint_type_t> for EndpointType {
    fn from(value: nw_endpoint_type_t) -> Self {
        match value {
            sys::nw_endpoint_type_invalid => Self::Invalid,
            sys::nw_endpoint_type_address => Self::Address,
            sys::nw_endpoint_type_host => Self::Host,
            sys::nw_endpoint_type_bonjour_service => Self::BonjourService,
            sys::nw_endpoint_type_url => Self::Url,
            other => Self::Unknown(other),
        }
    }
}

/// An endpoint represented as a host and port.
#[repr(transparent)]
#[derive(Debug)]
pub struct HostEndpoint {
    inner: Retained<sys::OS_nw_endpoint>,
}

unsafe impl Send for HostEndpoint {}

impl HostEndpoint {
    pub fn new(host: &str, port: u16) -> Result<Self> {
        let host_str =
            CString::new(host).map_err(|_| Error::FieldContainsNulByte(FieldName::Host))?;
        let port_str =
            CString::new(port.to_string()).expect("failed to create port string from u16");
        let nw_endpoint_ref =
            unsafe { sys::nw_endpoint_create_host(host_str.as_ptr(), port_str.as_ptr()) };

        Ok(Self {
            inner: unsafe { Retained::from_raw(nw_endpoint_ref) }.ok_or(Error::CreateEndpoint)?,
        })
    }

    /// Create new `HostEndpoint` retaining the raw pointer that we don't own.
    pub(crate) fn retain(nw_endpoint_ref: NonNull<sys::OS_nw_endpoint>) -> Self {
        debug_assert!(get_endpoint_type(nw_endpoint_ref) == EndpointType::Host);

        Self {
            inner: unsafe { Retained::retain(nw_endpoint_ref.as_ptr()) }
                .expect("failed to retain host endpoint"),
        }
    }

    pub fn host(&self) -> Result<String> {
        cstr_to_owned_string(unsafe { sys::nw_endpoint_get_hostname(self.inner.as_mut_ptr()) })
    }

    pub fn port(&self) -> u16 {
        unsafe { sys::nw_endpoint_get_port(self.inner.as_mut_ptr()) }
    }
}

/// An endpoint represented as an IP address and port.
#[repr(transparent)]
#[derive(Debug)]
pub struct AddressEndpoint {
    inner: Retained<sys::OS_nw_endpoint>,
}

/// An address held in address endpoint.
#[derive(Debug, Eq, PartialEq)]
pub enum Address {
    /// IP address and port.
    SocketAddr(SocketAddr),

    /// Unix path.
    Unix(PathBuf),
}

impl Address {
    unsafe fn from_raw(sockaddr: NonNull<nix::libc::sockaddr>) -> Result<Self> {
        let raw_address_family = i32::from(unsafe { (*sockaddr.as_ptr()).sa_family });

        match AddressFamily::from_i32(raw_address_family) {
            Some(AddressFamily::Inet) => SockaddrIn::from_raw(sockaddr.as_ptr(), None)
                .ok_or(Error::ConvertSocketAddr)
                .map(|sin| Address::SocketAddr(SocketAddr::V4(SocketAddrV4::from(sin)))),
            Some(AddressFamily::Inet6) => SockaddrIn6::from_raw(sockaddr.as_ptr(), None)
                .ok_or(Error::ConvertSocketAddr)
                .map(|sin6| Address::SocketAddr(SocketAddr::V6(SocketAddrV6::from(sin6)))),
            Some(AddressFamily::Unix) => UnixAddr::from_raw(sockaddr.as_ptr(), None)
                .ok_or(Error::ConvertSocketAddr)
                .map(|unix_addr| {
                    unix_addr
                        .path()
                        .map(|path| path.to_owned())
                        .unwrap_or_default()
                })
                .map(Address::Unix),
            _ => Err(Error::UnsupportedAddressFamily(raw_address_family)),
        }
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SocketAddr(addr) => addr.fmt(f),
            Self::Unix(addr) => write!(f, "{}", addr.display()),
        }
    }
}

unsafe impl Send for AddressEndpoint {}

impl AddressEndpoint {
    /// Creates an address from `Address`.
    pub fn new(addr: Address) -> Result<Self> {
        match addr {
            Address::SocketAddr(socket_addr) => Self::new_with_socket_addr(socket_addr),
            Address::Unix(unix_path) => Self::new_with_unix_path(&unix_path),
        }
    }

    /// Creates an address endpoint holding `SocketAddr`.
    pub fn new_with_socket_addr(socket_addr: SocketAddr) -> Result<Self> {
        let sockaddr_storage = SockaddrStorage::from(socket_addr);
        let nw_endpoint_ref = unsafe { sys::nw_endpoint_create_address(sockaddr_storage.as_ptr()) };

        Ok(Self {
            inner: unsafe { Retained::from_raw(nw_endpoint_ref) }.ok_or(Error::CreateEndpoint)?,
        })
    }

    /// Creates an address endpoint holding unix path.
    pub fn new_with_unix_path<P: ?Sized + nix::NixPath>(path: &P) -> Result<Self> {
        let unix_addr = UnixAddr::new(path).map_err(Error::CreateUnixAddr)?;
        let nw_endpoint_ref = unsafe { sys::nw_endpoint_create_address(unix_addr.as_ptr().cast()) };

        Ok(Self {
            inner: unsafe { Retained::from_raw(nw_endpoint_ref) }.ok_or(Error::CreateEndpoint)?,
        })
    }

    /// Create new `AddressEndpoint` retaining the raw pointer that we don't own.
    pub(crate) fn retain(nw_endpoint_ref: NonNull<sys::OS_nw_endpoint>) -> Self {
        debug_assert!(get_endpoint_type(nw_endpoint_ref) == EndpointType::Address);

        Self {
            inner: unsafe { Retained::retain(nw_endpoint_ref.as_ptr()) }
                .expect("failed ot retain address endpoint"),
        }
    }

    pub fn address(&self) -> Result<Address> {
        let sockaddr = unsafe { sys::nw_endpoint_get_address(self.inner.as_mut_ptr()) };

        NonNull::new(sockaddr.cast_mut())
            .ok_or(Error::InvalidSocketAddr)
            .and_then(|sockaddr| unsafe { Address::from_raw(sockaddr) })
    }

    pub fn port(&self) -> u16 {
        unsafe { sys::nw_endpoint_get_port(self.inner.as_mut_ptr()) }
    }
}

/// An endpoint represented as a URL, with host and port values inferred from the URL.
#[repr(transparent)]
#[derive(Debug)]
pub struct UrlEndpoint {
    inner: Retained<sys::OS_nw_endpoint>,
}

unsafe impl Send for UrlEndpoint {}

impl UrlEndpoint {
    pub fn new(url: &str) -> Result<Self> {
        let url_str = CString::new(url).map_err(|_| Error::FieldContainsNulByte(FieldName::Url))?;
        let nw_endpoint_ref = unsafe { sys::nw_endpoint_create_url(url_str.as_ptr()) };

        Ok(Self {
            inner: unsafe { Retained::from_raw(nw_endpoint_ref) }.ok_or(Error::CreateEndpoint)?,
        })
    }

    /// Create new `UrlEndpoint` retaining the raw pointer that we don't own.
    pub(crate) fn retain(nw_endpoint_ref: NonNull<sys::OS_nw_endpoint>) -> Self {
        debug_assert!(get_endpoint_type(nw_endpoint_ref) == EndpointType::Url);

        Self {
            inner: unsafe { Retained::retain(nw_endpoint_ref.as_ptr()) }
                .expect("failed ot retain url endpoint"),
        }
    }

    pub fn url(&self) -> Result<String> {
        cstr_to_owned_string(unsafe { sys::nw_endpoint_get_url(self.inner.as_mut_ptr()) })
    }
}

/// An endpoint represented as a Bonjour service.
#[repr(transparent)]
#[derive(Debug)]
pub struct BonjourServiceEndpoint {
    inner: Retained<sys::OS_nw_endpoint>,
}

unsafe impl Send for BonjourServiceEndpoint {}

impl BonjourServiceEndpoint {
    pub fn new(name: &str, service_type: &str, domain: &str) -> Result<Self> {
        let name_cstr =
            CString::new(name).map_err(|_| Error::FieldContainsNulByte(FieldName::Name))?;
        let service_cstr = CString::new(service_type)
            .map_err(|_| Error::FieldContainsNulByte(FieldName::Service))?;
        let domain_cstr =
            CString::new(domain).map_err(|_| Error::FieldContainsNulByte(FieldName::Domain))?;

        let nw_endpoint_ref = unsafe {
            sys::nw_endpoint_create_bonjour_service(
                name_cstr.as_ptr(),
                service_cstr.as_ptr(),
                domain_cstr.as_ptr(),
            )
        };

        Ok(Self {
            inner: unsafe { Retained::from_raw(nw_endpoint_ref) }.ok_or(Error::CreateEndpoint)?,
        })
    }

    /// Create new `BonjourServiceEndpoint` retaining the raw pointer that we don't own.
    pub(crate) fn retain(nw_endpoint_ref: NonNull<sys::OS_nw_endpoint>) -> Self {
        debug_assert!(get_endpoint_type(nw_endpoint_ref) == EndpointType::BonjourService);

        Self {
            inner: unsafe { Retained::retain(nw_endpoint_ref.as_ptr()) }
                .expect("failed ot retain bonjour service endpoint"),
        }
    }

    pub fn name(&self) -> Result<String> {
        cstr_to_owned_string(unsafe {
            sys::nw_endpoint_get_bonjour_service_name(self.inner.as_mut_ptr())
        })
    }

    pub fn domain(&self) -> Result<String> {
        cstr_to_owned_string(unsafe {
            sys::nw_endpoint_get_bonjour_service_domain(self.inner.as_mut_ptr())
        })
    }

    pub fn service_type(&self) -> Result<String> {
        cstr_to_owned_string(unsafe {
            sys::nw_endpoint_get_bonjour_service_type(self.inner.as_mut_ptr())
        })
    }
}

fn get_endpoint_type(nw_endpoint_ref: NonNull<OS_nw_endpoint>) -> EndpointType {
    EndpointType::from(unsafe { sys::nw_endpoint_get_type(nw_endpoint_ref.as_ptr()) })
}

fn cstr_to_owned_string(ptr: *const c_char) -> Result<String> {
    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .map(|s| s.to_owned())
        .map_err(Error::DecodeUtf8)
}

#[derive(Debug)]
pub enum FieldName {
    Url,
    Name,
    Host,
    Service,
    Domain,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to create endpoint due to invalid data")]
    CreateEndpoint,

    #[error("Failed to create unix address")]
    CreateUnixAddr(#[source] nix::errno::Errno),

    #[error("Failed to decode UTF-8 string")]
    DecodeUtf8(std::str::Utf8Error),

    #[error("Failed to parse IP address")]
    ParseIpAddress(std::net::AddrParseError),

    #[error("{:?} contains nul byte", _0)]
    FieldContainsNulByte(FieldName),

    #[error("Invalid socket address was returned")]
    InvalidSocketAddr,

    #[error("Failure to convert socket address to rust representation")]
    ConvertSocketAddr,

    #[error("Unsupported address family: {0}")]
    UnsupportedAddressFamily(i32),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use super::*;

    #[test]
    fn create_host_endpoint() {
        let ep = HostEndpoint::new("nymtech.net", 443).unwrap();
        assert_eq!(ep.host().unwrap(), "nymtech.net");
        assert_eq!(ep.port(), 443);
    }

    #[test]
    fn create_invalid_host_endpoint() {
        assert!(HostEndpoint::new("", 0).is_err());
    }

    #[test]
    fn create_address_endpoint() {
        let ep = AddressEndpoint::new_with_socket_addr(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            443,
        ))
        .unwrap();
        assert_eq!(
            ep.address().unwrap(),
            Address::SocketAddr(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 443)))
        );
        assert_eq!(ep.port(), 443);
    }

    #[test]
    fn create_unix_address_endpoint() {
        let ep = AddressEndpoint::new_with_unix_path("/var/run/mysock").unwrap();
        assert_eq!(
            ep.address().unwrap(),
            Address::Unix(PathBuf::from("/var/run/mysock"))
        );
        assert_eq!(ep.port(), 0);
    }

    #[test]
    fn create_url_endpoint() {
        let ep = UrlEndpoint::new("https://nymtech.net").unwrap();
        assert_eq!(ep.url().unwrap(), "https://nymtech.net");
    }

    #[test]
    fn create_invalid_url_endpoint() {
        assert!(UrlEndpoint::new("").is_err());
    }

    #[test]
    fn create_bonjour_endpoint() {
        let ep =
            BonjourServiceEndpoint::new("apple._music._tcp.local", "_music._tcp", "local").unwrap();
        assert_eq!(ep.name().unwrap(), "apple._music._tcp.local");
        assert_eq!(ep.service_type().unwrap(), "_music._tcp");
        assert_eq!(ep.domain().unwrap(), "local");
    }

    #[test]
    fn create_invalid_bonjour_endpoint() {
        assert!(BonjourServiceEndpoint::new("", "", "").is_err());
    }
}
