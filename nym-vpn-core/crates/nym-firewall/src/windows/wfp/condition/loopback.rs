use crate::imp::wfp::condition::{Condition, Loopback, MatchType};
use std::fmt;
use windows::Win32::NetworkManagement::{
    IpHelper::IF_TYPE_SOFTWARE_LOOPBACK,
    WindowsFilteringPlatform::{
        FWP_CONDITION_FLAG_IS_LOOPBACK, FWP_CONDITION_VALUE0, FWP_CONDITION_VALUE0_0, FWP_UINT32,
        FWPM_CONDITION_FLAGS, FWPM_CONDITION_INTERFACE_TYPE, FWPM_FILTER_CONDITION0,
    },
};

/// ConditionLoopback
#[derive(Debug, Clone)]
pub struct ConditionLoopback {
    pub loopback: Loopback,
    pub match_type: MatchType,
}

impl ConditionLoopback {
    pub fn new(loopback: Loopback, match_type: MatchType) -> Self {
        ConditionLoopback {
            loopback,
            match_type,
        }
    }
}

impl Condition for ConditionLoopback {
    fn condition(&self) -> FWPM_FILTER_CONDITION0 {
        match self.loopback {
            Loopback::Interface => FWPM_FILTER_CONDITION0 {
                fieldKey: FWPM_CONDITION_INTERFACE_TYPE,
                matchType: self.match_type.into(),
                conditionValue: FWP_CONDITION_VALUE0 {
                    r#type: FWP_UINT32,
                    Anonymous: FWP_CONDITION_VALUE0_0 {
                        uint32: IF_TYPE_SOFTWARE_LOOPBACK,
                    },
                },
            },
            Loopback::Traffic => FWPM_FILTER_CONDITION0 {
                fieldKey: FWPM_CONDITION_FLAGS,
                matchType: self.match_type.into(),
                conditionValue: FWP_CONDITION_VALUE0 {
                    r#type: FWP_UINT32,
                    Anonymous: FWP_CONDITION_VALUE0_0 {
                        uint32: FWP_CONDITION_FLAG_IS_LOOPBACK,
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

impl fmt::Display for ConditionLoopback {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ConditionLoopback {{ Loopback: {:?}, Match: {:?} }}",
            self.loopback, self.match_type
        )
    }
}
