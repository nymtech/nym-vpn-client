use crate::imp::wfp::condition::{Condition, Location, MatchType};
use std::{fmt, ops::Range};
use windows::Win32::NetworkManagement::WindowsFilteringPlatform::{
    FWP_CONDITION_VALUE0, FWP_CONDITION_VALUE0_0, FWP_RANGE_TYPE, FWP_RANGE0, FWP_UINT16,
    FWP_VALUE0, FWP_VALUE0_0, FWPM_CONDITION_IP_LOCAL_PORT, FWPM_CONDITION_IP_REMOTE_PORT,
    FWPM_FILTER_CONDITION0,
};

/// ConditionPortRange
#[derive(Clone)]
pub struct ConditionPortRange {
    pub location: Location,
    pub port_range: Box<FWP_RANGE0>,
    pub match_type: MatchType,
}

impl ConditionPortRange {
    pub fn new(location: Location, port_range: Range<u16>, match_type: MatchType) -> Self {
        let wfp_range = FWP_RANGE0 {
            valueLow: FWP_VALUE0 {
                r#type: FWP_UINT16,
                Anonymous: FWP_VALUE0_0 {
                    uint16: port_range.start,
                },
            },
            valueHigh: FWP_VALUE0 {
                r#type: FWP_UINT16,
                Anonymous: FWP_VALUE0_0 {
                    uint16: port_range.end - 1, // WFP port range is inclusive
                },
            },
        };

        ConditionPortRange {
            location,
            port_range: Box::new(wfp_range),
            match_type,
        }
    }
}

impl Condition for ConditionPortRange {
    fn condition(&self) -> FWPM_FILTER_CONDITION0 {
        FWPM_FILTER_CONDITION0 {
            fieldKey: match self.location {
                Location::Local => FWPM_CONDITION_IP_LOCAL_PORT,
                Location::Remote => FWPM_CONDITION_IP_REMOTE_PORT,
            },
            matchType: self.match_type.into(),
            conditionValue: FWP_CONDITION_VALUE0 {
                r#type: FWP_RANGE_TYPE,
                Anonymous: FWP_CONDITION_VALUE0_0 {
                    rangeValue: self.port_range.as_ref() as *const _ as *const _ as *mut _,
                },
            },
        }
    }

    #[cfg(test)]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl fmt::Debug for ConditionPortRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "location: {:?}, port_range: {}-{}, match_type: {:?}",
            self.location,
            unsafe { self.port_range.valueLow.Anonymous.uint16 },
            unsafe { self.port_range.valueHigh.Anonymous.uint16 },
            self.match_type
        )
    }
}

impl fmt::Display for ConditionPortRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ConditionPortRange {{ Location: {:?}, Port Range: {}-{}, Match: {:?} }}",
            self.location,
            unsafe { self.port_range.valueLow.Anonymous.uint16 },
            unsafe { self.port_range.valueHigh.Anonymous.uint16 },
            self.match_type
        )
    }
}
