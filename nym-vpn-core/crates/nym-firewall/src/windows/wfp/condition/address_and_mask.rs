use crate::imp::wfp::condition::{Condition, Location, MatchType};
use std::{fmt, net::IpAddr};
use windows::Win32::NetworkManagement::WindowsFilteringPlatform::{
    FWP_CONDITION_VALUE0, FWP_CONDITION_VALUE0_0, FWP_V4_ADDR_AND_MASK, FWP_V4_ADDR_MASK,
    FWP_V6_ADDR_AND_MASK, FWP_V6_ADDR_MASK, FWPM_CONDITION_IP_LOCAL_ADDRESS,
    FWPM_CONDITION_IP_REMOTE_ADDRESS, FWPM_FILTER_CONDITION0,
};

/// ConditionAddressAndMask
#[derive(Debug, Clone)]
pub struct ConditionAddressAndMask {
    pub location: Location,
    pub ip_addr: IpAddr,
    pub mask: u8,
    pub address_and_mask: AddressAndMask,
    pub match_type: MatchType,
}

#[derive(Debug, Clone)]
pub enum AddressAndMask {
    V4(Box<FWP_V4_ADDR_AND_MASK>),
    V6(Box<FWP_V6_ADDR_AND_MASK>),
}

impl ConditionAddressAndMask {
    pub fn new(location: Location, ip_addr: &IpAddr, mask: u8, match_type: MatchType) -> Self {
        let ip_addr = *ip_addr;
        let address_and_mask = match ip_addr {
            IpAddr::V4(v4_addr) => AddressAndMask::V4(Box::new(FWP_V4_ADDR_AND_MASK {
                addr: v4_addr.to_bits(),
                mask: mask as u32,
            })),
            IpAddr::V6(v6_addr) => AddressAndMask::V6(Box::new(FWP_V6_ADDR_AND_MASK {
                addr: v6_addr.octets(),
                prefixLength: mask,
            })),
        };

        ConditionAddressAndMask {
            location,
            ip_addr,
            mask,
            address_and_mask,
            match_type,
        }
    }
}

impl Condition for ConditionAddressAndMask {
    fn condition(&self) -> FWPM_FILTER_CONDITION0 {
        match &self.address_and_mask {
            AddressAndMask::V4(addr_and_mask) => FWPM_FILTER_CONDITION0 {
                fieldKey: match self.location {
                    Location::Local => FWPM_CONDITION_IP_LOCAL_ADDRESS,
                    Location::Remote => FWPM_CONDITION_IP_REMOTE_ADDRESS,
                },
                matchType: self.match_type.into(),
                conditionValue: FWP_CONDITION_VALUE0 {
                    r#type: FWP_V4_ADDR_MASK,
                    Anonymous: FWP_CONDITION_VALUE0_0 {
                        v4AddrMask: addr_and_mask.as_ref() as *const _ as *mut _,
                    },
                },
            },
            AddressAndMask::V6(addr_and_mask) => FWPM_FILTER_CONDITION0 {
                fieldKey: match self.location {
                    Location::Local => FWPM_CONDITION_IP_LOCAL_ADDRESS,
                    Location::Remote => FWPM_CONDITION_IP_REMOTE_ADDRESS,
                },
                matchType: self.match_type.into(),
                conditionValue: FWP_CONDITION_VALUE0 {
                    r#type: FWP_V6_ADDR_MASK,
                    Anonymous: FWP_CONDITION_VALUE0_0 {
                        v6AddrMask: addr_and_mask.as_ref() as *const _ as *mut _,
                    },
                },
            },
        }
    }

    #[cfg(test)]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl fmt::Display for ConditionAddressAndMask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ConditionAddressAndMask {{ Location: {:?}, IP Address: {}, Mask: {}, Match: {:?} }}",
            self.location, self.ip_addr, self.mask, self.match_type
        )
    }
}
