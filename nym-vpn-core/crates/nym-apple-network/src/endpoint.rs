// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    ffi::{c_char, CStr, CString},
    net::{IpAddr, SocketAddr, SocketAddrV4, SocketAddrV6},
};

use nix::sys::socket::{AddressFamily, SockaddrLike, SockaddrStorage};
use objc2::rc::Retained;

use super::sys;
pub use sys::nw_endpoint_type_t;

#[derive(Debug)]
pub enum Endpoint {
    Invalid,
    Address(AddressEndpoint),
    Host(HostEndpoint),
    BonjourService(BonjourServiceEndpoint),
    Url(UrlEndpoint),
    Unknown(UnknownEndpoint),
}

impl Endpoint {
    pub(crate) fn retain(nw_endpoint_ref: sys::nw_endpoint_t) -> Option<Self> {
        Some(match get_endpoint_type(nw_endpoint_ref) {
            EndpointType::Address => Self::Address(
                AddressEndpoint::retain(nw_endpoint_ref)
                    .expect("failed to retain address endpoint"),
            ),
            EndpointType::Host => Self::Host(
                HostEndpoint::retain(nw_endpoint_ref).expect("failed to retain host endpoint"),
            ),
            EndpointType::Url => Self::Url(
                UrlEndpoint::retain(nw_endpoint_ref).expect("failed to retain url endpoint"),
            ),
            EndpointType::BonjourService => Self::BonjourService(
                BonjourServiceEndpoint::retain(nw_endpoint_ref)
                    .expect("failed to retain bonjour service endpoint"),
            ),
            EndpointType::Unknown(_) => Endpoint::Unknown(UnknownEndpoint(unsafe {
                Retained::from_raw(nw_endpoint_ref)
            })),
            EndpointType::Invalid => Self::Invalid,
        })
    }
}

/// Holds any endpoint that couldn't be parsed or unknown to the bindings.
/// This will make ensure to release the underlying handle properly.
#[derive(Debug)]
#[allow(unused)]
pub struct UnknownEndpoint(Option<Retained<sys::OS_nw_endpoint>>);

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

/// Endpoint that holds hostname and port.
#[derive(Debug)]
pub struct HostEndpoint {
    inner: Retained<sys::OS_nw_endpoint>,
}

impl HostEndpoint {
    pub fn new(host: &str, port: u16) -> Result<Self> {
        let host_str =
            CString::new(host).map_err(|_| Error::FieldContainsNulByte(FieldName::Host))?;
        let port_str =
            CString::new(port.to_string()).expect("failed to create port string from u16");
        let nw_endpoint_ref =
            unsafe { sys::nw_endpoint_create_host(host_str.as_ptr(), port_str.as_ptr()) };

        if nw_endpoint_ref.is_null() {
            Err(Error::InvalidEndpointData)
        } else {
            Ok(Self::retain(nw_endpoint_ref).expect("failed to retain nw_endpoint_ref"))
        }
    }

    pub(crate) fn retain(nw_endpoint_ref: sys::nw_endpoint_t) -> Option<Self> {
        if get_endpoint_type(nw_endpoint_ref) == EndpointType::Host {
            Some(Self {
                inner: unsafe { Retained::retain(nw_endpoint_ref)? },
            })
        } else {
            None
        }
    }

    pub fn host(&self) -> Result<String> {
        cstr_to_owned_string(unsafe { sys::nw_endpoint_get_hostname(self.as_raw_mut()) })
    }

    pub fn port(&self) -> u16 {
        unsafe { sys::nw_endpoint_get_port(self.as_raw_mut()) }
    }

    fn as_raw_mut(&self) -> sys::nw_endpoint_t {
        Retained::as_ptr(&self.inner).cast_mut()
    }
}

/// Endpoint that holds IP address and port.
#[derive(Debug)]
pub struct AddressEndpoint {
    inner: Retained<sys::OS_nw_endpoint>,
}

impl AddressEndpoint {
    pub fn new(addr: IpAddr, port: u16) -> Result<Self> {
        let sockaddr = SocketAddr::new(addr, port);
        let sockaddr_storage = SockaddrStorage::from(sockaddr);
        let nw_endpoint_ref = unsafe { sys::nw_endpoint_create_address(sockaddr_storage.as_ptr()) };

        if nw_endpoint_ref.is_null() {
            Err(Error::InvalidEndpointData)
        } else {
            Ok(Self::retain(nw_endpoint_ref).expect("failed to retain nw_endpoint_ref"))
        }
    }

    pub(crate) fn retain(nw_endpoint_ref: sys::nw_endpoint_t) -> Option<Self> {
        if get_endpoint_type(nw_endpoint_ref) == EndpointType::Address {
            Some(Self {
                inner: unsafe { Retained::retain(nw_endpoint_ref)? },
            })
        } else {
            None
        }
    }

    pub fn address(&self) -> Result<SocketAddr> {
        let sa_ptr = unsafe { sys::nw_endpoint_get_address(self.as_raw_mut()) };
        if sa_ptr.is_null() {
            Err(Error::NoSocketAddr)
        } else {
            let sockaddr_storage = unsafe { SockaddrStorage::from_raw(sa_ptr, None) }
                .ok_or(Error::SockAddrToAddrStorageConversion)?;

            match sockaddr_storage.family().ok_or(Error::NoAddrFamily)? {
                AddressFamily::Inet => sockaddr_storage
                    .as_sockaddr_in()
                    .ok_or(Error::SockAddrStorageToAddr)
                    .map(|sin| SocketAddr::V4(SocketAddrV4::from(*sin))),
                AddressFamily::Inet6 => sockaddr_storage
                    .as_sockaddr_in6()
                    .ok_or(Error::SockAddrStorageToAddr)
                    .map(|sin6| SocketAddr::V6(SocketAddrV6::from(*sin6))),
                other => Err(Error::UnsupportedAddrFamily(other)),
            }
        }
    }

    pub fn ip(&self) -> Result<IpAddr> {
        self.address().map(|x| x.ip())
    }

    pub fn port(&self) -> u16 {
        unsafe { sys::nw_endpoint_get_port(self.as_raw_mut()) }
    }

    fn as_raw_mut(&self) -> sys::nw_endpoint_t {
        Retained::as_ptr(&self.inner).cast_mut()
    }
}

#[derive(Debug)]
pub struct UrlEndpoint {
    inner: Retained<sys::OS_nw_endpoint>,
}

impl UrlEndpoint {
    pub fn new(url: &str) -> Result<Self> {
        let url_str = CString::new(url).map_err(|_| Error::FieldContainsNulByte(FieldName::Url))?;
        let nw_endpoint_ref = unsafe { sys::nw_endpoint_create_url(url_str.as_ptr()) };

        if nw_endpoint_ref.is_null() {
            Err(Error::InvalidEndpointData)
        } else {
            Ok(Self::retain(nw_endpoint_ref).expect("failed to retain nw_endpoint_ref"))
        }
    }

    pub(crate) fn retain(nw_endpoint_ref: sys::nw_endpoint_t) -> Option<Self> {
        if get_endpoint_type(nw_endpoint_ref) == EndpointType::Url {
            Some(Self {
                inner: unsafe { Retained::retain(nw_endpoint_ref)? },
            })
        } else {
            None
        }
    }

    pub fn url(&self) -> Result<String> {
        cstr_to_owned_string(unsafe { sys::nw_endpoint_get_url(self.as_raw_mut()) })
    }

    fn as_raw_mut(&self) -> sys::nw_endpoint_t {
        Retained::as_ptr(&self.inner).cast_mut()
    }
}

#[derive(Debug)]
pub struct BonjourServiceEndpoint {
    inner: Retained<sys::OS_nw_endpoint>,
}

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

        if nw_endpoint_ref.is_null() {
            Err(Error::InvalidEndpointData)
        } else {
            Ok(Self::retain(nw_endpoint_ref).expect("failed to retain nw_endpoint_ref"))
        }
    }

    pub(crate) fn retain(nw_endpoint_ref: sys::nw_endpoint_t) -> Option<Self> {
        if get_endpoint_type(nw_endpoint_ref) == EndpointType::BonjourService {
            Some(Self {
                inner: unsafe { Retained::retain(nw_endpoint_ref)? },
            })
        } else {
            None
        }
    }

    pub fn name(&self) -> Result<String> {
        cstr_to_owned_string(unsafe {
            sys::nw_endpoint_get_bonjour_service_name(self.as_raw_mut())
        })
    }

    pub fn domain(&self) -> Result<String> {
        cstr_to_owned_string(unsafe {
            sys::nw_endpoint_get_bonjour_service_domain(self.as_raw_mut())
        })
    }

    pub fn service_type(&self) -> Result<String> {
        cstr_to_owned_string(unsafe {
            sys::nw_endpoint_get_bonjour_service_type(self.as_raw_mut())
        })
    }

    fn as_raw_mut(&self) -> sys::nw_endpoint_t {
        Retained::as_ptr(&self.inner).cast_mut()
    }
}

fn get_endpoint_type(nw_endpoint_ref: sys::nw_endpoint_t) -> EndpointType {
    EndpointType::from(unsafe { sys::nw_endpoint_get_type(nw_endpoint_ref) })
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
    InvalidEndpointData,

    #[error("Failed to decode UTF-8 string")]
    DecodeUtf8(std::str::Utf8Error),

    #[error("Failed to parse IP address")]
    ParseIpAddress(std::net::AddrParseError),

    #[error("{:?} contains nul byte", _0)]
    FieldContainsNulByte(FieldName),

    #[error("No socket address was returned")]
    NoSocketAddr,

    #[error("Failed to convert socket address to socket address storage")]
    SockAddrToAddrStorageConversion,

    #[error("No address family set on socket address storage")]
    NoAddrFamily,

    #[error("Failure to convert socket address storage to socket address")]
    SockAddrStorageToAddr,

    #[error("Unsupported address family")]
    UnsupportedAddrFamily(AddressFamily),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

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
        let ep = AddressEndpoint::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 443).unwrap();
        assert_eq!(
            ep.address().unwrap(),
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 443))
        );
        assert_eq!(ep.port(), 443);
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
