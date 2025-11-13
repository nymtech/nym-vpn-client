use crate::{
    Error,
    imp::wfp::condition::{Condition, MatchType},
};
use nym_windows::{error::win32_error, str::wstr};
use std::fmt;
use windows::{
    Win32::{
        Foundation::ERROR_SUCCESS,
        NetworkManagement::{
            IpHelper::{ConvertInterfaceAliasToLuid, ConvertInterfaceNameToLuidW},
            Ndis::NET_LUID_LH,
            WindowsFilteringPlatform::{
                FWP_CONDITION_VALUE0, FWP_CONDITION_VALUE0_0, FWP_UINT32, FWP_UINT64,
                FWPM_CONDITION_INTERFACE_INDEX, FWPM_CONDITION_IP_LOCAL_INTERFACE,
                FWPM_FILTER_CONDITION0,
            },
        },
    },
    core::PCWSTR,
};

/// ConditionInterface
#[derive(Debug, Clone)]
pub struct ConditionInterface {
    pub interface: Interface,
    pub match_type: MatchType,
}

/// Interface
#[derive(Debug, Clone)]
pub enum Interface {
    Index(u32),
    Luid(Box<u64>),
}

impl ConditionInterface {
    pub fn from_name(name: &str, match_type: MatchType) -> Result<Self, Error> {
        let wname = wstr(name);
        let mut luid = NET_LUID_LH::default();
        let status = unsafe { ConvertInterfaceNameToLuidW(PCWSTR(wname.as_ptr()), &mut luid) };
        if status != ERROR_SUCCESS {
            return Err(Error::Condition {
                reason: format!(
                    "ConvertInterfaceNameToLuidW failed for interface '{name}': {}",
                    win32_error(status.0)
                ),
            });
        }
        let interface = Interface::Luid(Box::new(unsafe { luid.Value }));
        Ok(Self {
            interface,
            match_type,
        })
    }

    pub fn from_alias(alias: &str, match_type: MatchType) -> Result<Self, Error> {
        let walias = wstr(alias);
        let mut luid = NET_LUID_LH::default();
        let status = unsafe { ConvertInterfaceAliasToLuid(PCWSTR(walias.as_ptr()), &mut luid) };
        if status != ERROR_SUCCESS {
            return Err(Error::Condition {
                reason: format!(
                    "ConvertInterfaceAliasToLuid failed for interface '{alias}': {}",
                    win32_error(status.0)
                ),
            });
        }
        let interface = Interface::Luid(Box::new(unsafe { luid.Value }));
        Ok(Self {
            interface,
            match_type,
        })
    }

    pub fn from_index(index: u32, match_type: MatchType) -> Self {
        let interface = Interface::Index(index);
        Self {
            interface,
            match_type,
        }
    }
}

impl Condition for ConditionInterface {
    fn condition(&self) -> FWPM_FILTER_CONDITION0 {
        match &self.interface {
            Interface::Index(index) => FWPM_FILTER_CONDITION0 {
                fieldKey: FWPM_CONDITION_INTERFACE_INDEX,
                matchType: self.match_type.into(),
                conditionValue: FWP_CONDITION_VALUE0 {
                    r#type: FWP_UINT32,
                    Anonymous: FWP_CONDITION_VALUE0_0 { uint32: *index },
                },
            },
            Interface::Luid(luid) => FWPM_FILTER_CONDITION0 {
                fieldKey: FWPM_CONDITION_IP_LOCAL_INTERFACE,
                matchType: self.match_type.into(),
                conditionValue: FWP_CONDITION_VALUE0 {
                    r#type: FWP_UINT64,
                    Anonymous: FWP_CONDITION_VALUE0_0 {
                        uint64: luid.as_ref() as *const _ as *mut _,
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

impl fmt::Display for ConditionInterface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.interface {
            Interface::Index(index) => {
                write!(
                    f,
                    "ConditionInterface {{ Index: {}, Match: {:?} }}",
                    index, self.match_type
                )
            }
            Interface::Luid(luid) => {
                write!(
                    f,
                    "ConditionInterface {{ LUID: 0x{:016x}, Match: {:?} }}",
                    **luid, self.match_type
                )
            }
        }
    }
}
