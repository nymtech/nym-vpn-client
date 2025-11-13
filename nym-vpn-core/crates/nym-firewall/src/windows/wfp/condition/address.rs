use crate::imp::wfp::condition::{Condition, Location, MatchType};
use std::{fmt, net::IpAddr};
use windows::Win32::NetworkManagement::WindowsFilteringPlatform::{
    FWP_BYTE_ARRAY16, FWP_CONDITION_VALUE0, FWP_CONDITION_VALUE0_0, FWP_DATA_TYPE, FWP_UINT32,
    FWPM_CONDITION_IP_LOCAL_ADDRESS_V4, FWPM_CONDITION_IP_LOCAL_ADDRESS_V6,
    FWPM_CONDITION_IP_REMOTE_ADDRESS_V4, FWPM_CONDITION_IP_REMOTE_ADDRESS_V6,
    FWPM_FILTER_CONDITION0,
};

// Not exposed by windows-rs!
const FWP_BYTE_ARRAY16_TYPE: FWP_DATA_TYPE = FWP_DATA_TYPE(11);

/// ConditionAddress
#[derive(Debug, Clone)]
pub struct ConditionAddress {
    pub location: Location,
    pub ip_addr: IpAddr,
    pub address: Address,
    pub match_type: MatchType,
}

#[derive(Debug, Clone)]
pub enum Address {
    V4(u32),
    V6(Box<FWP_BYTE_ARRAY16>),
}

impl ConditionAddress {
    pub fn new(location: Location, ip_addr: &IpAddr, match_type: MatchType) -> Self {
        let ip_addr = *ip_addr;
        let address = match ip_addr {
            IpAddr::V4(v4_addr) => Address::V4(v4_addr.to_bits()),
            IpAddr::V6(v6_addr) => Address::V6(Box::new(FWP_BYTE_ARRAY16 {
                byteArray16: v6_addr.octets(),
            })),
        };

        ConditionAddress {
            location,
            ip_addr,
            address,
            match_type,
        }
    }
}

impl Condition for ConditionAddress {
    fn condition(&self) -> FWPM_FILTER_CONDITION0 {
        match &self.address {
            Address::V4(addr) => FWPM_FILTER_CONDITION0 {
                fieldKey: match self.location {
                    Location::Local => FWPM_CONDITION_IP_LOCAL_ADDRESS_V4,
                    Location::Remote => FWPM_CONDITION_IP_REMOTE_ADDRESS_V4,
                },
                matchType: self.match_type.into(),
                conditionValue: FWP_CONDITION_VALUE0 {
                    r#type: FWP_UINT32,
                    Anonymous: FWP_CONDITION_VALUE0_0 { uint32: *addr },
                },
            },
            Address::V6(addr) => FWPM_FILTER_CONDITION0 {
                fieldKey: match self.location {
                    Location::Local => FWPM_CONDITION_IP_LOCAL_ADDRESS_V6,
                    Location::Remote => FWPM_CONDITION_IP_REMOTE_ADDRESS_V6,
                },
                matchType: self.match_type.into(),
                conditionValue: FWP_CONDITION_VALUE0 {
                    r#type: FWP_BYTE_ARRAY16_TYPE,
                    Anonymous: FWP_CONDITION_VALUE0_0 {
                        byteArray16: addr.as_ref() as *const _ as *mut _,
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

impl fmt::Display for ConditionAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ConditionAddress {{ Location: {:?}, IP Address: {}, Match: {:?} }}",
            self.location, self.ip_addr, self.match_type
        )
    }
}
