mod address;
mod address_and_mask;
mod application;
mod direction;
mod icmp;
mod interface;
mod loopback;
mod port;
mod port_range;
mod protocol;

#[cfg(test)]
pub mod tests;

use crate::imp::Error;
use std::{fmt, net::IpAddr, ops::Range, path::Path};
use windows::Win32::{
    NetworkManagement::WindowsFilteringPlatform::{
        FWP_MATCH_EQUAL, FWP_MATCH_EQUAL_CASE_INSENSITIVE, FWP_MATCH_FLAGS_ALL_SET,
        FWP_MATCH_FLAGS_ANY_SET, FWP_MATCH_FLAGS_NONE_SET, FWP_MATCH_GREATER,
        FWP_MATCH_GREATER_OR_EQUAL, FWP_MATCH_LESS, FWP_MATCH_LESS_OR_EQUAL, FWP_MATCH_NOT_EQUAL,
        FWP_MATCH_NOT_PREFIX, FWP_MATCH_RANGE, FWP_MATCH_TYPE, FWPM_FILTER_CONDITION0,
    },
    Networking::WinSock::{
        IPPROTO, IPPROTO_ICMP, IPPROTO_ICMPV6, IPPROTO_IP, IPPROTO_IPV6, IPPROTO_RAW, IPPROTO_TCP,
        IPPROTO_UDP,
    },
};

trait Condition: fmt::Debug + fmt::Display {
    fn condition(&self) -> FWPM_FILTER_CONDITION0;

    #[cfg(test)]
    fn as_any(&self) -> &dyn std::any::Any;
}

///
/// WFP filter condition builder.
///
/// Note: care is taken to hold any condition data, that is passed to WFP via a pointer,
/// in a Box or Vec so the address remains valid until the filter is built.
///
#[derive(Debug, Default)]
pub struct ConditionBuilder {
    conditions: Vec<Box<dyn Condition>>,
}

impl ConditionBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn address(mut self, location: Location, address: &IpAddr) -> Self {
        let condition = address::ConditionAddress::new(location, address, MatchType::Equal);
        self.conditions.push(Box::new(condition));
        self
    }

    pub fn address_match(
        mut self,
        location: Location,
        address: &IpAddr,
        match_type: MatchType,
    ) -> Self {
        let condition = address::ConditionAddress::new(location, address, match_type);
        self.conditions.push(Box::new(condition));
        self
    }

    pub fn address_and_mask(mut self, location: Location, address: &IpAddr, mask: u8) -> Self {
        let condition = address_and_mask::ConditionAddressAndMask::new(
            location,
            address,
            mask,
            MatchType::Equal,
        );
        self.conditions.push(Box::new(condition));
        self
    }

    pub fn address_and_mask_match(
        mut self,
        location: Location,
        address: &IpAddr,
        mask: u8,
        match_type: MatchType,
    ) -> Self {
        let condition =
            address_and_mask::ConditionAddressAndMask::new(location, address, mask, match_type);
        self.conditions.push(Box::new(condition));
        self
    }

    pub fn application<P: AsRef<Path>>(mut self, file_path: P) -> Result<Self, Error> {
        let condition = application::ConditionApplication::new(file_path, MatchType::Equal)?;
        self.conditions.push(Box::new(condition));
        Ok(self)
    }

    pub fn appication_match<P: AsRef<Path>>(
        mut self,
        file_path: P,
        match_type: MatchType,
    ) -> Result<Self, Error> {
        let condition = application::ConditionApplication::new(file_path, match_type)?;
        self.conditions.push(Box::new(condition));
        Ok(self)
    }

    pub fn direction(mut self, direction: Direction) -> Self {
        let condition = direction::ConditionDirection::new(direction, MatchType::Equal);
        self.conditions.push(Box::new(condition));
        self
    }

    pub fn direction_match(mut self, direction: Direction, match_type: MatchType) -> Self {
        let condition = direction::ConditionDirection::new(direction, match_type);
        self.conditions.push(Box::new(condition));
        self
    }

    pub fn icmp(mut self, icmp: Icmp, value: u32) -> Self {
        let condition = icmp::ConditionIcmp::new(icmp, value, MatchType::Equal);
        self.conditions.push(Box::new(condition));
        self
    }

    pub fn icmp_match(mut self, icmp: Icmp, value: u32, match_type: MatchType) -> Self {
        let condition = icmp::ConditionIcmp::new(icmp, value, match_type);
        self.conditions.push(Box::new(condition));
        self
    }

    pub fn interface_name(mut self, name: &str) -> Result<Self, Error> {
        let condition = interface::ConditionInterface::from_name(name, MatchType::Equal)?;
        self.conditions.push(Box::new(condition));
        Ok(self)
    }

    pub fn interface_alias(mut self, alias: &str) -> Result<Self, Error> {
        let condition = interface::ConditionInterface::from_alias(alias, MatchType::Equal)?;
        self.conditions.push(Box::new(condition));
        Ok(self)
    }

    pub fn interface_index(mut self, index: u32) -> Self {
        let condition = interface::ConditionInterface::from_index(index, MatchType::Equal);
        self.conditions.push(Box::new(condition));
        self
    }

    pub fn loopback(mut self, loopback: Loopback) -> Self {
        let condition = loopback::ConditionLoopback::new(loopback, MatchType::FlagsAllSet);
        self.conditions.push(Box::new(condition));
        self
    }

    pub fn loopback_match(
        mut self,
        loopback: Loopback,
        match_type: MatchType,
    ) -> Result<Self, Error> {
        if !matches!(match_type, MatchType::FlagsAllSet | MatchType::FlagsNoneSet) {
            return Err(Error::Condition {
                reason: "Loopback condition match type must be FlagsAllSet or FlagsNoneSet"
                    .to_string(),
            });
        }

        let condition = loopback::ConditionLoopback::new(loopback, match_type);
        self.conditions.push(Box::new(condition));
        Ok(self)
    }

    pub fn port(mut self, location: Location, port: u16) -> Self {
        let condition = port::ConditionPort::new(location, port, MatchType::Equal);
        self.conditions.push(Box::new(condition));
        self
    }

    pub fn port_match(mut self, location: Location, port: u16, match_type: MatchType) -> Self {
        let condition = port::ConditionPort::new(location, port, match_type);
        self.conditions.push(Box::new(condition));
        self
    }

    /// Note that the range is exclusive but is converted to an inclusive range internally.
    pub fn port_range(mut self, location: Location, port_range: Range<u16>) -> Self {
        let condition = port_range::ConditionPortRange::new(location, port_range, MatchType::Range);
        self.conditions.push(Box::new(condition));
        self
    }

    /// Note that the range is exclusive but is converted to an inclusive range internally.
    pub fn port_range_match(
        mut self,
        location: Location,
        port_range: Range<u16>,
        match_type: MatchType,
    ) -> Self {
        let condition = port_range::ConditionPortRange::new(location, port_range, match_type);
        self.conditions.push(Box::new(condition));
        self
    }

    pub fn protocol(mut self, protocol: Protocol) -> Self {
        let condition = protocol::ConditionProtocol::new(protocol, MatchType::Equal);
        self.conditions.push(Box::new(condition));
        self
    }

    pub fn protocol_match(mut self, protocol: Protocol, match_type: MatchType) -> Self {
        let condition = protocol::ConditionProtocol::new(protocol, match_type);
        self.conditions.push(Box::new(condition));
        self
    }

    pub fn build(self) -> Result<Conditions, Error> {
        let wfp_conditions = self.conditions.iter().map(|c| c.condition()).collect();
        let conditions = Conditions {
            conditions: self.conditions,
            wfp_conditions,
        };
        Ok(conditions)
    }
}

pub struct Conditions {
    conditions: Vec<Box<dyn Condition>>,
    wfp_conditions: Vec<FWPM_FILTER_CONDITION0>,
}

impl Conditions {
    pub fn len(&self) -> u32 {
        self.wfp_conditions.len() as u32
    }

    pub fn as_ptr(&self) -> *const FWPM_FILTER_CONDITION0 {
        self.wfp_conditions.as_ptr()
    }
}

impl fmt::Display for Conditions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Conditions ({}) [", self.conditions.len())?;
        for condition in &self.conditions {
            writeln!(f, "  {},", condition)?;
        }
        writeln!(f, "]")
    }
}

/// MatchType
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchType {
    Equal,
    EqualCaseInsensitive,
    FlagsAllSet,
    FlagsAnySet,
    FlagsNoneSet,
    Greater,
    GreaterOrEqual,
    Less,
    LessOrEqual,
    NotEqual,
    NotPrefix,
    Prefix,
    Range,
}

impl From<MatchType> for FWP_MATCH_TYPE {
    fn from(m: MatchType) -> Self {
        match m {
            MatchType::Equal => FWP_MATCH_EQUAL,
            MatchType::EqualCaseInsensitive => FWP_MATCH_EQUAL_CASE_INSENSITIVE,
            MatchType::FlagsAllSet => FWP_MATCH_FLAGS_ALL_SET,
            MatchType::FlagsAnySet => FWP_MATCH_FLAGS_ANY_SET,
            MatchType::FlagsNoneSet => FWP_MATCH_FLAGS_NONE_SET,
            MatchType::Greater => FWP_MATCH_GREATER,
            MatchType::GreaterOrEqual => FWP_MATCH_GREATER_OR_EQUAL,
            MatchType::Less => FWP_MATCH_LESS,
            MatchType::LessOrEqual => FWP_MATCH_LESS_OR_EQUAL,
            MatchType::NotEqual => FWP_MATCH_NOT_EQUAL,
            MatchType::NotPrefix => FWP_MATCH_NOT_PREFIX,
            MatchType::Prefix => FWP_MATCH_NOT_PREFIX,
            MatchType::Range => FWP_MATCH_RANGE,
        }
    }
}

/// Location
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Location {
    Local,
    Remote,
}

/// Loopback
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Loopback {
    Interface,
    Traffic,
}

/// Direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Inbound,
    Outbound,
}

/// Icmp
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icmp {
    Code,
    Type,
}

/// Protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    Tcp,
    Udp,
    Icmp,
    IcmpV6,
    Ip,
    IpV6,
    Raw,
}

impl From<Protocol> for IPPROTO {
    fn from(protocol: Protocol) -> Self {
        match protocol {
            Protocol::Tcp => IPPROTO_TCP,
            Protocol::Udp => IPPROTO_UDP,
            Protocol::Icmp => IPPROTO_ICMP,
            Protocol::IcmpV6 => IPPROTO_ICMPV6,
            Protocol::Ip => IPPROTO_IP,
            Protocol::IpV6 => IPPROTO_IPV6,
            Protocol::Raw => IPPROTO_RAW,
        }
    }
}
