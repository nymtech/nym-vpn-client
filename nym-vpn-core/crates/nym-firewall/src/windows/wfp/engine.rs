use crate::imp::{Error, wfp::transaction::*};
use nym_windows::{error::win32_error, str::wstr};
use windows::{
    Win32::{
        Foundation::{HANDLE, INVALID_HANDLE_VALUE, STATUS_SUCCESS},
        NetworkManagement::WindowsFilteringPlatform::{
            FWPM_DISPLAY_DATA0, FWPM_SESSION_FLAG_DYNAMIC, FWPM_SESSION0, FwpmEngineClose0,
            FwpmEngineOpen0,
        },
        System::Rpc::RPC_C_AUTHN_DEFAULT,
    },
    core::{PCWSTR, PWSTR},
};

/// WFP Engine connection
#[derive(Clone, Debug, Default)]
pub struct Engine {
    handle: EngineHandle,
    config: EngineConfig,
}

impl Engine {
    /// Create a new WFP engine connection.
    pub fn init(config: &EngineConfig) -> Result<Self, Error> {
        let name = wstr("Nym VPN Firewall Engine");
        let description = wstr("Nym VPN Firewall Engine Session");
        let session_info = FWPM_SESSION0 {
            displayData: FWPM_DISPLAY_DATA0 {
                name: PWSTR(name.as_ptr() as *mut _),
                description: PWSTR(description.as_ptr() as *mut _),
            },
            flags: if config.dynamic {
                FWPM_SESSION_FLAG_DYNAMIC
            } else {
                0
            },
            txnWaitTimeoutInMSec: config.timeout_secs * 1000,
            ..Default::default()
        };

        let mut handle = INVALID_HANDLE_VALUE;

        let status = unsafe {
            FwpmEngineOpen0(
                PCWSTR::null(),
                RPC_C_AUTHN_DEFAULT as u32,
                None,
                Some(&session_info),
                &mut handle,
            )
        };
        if status != STATUS_SUCCESS.0 as u32 {
            return Err(Error::Initialization {
                reason: format!("FwpmEngineOpen0 failed: {}", win32_error(status)),
            });
        }

        tracing::debug!("WFP engine opened successfully");

        Ok(Engine {
            handle: EngineHandle(handle),
            config: config.clone(),
        })
    }

    pub fn deinit(&mut self) -> Result<(), Error> {
        if self.handle.0 != INVALID_HANDLE_VALUE {
            let status = unsafe { FwpmEngineClose0(self.handle.0) };
            self.handle.0 = INVALID_HANDLE_VALUE;
            if status != STATUS_SUCCESS.0 as u32 {
                return Err(Error::Deinitialization {
                    reason: format!("FwpmEngineClose0 failed: {}", win32_error(status)),
                });
            }
        }
        Ok(())
    }

    pub fn handle(&self) -> HANDLE {
        self.handle.0
    }

    pub fn begin_transaction(&self) -> Result<Transaction<'_>, Error> {
        Transaction::begin(self)
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        if let Err(e) = self.deinit() {
            tracing::error!("Error during WFP engine drop: {e}");
        }
    }
}

/// Thread-safe wrapper for WFP engine handle
#[derive(Clone, Debug, Default)]
struct EngineHandle(HANDLE);

// SAFETY: WFP engine handles are safe to send between threads
unsafe impl Send for EngineHandle {}

// SAFETY: WFP engine handles are safe to share between threads
unsafe impl Sync for EngineHandle {}

/// Engine configuration settings
#[derive(Clone, Debug, Default)]
pub struct EngineConfig {
    pub dynamic: bool,
    pub timeout_secs: u32,
    pub allow_dhcp: bool,
    pub allow_lan: bool,
}
