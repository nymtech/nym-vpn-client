use anyhow::{anyhow, Context, Result};
use serde::{de::DeserializeOwned, Serialize};
use std::{fmt, fs, path::PathBuf, str};
use tracing::{debug, error, instrument};

use super::util::{check_dir, check_file};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AppStorage<T>
where
    T: Serialize + DeserializeOwned + Default + fmt::Debug,
{
    pub data: T,
    pub dir_path: PathBuf,
    pub filename: String,
    pub full_path: PathBuf,
}

impl<T> AppStorage<T>
where
    T: Serialize + DeserializeOwned + Default + fmt::Debug,
{
    pub fn new(dir_path: PathBuf, filename: &str, data: Option<T>) -> Result<Self> {
        let mut full_path = dir_path.clone();
        full_path.push(filename);

        // check if the directory exists, if not create it
        check_dir(&dir_path)?;
        // check if the file exists, if not create it
        check_file(&full_path)?;

        Ok(Self {
            data: data.unwrap_or_default(),
            dir_path,
            filename: filename.to_owned(),
            full_path,
        })
    }

    #[instrument]
    pub fn read(&self) -> Result<T> {
        debug!("reading stored data from {}", self.full_path.display());
        let content = fs::read(&self.full_path)
            .inspect_err(|e| error!("Failed to read file `{}`: {e}", &self.full_path.display()))
            .context(format!("Failed to read file {}", self.full_path.display()))?;

        toml::from_str::<T>(str::from_utf8(&content)?).map_err(|e| {
            error!("{e}");
            anyhow!("{e}")
        })
    }

    #[instrument]
    pub fn write(&self) -> Result<()> {
        debug!("writing data to {}", self.full_path.display());
        let toml = toml::to_string(&self.data)?;
        fs::write(&self.full_path, toml)
            .inspect_err(|e| error!("Failed to write to `{}`: {e}", &self.full_path.display()))?;
        Ok(())
    }

    #[instrument]
    pub fn clear(&self) -> Result<()> {
        debug!("clearing data {}", self.full_path.display());
        fs::write(&self.full_path, vec![])
            .inspect_err(|e| error!("Failed to write to `{}`: {e}", &self.full_path.display()))?;
        Ok(())
    }
}
