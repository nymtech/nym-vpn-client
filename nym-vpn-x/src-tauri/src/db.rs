use anyhow::{anyhow, Result};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use sled::IVec;
use std::{
    fmt::{self, Display},
    fs::create_dir_all,
    io,
    path::PathBuf,
};
use strum::{AsRefStr, EnumString};
use tauri::api::path::data_dir;
use thiserror::Error;
use tracing::{error, info, instrument, warn};
use ts_rs::TS;

use crate::APP_DIR;

const DB_DIR: &str = "db";

pub type JsonValue = Value;

#[allow(dead_code)]
#[derive(Deserialize, Serialize, AsRefStr, EnumString, Debug, Clone, Copy, TS)]
#[ts(export)]
pub enum Key {
    #[strum(serialize = "monitoring")]
    Monitoring,
    #[strum(serialize = "autoconnect")]
    Autoconnect,
    #[strum(serialize = "entry_location_enabled")]
    EntryLocationEnabled,
    #[strum(serialize = "ui_theme")]
    UiTheme,
    #[strum(serialize = "ui_root_font_size")]
    UiRootFontSize,
    #[strum(serialize = "vpn_mode")]
    VpnMode,
    #[strum(serialize = "entry_node_location")]
    EntryNodeLocation,
    #[strum(serialize = "exit_node_location")]
    ExitNodeLocation,
    #[strum(serialize = "window_size")]
    WindowSize,
    #[strum(serialize = "window_position")]
    WindowPosition,
    #[strum(serialize = "welcome_screen_seen")]
    WelcomeScreenSeen,
    #[strum(serialize = "credential_expiry")]
    CredentialExpiry,
    #[strum(serialize = "desktop_notifications")]
    DesktopNotifications,
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
        let mut path = data_dir()
            .ok_or(anyhow!("failed to retrieve data directory path"))
            .inspect_err(|e| error!("failed to retrieve data directory path: {e}"))?;
        path.push(APP_DIR);
        path.push(DB_DIR);
        info!("opening sled db at {}", path.display());
        create_dir_all(&path).map_err(|e| {
            error!("failed to create db directory {}", path.display());
            DbError::Io(e)
        })?;
        // TODO handle db recovery
        let db = sled::open(&path)
            .inspect_err(|e| error!("failed to open sled db from path {}: {e}", path.display()))?;
        if db.was_recovered() {
            info!("sled db recovered");
        } else {
            info!("new sled db created");
        }
        Ok(Self { db, path })
    }

    /// Discard deserialization errors by removing the key
    #[instrument(skip(self))]
    fn discard_deserialize<T>(
        &self,
        key: Key,
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
    pub fn get_raw(&self, key: Key) -> Result<Option<IVec>, DbError> {
        self.db.get(key.as_ref()).map_err(|e| {
            error!("failed to get key [{key}]: {e}");
            DbError::Db(e)
        })
    }

    /// Get the value for a key as a deserialized type
    #[instrument(skip(self))]
    pub fn get_typed<T>(&self, key: Key) -> Result<Option<T>, DbError>
    where
        T: DeserializeOwned + fmt::Debug,
    {
        let res = self
            .get_raw(key)?
            .map(|v| serde_json::from_slice::<T>(&v))
            .transpose()
            .map_err(|e| {
                error!("failed to deserialize value for key [{key}]: {e}");
                DbError::Deserialize(e)
            });

        info!("get key [{key}] with value {res:?}");
        self.discard_deserialize(key, res)
    }

    /// Get the value for a key as a deserialized JSON value
    #[instrument(skip(self))]
    pub fn get(&self, key: Key) -> Result<Option<JsonValue>, DbError> {
        let res = self
            .get_raw(key)?
            .map(|v| serde_json::from_slice::<Value>(&v))
            .transpose()
            .map_err(|e| {
                error!("failed to deserialize value for key [{key}]: {e}");
                DbError::Deserialize(e)
            });

        info!("get key [{key}] with value {res:?}");
        self.discard_deserialize(key, res)
    }

    /// Insert a key to a new JSON value returning the previous value if any
    #[instrument(skip(self))]
    pub fn insert<T>(&self, key: Key, value: T) -> Result<Option<JsonValue>, DbError>
    where
        T: Serialize + std::fmt::Debug,
    {
        let json_value = serde_json::to_vec(&value).map_err(|e| {
            error!("failed to serialize value for [{key}]: {e}");
            DbError::Serialize(format!("failed to serialize value for [{key}]: {e}"))
        })?;
        let res = self
            .db
            .insert(key.as_ref(), json_value)?
            .map(|v| serde_json::from_slice::<Value>(&v))
            .transpose()
            .map_err(|e| {
                error!("failed to deserialize value for key [{key}]: {e}");
                DbError::Deserialize(e)
            });

        // flush db in the background
        let db = self.db.clone();
        tokio::spawn(async move {
            let _ = db.flush_async().await.inspect_err(|e| {
                error!("failed to flush: {e}");
            });
            info!("flushed db");
        });

        info!("inserted key [{key}] with value {value:?}");
        self.discard_deserialize(key, res)
    }

    /// Remove a key returning the previous value if any
    #[instrument(skip(self))]
    pub fn remove_raw(&self, key: Key) -> Result<Option<IVec>, DbError> {
        self.db.remove(key.as_ref()).map_err(|e| {
            error!("failed to remove key [{key}]: {e}");
            DbError::Db(e)
        })
    }

    /// Remove a key returning the previous value if any
    #[instrument(skip(self))]
    pub fn remove(&self, key: Key) -> Result<Option<JsonValue>, DbError> {
        let res = self
            .db
            .remove(key.as_ref())?
            .map(|v| serde_json::from_slice::<Value>(&v))
            .transpose()
            .map_err(|e| {
                error!("failed to deserialize value for key [{key}]: {e}");
                DbError::Deserialize(e)
            });

        // flush db in the background
        let db = self.db.clone();
        tokio::spawn(async move {
            let _ = db.flush_async().await.inspect_err(|e| {
                error!("failed to flush: {e}");
            });
            info!("flushed db");
        });

        info!("removed key [{key}]");
        self.discard_deserialize(key, res)
    }

    /// Asynchronously flushes all dirty IO buffers and calls fsync
    #[instrument(skip(self))]
    pub async fn flush(&self) -> Result<usize, DbError> {
        self.db.flush_async().await.map_err(|e| {
            error!("failed to flush: {e}");
            DbError::Db(e)
        })
    }
}
