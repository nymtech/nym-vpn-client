use crate::imp::wfp::condition::{Condition, Direction, MatchType};
use std::fmt;
use windows::Win32::NetworkManagement::WindowsFilteringPlatform::{
    FWP_CONDITION_VALUE0, FWP_CONDITION_VALUE0_0, FWP_DIRECTION_INBOUND, FWP_DIRECTION_OUTBOUND,
    FWP_UINT32, FWPM_CONDITION_DIRECTION, FWPM_FILTER_CONDITION0,
};

/// ConditionDirection
#[derive(Debug, Clone)]
pub struct ConditionDirection {
    pub direction: Direction,
    pub match_type: MatchType,
}

impl ConditionDirection {
    pub fn new(direction: Direction, match_type: MatchType) -> Self {
        ConditionDirection {
            direction,
            match_type,
        }
    }
}

impl Condition for ConditionDirection {
    fn condition(&self) -> FWPM_FILTER_CONDITION0 {
        FWPM_FILTER_CONDITION0 {
            fieldKey: FWPM_CONDITION_DIRECTION,
            matchType: self.match_type.into(),
            conditionValue: FWP_CONDITION_VALUE0 {
                r#type: FWP_UINT32,
                Anonymous: FWP_CONDITION_VALUE0_0 {
                    uint32: match self.direction {
                        Direction::Inbound => FWP_DIRECTION_INBOUND.0 as u32,
                        Direction::Outbound => FWP_DIRECTION_OUTBOUND.0 as u32,
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

impl fmt::Display for ConditionDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ConditionDirection {{ Direction: {:?}, Match: {:?} }}",
            self.direction, self.match_type
        )
    }
}
