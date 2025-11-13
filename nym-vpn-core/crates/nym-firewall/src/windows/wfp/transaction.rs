use windows::Win32::{
    Foundation::STATUS_SUCCESS,
    NetworkManagement::WindowsFilteringPlatform::{
        FwpmTransactionAbort0, FwpmTransactionBegin0, FwpmTransactionCommit0,
    },
};

use super::Engine;
use crate::imp::Error;
use nym_windows::error::win32_error;

/// WFP Transaction.
/// If the transation is neither committed nor aborted explicitly, it is aborted in Drop.
pub struct Transaction<'a> {
    engine: &'a Engine,
    completed: bool,
}

impl<'a> Transaction<'a> {
    /// Begin a new WFP transaction.
    pub fn begin(engine: &'a Engine) -> Result<Self, Error> {
        let status = unsafe { FwpmTransactionBegin0(engine.handle(), 0) };
        if status != STATUS_SUCCESS.0 as u32 {
            return Err(Error::Transaction {
                reason: format!("FwpmTransactionBegin0 failed: {}", win32_error(status)),
            });
        }

        Ok(Transaction {
            engine,
            completed: false,
        })
    }

    /// Commit the transaction.
    pub fn commit(&mut self) -> Result<(), Error> {
        if !self.completed {
            self.completed = true; // Consider it completed, even if it fails to commit
            let status = unsafe { FwpmTransactionCommit0(self.engine.handle()) };
            if status != STATUS_SUCCESS.0 as u32 {
                return Err(Error::Transaction {
                    reason: format!("FwpmTransactionCommit0 failed: {}", win32_error(status)),
                });
            }
        }
        Ok(())
    }

    /// Abort the transaction.
    pub fn abort(&mut self) -> Result<(), Error> {
        if !self.completed {
            self.completed = true; // Consider it completed, even if it fails to abort
            let status = unsafe { FwpmTransactionAbort0(self.engine.handle()) };
            if status != STATUS_SUCCESS.0 as u32 {
                return Err(Error::Transaction {
                    reason: format!("FwpmTransactionAbort0 failed: {}", win32_error(status)),
                });
            }
        }
        Ok(())
    }
}

impl<'a> Drop for Transaction<'a> {
    fn drop(&mut self) {
        if let Err(e) = self.abort() {
            tracing::error!("Error during WFP transaction drop: {e}");
        }
    }
}
