use crate::DEFAULT_NETSTATS_ENABLED;
use crate::fs::path::APP_DATA_DIR;

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use sled::IVec;
use std::{
    fmt::{self, Display},
    fs::create_dir_all,
    io,
    path::PathBuf,
};
use strum::{AsRefStr, EnumIter};
use thiserror::Error;
use tracing::{debug, error, info, instrument, warn};
use ts_rs::TS;

const DB_DIR: &str = "db";

pub type JsonValue = Value;

#[allow(dead_code)]
#[derive(Deserialize, Serialize, AsRefStr, EnumIter, Debug, Clone, Copy, TS)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
#[ts(export, export_to = "tauri.ts", rename = "DbKey")]
pub enum Key {
    UiTheme,
    UiRootFontSize,
    UiLanguage,
    TechnicalOptinSeen,
    DesktopNotifications,
    LastNetworkEnv,
    NetworkStatsEnabled,
    CustomSplitTunnelApps,
    // favorites are handled by the core `nym-favorites` crate
    // (see commands::favorites), not stored in this k/v db.
    // some data cache (no semantic difference)
    CacheMxEntryGateways,
    CacheMxExitGateways,
    CacheWgGateways,
    CacheAccountId,
    CacheDeviceId,
}

impl Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

/// Sled db wrapper, embedded k/v store
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Db {
    pub db: sled::Db,
    pub path: PathBuf,
}

#[derive(Error, Debug)]
pub enum DbError {
    #[error("lock error {0}")]
    Locked(String),
    #[error("IO error {0}")]
    Io(#[from] io::Error),
    #[error("db error {0}")]
    Db(#[from] sled::Error),
    #[error("deserialize error {0}")]
    Deserialize(#[from] serde_json::Error),
    #[error("serialize error {0}")]
    Serialize(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl Db {
    #[instrument]
    pub fn new() -> Result<Self, DbError> {
        let path = APP_DATA_DIR
            .clone()
            .ok_or(anyhow!("failed to get app data dir"))?
            .join(DB_DIR);
        info!("opening db at {}", path.display());
        create_dir_all(&path).map_err(|e| {
            error!("failed to create db directory {}", path.display());
            DbError::Io(e)
        })?;
        let db = sled::open(&path).map_err(|e| {
            error!("failed to open sled db {}: {e}", path.display());
            match &e {
                sled::Error::Io(io_err) => {
                    if io_err.kind() == io::ErrorKind::Other
                        && io_err.to_string().starts_with("could not acquire lock")
                    {
                        // this error happens when it failed to acquire db lock, ie db is already locked
                        // by another running app instance
                        return DbError::Locked(e.to_string());
                    }
                    DbError::Db(e)
                }
                _ => DbError::Db(e),
            }
        })?;
        if db.was_recovered() {
            info!("using existing db at {}", &path.display());
        } else {
            info!("new db created at {}", &path.display());
        }
        Ok(Self { db, path })
    }

    /// Init some keys if they do not exist
    #[instrument(skip(self))]
    pub fn set_defaults(&self) -> Result<(), DbError> {
        let network_stats_enabled = self.get(Key::NetworkStatsEnabled.as_ref()).ok().flatten();
        if network_stats_enabled.is_none() {
            self.insert(Key::NetworkStatsEnabled.as_ref(), DEFAULT_NETSTATS_ENABLED)?;
        }
        Ok(())
    }

    /// Discard deserialization errors by removing the key
    #[instrument(skip(self))]
    fn discard_deserialize<T>(
        &self,
        key: &str,
        result: Result<Option<T>, DbError>,
    ) -> Result<Option<T>, DbError>
    where
        T: DeserializeOwned + fmt::Debug,
    {
        if let Err(DbError::Deserialize(e)) = result {
            warn!("removing key [{key}] due to deserialization error: {e}");
            self.remove_raw(key)?;
            return Ok(None);
        }
        result
    }

    /// Get the value for a key as raw bytes
    #[instrument(skip(self))]
    pub fn get_raw(&self, key: &str) -> Result<Option<IVec>, DbError> {
        self.db.get(key).map_err(|e| {
            error!("failed to get key [{key}]: {e}");
            DbError::Db(e)
        })
    }

    /// Get the value for a key as a deserialized type
    #[instrument(skip(self))]
    pub fn get_typed<T>(&self, key: &str) -> Result<Option<T>, DbError>
    where
        T: DeserializeOwned + fmt::Debug,
    {
        let res = self
            .get_raw(key)?
            .map(|v| serde_json::from_slice::<T>(&v))
            .transpose()
            .map_err(DbError::Deserialize);

        Db::get_log(key, &res);
        self.discard_deserialize(key, res)
    }

    /// Get the value for a key as a deserialized JSON value
    #[instrument(skip(self))]
    pub fn get(&self, key: &str) -> Result<Option<JsonValue>, DbError> {
        let res = self
            .get_raw(key)?
            .map(|v| serde_json::from_slice::<Value>(&v))
            .transpose()
            .map_err(DbError::Deserialize);

        Db::get_log(key, &res);
        self.discard_deserialize(key, res)
    }

    /// Insert a key to a new JSON value returning the previous value if any
    #[instrument(skip(self, value))]
    pub fn insert<T>(&self, key: &str, value: T) -> Result<Option<JsonValue>, DbError>
    where
        T: Serialize + fmt::Debug,
    {
        let json_value = serde_json::to_vec(&value).map_err(|e| {
            error!("failed to serialize value for [{key}]: {e}");
            DbError::Serialize(format!("failed to serialize value for [{key}]: {e}"))
        })?;
        let res = self
            .db
            .insert(key, json_value)
            .inspect_err(|e| error!("failed to insert key [{key}]: {e}"))?
            .map(|v| serde_json::from_slice::<Value>(&v))
            .transpose()
            .map_err(|e| {
                error!("failed to deserialize value for key [{key}]: {e}");
                DbError::Deserialize(e)
            });

        if key.starts_with("cache-") {
            // some cached data like gateways list are BIG, skip them
            debug!("set key [{key}]");
        } else if Self::is_sensitive_key(key) {
            debug!("set key [{key}] value=<redacted_paths>");
        } else {
            debug!("set key [{key}] value={:?}", value);
        }
        self.discard_deserialize(key, res)
    }

    /// Remove a key returning the previous value if any
    #[instrument(skip(self))]
    pub fn remove_raw(&self, key: &str) -> Result<Option<IVec>, DbError> {
        self.db.remove(key).map_err(|e| {
            error!("failed to remove key [{key}]: {e}");
            DbError::Db(e)
        })
    }

    /// Remove a key returning the previous value if any
    #[instrument(skip(self))]
    pub fn remove(&self, key: &str) -> Result<Option<JsonValue>, DbError> {
        let res = self
            .db
            .remove(key)
            .inspect_err(|e| error!("failed to remove key [{key}]: {e}"))?
            .map(|v| serde_json::from_slice::<Value>(&v))
            .transpose()
            .map_err(|e| {
                error!("failed to deserialize value for key [{key}]: {e}");
                DbError::Deserialize(e)
            });

        debug!("del key [{key}]");
        self.discard_deserialize(key, res)
    }

    /// Asynchronously flushes all dirty IO buffers and calls fsync
    #[instrument(skip(self))]
    pub async fn flush(&self) -> Result<usize, DbError> {
        debug!("flush");
        self.db.flush_async().await.map_err(|e| {
            error!("failed to flush: {e}");
            DbError::Db(e)
        })
    }

    fn get_log<T>(key: &str, value: &Result<Option<T>, DbError>)
    where
        T: DeserializeOwned + fmt::Debug,
    {
        match &value {
            Ok(Some(v)) => {
                if key.starts_with("cache") {
                    debug!("get key [{key}] SOMEVAL");
                } else if Self::is_sensitive_key(key) {
                    debug!("get key [{key}] <redacted_paths>");
                } else {
                    debug!("get key [{key}] {v:?}");
                }
            }
            Ok(None) => {
                debug!("get key [{key}] NOTSET");
            }
            Err(e) => {
                error!("failed to get key [{key}]: {e}");
            }
        }
    }

    fn is_sensitive_key(key: &str) -> bool {
        key == Key::CustomSplitTunnelApps.as_ref()
    }
}
