use crate::imp::wfp::condition::{Condition, MatchType, Protocol};
use std::fmt;
use windows::Win32::{
    NetworkManagement::WindowsFilteringPlatform::{
        FWP_CONDITION_VALUE0, FWP_CONDITION_VALUE0_0, FWP_UINT8, FWPM_CONDITION_IP_PROTOCOL,
        FWPM_FILTER_CONDITION0,
    },
    Networking::WinSock::IPPROTO,
};

/// ConditionProtocol
#[derive(Clone, Debug)]
pub struct ConditionProtocol {
    pub protocol: Protocol,
    pub match_type: MatchType,
}

impl ConditionProtocol {
    pub fn new(protocol: Protocol, match_type: MatchType) -> Self {
        ConditionProtocol {
            protocol,
            match_type,
        }
    }
}

impl Condition for ConditionProtocol {
    fn condition(&self) -> FWPM_FILTER_CONDITION0 {
        let ip_proto: IPPROTO = self.protocol.into();

        FWPM_FILTER_CONDITION0 {
            fieldKey: FWPM_CONDITION_IP_PROTOCOL,
            matchType: self.match_type.into(),
            conditionValue: FWP_CONDITION_VALUE0 {
                r#type: FWP_UINT8,
                Anonymous: FWP_CONDITION_VALUE0_0 {
                    uint8: ip_proto.0 as u8,
                },
            },
        }
    }

    #[cfg(test)]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl fmt::Display for ConditionProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ConditionProtocol {{ Protocol: {:?}, Match: {:?} }}",
            self.protocol, self.match_type
        )
    }
}
