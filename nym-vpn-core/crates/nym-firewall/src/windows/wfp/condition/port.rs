use crate::imp::wfp::condition::{Condition, Location, MatchType};
use std::fmt;
use windows::Win32::NetworkManagement::WindowsFilteringPlatform::{
    FWP_CONDITION_VALUE0, FWP_CONDITION_VALUE0_0, FWP_UINT16, FWPM_CONDITION_IP_LOCAL_PORT,
    FWPM_CONDITION_IP_REMOTE_PORT, FWPM_FILTER_CONDITION0,
};

/// ConditionPort
#[derive(Clone, Debug)]
pub struct ConditionPort {
    pub location: Location,
    pub port: u16,
    pub match_type: MatchType,
}

impl ConditionPort {
    pub fn new(location: Location, port: u16, match_type: MatchType) -> Self {
        ConditionPort {
            location,
            port,
            match_type,
        }
    }
}

impl Condition for ConditionPort {
    fn condition(&self) -> FWPM_FILTER_CONDITION0 {
        FWPM_FILTER_CONDITION0 {
            fieldKey: match self.location {
                Location::Local => FWPM_CONDITION_IP_LOCAL_PORT,
                Location::Remote => FWPM_CONDITION_IP_REMOTE_PORT,
            },
            matchType: self.match_type.into(),
            conditionValue: FWP_CONDITION_VALUE0 {
                r#type: FWP_UINT16,
                Anonymous: FWP_CONDITION_VALUE0_0 { uint16: self.port },
            },
        }
    }

    #[cfg(test)]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl fmt::Display for ConditionPort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ConditionPort {{ Location: {:?}, Port: {}, Match: {:?} }}",
            self.location, self.port, self.match_type
        )
    }
}
