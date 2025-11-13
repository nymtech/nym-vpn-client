use crate::{
    Error,
    imp::wfp::condition::{Condition, MatchType},
};
use nym_windows::{error::win32_error, str::wstr};
use std::{
    fmt,
    path::{Path, PathBuf},
    ptr,
    slice::from_raw_parts,
};
use windows::{
    Win32::{
        Foundation::STATUS_SUCCESS,
        NetworkManagement::WindowsFilteringPlatform::{
            FWP_BYTE_BLOB, FWP_BYTE_BLOB_TYPE, FWP_CONDITION_VALUE0, FWP_CONDITION_VALUE0_0,
            FWPM_CONDITION_ALE_APP_ID, FWPM_FILTER_CONDITION0, FwpmFreeMemory0,
            FwpmGetAppIdFromFileName0,
        },
    },
    core::PCWSTR,
};

/// ConditionApplication
#[derive(Debug, Clone)]
pub struct ConditionApplication {
    pub file_path: PathBuf,
    pub app_id_data: Vec<u8>,
    pub app_id_blob: Box<FWP_BYTE_BLOB>,
    pub match_type: MatchType,
}

impl ConditionApplication {
    pub fn new<P: AsRef<Path>>(file_path: P, match_type: MatchType) -> Result<Self, Error> {
        let path_str = file_path
            .as_ref()
            .to_str()
            .ok_or_else(|| Error::Condition {
                reason: "Invalid application path".to_string(),
            })?;
        let path_wstr = wstr(path_str);
        let mut appid: *mut FWP_BYTE_BLOB = ptr::null_mut();
        let status = unsafe { FwpmGetAppIdFromFileName0(PCWSTR(path_wstr.as_ptr()), &mut appid) };
        if status != STATUS_SUCCESS.0 as u32 {
            return Err(Error::Condition {
                reason: format!(
                    "FwpmGetAppIdFromFileName0 failed for application '{path_str}': {}",
                    win32_error(status)
                ),
            });
        }
        let slice = unsafe {
            debug_assert!(!appid.is_null());
            debug_assert!(!(*appid).data.is_null());
            debug_assert!(!(*appid).size != 0);
            from_raw_parts((*appid).data as *const u8, (*appid).size as usize)
        };
        let app_id_data = slice.to_vec();
        let app_id_blob = Box::new(FWP_BYTE_BLOB {
            size: app_id_data.len() as u32,
            data: app_id_data.as_ptr() as *mut _,
        });

        unsafe {
            FwpmFreeMemory0(&mut appid as *mut _ as *mut _);
        }

        Ok(ConditionApplication {
            file_path: file_path.as_ref().to_path_buf(),
            app_id_data,
            app_id_blob,
            match_type,
        })
    }
}

impl Condition for ConditionApplication {
    fn condition(&self) -> FWPM_FILTER_CONDITION0 {
        FWPM_FILTER_CONDITION0 {
            fieldKey: FWPM_CONDITION_ALE_APP_ID,
            matchType: self.match_type.into(),
            conditionValue: FWP_CONDITION_VALUE0 {
                r#type: FWP_BYTE_BLOB_TYPE,
                Anonymous: FWP_CONDITION_VALUE0_0 {
                    byteBlob: self.app_id_blob.as_ref() as *const _ as *mut _,
                },
            },
        }
    }

    #[cfg(test)]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl fmt::Display for ConditionApplication {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ConditionApplication {{ Path: {}, Match: {:?} }}",
            self.file_path.display(),
            self.match_type
        )
    }
}
