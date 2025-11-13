use crate::imp::wfp::condition::{Condition, Icmp, MatchType};
use std::fmt;
use windows::Win32::NetworkManagement::WindowsFilteringPlatform::{
    FWP_CONDITION_VALUE0, FWP_CONDITION_VALUE0_0, FWP_UINT32, FWPM_CONDITION_IP_LOCAL_PORT,
    FWPM_CONDITION_IP_REMOTE_PORT, FWPM_FILTER_CONDITION0,
};

/// ConditionIcmp
#[derive(Debug, Clone)]
pub struct ConditionIcmp {
    pub icmp: Icmp,
    pub value: u32,
    pub match_type: MatchType,
}

impl ConditionIcmp {
    pub fn new(icmp: Icmp, value: u32, match_type: MatchType) -> Self {
        ConditionIcmp {
            icmp,
            value,
            match_type,
        }
    }
}

impl Condition for ConditionIcmp {
    fn condition(&self) -> FWPM_FILTER_CONDITION0 {
        FWPM_FILTER_CONDITION0 {
            fieldKey: match self.icmp {
                // windows-rs does not expose these constants directly.  In the C header they are:
                // #define FWPM_CONDITION_IP_ICMP_TYPE FWPM_CONDITION_IP_LOCAL_PORT
                // #define FWPM_CONDITION_IP_ICMP_CODE FWPM_CONDITION_IP_REMOTE_PORT
                // So how does WFP distinguish between an ICMP condition and a port condition?
                Icmp::Type => FWPM_CONDITION_IP_LOCAL_PORT,
                Icmp::Code => FWPM_CONDITION_IP_REMOTE_PORT,
            },
            matchType: self.match_type.into(),
            conditionValue: FWP_CONDITION_VALUE0 {
                r#type: FWP_UINT32,
                Anonymous: FWP_CONDITION_VALUE0_0 { uint32: self.value },
            },
        }
    }

    #[cfg(test)]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl fmt::Display for ConditionIcmp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ConditionIcmp {{ Icmp: {:?}, Value: {}, Match: {:?} }}",
            self.icmp, self.value, self.match_type
        )
    }
}
