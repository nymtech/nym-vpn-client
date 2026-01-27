// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use serde::{de::DeserializeOwned, ser::Serialize};

use crate::{Error, Result};
use std::{fs::File, path::Path};

/// Deserialize a value from JSON file.
pub fn deserialize_from_json_file<S, T>(path: S) -> Result<T>
where
    S: AsRef<Path>,
    T: DeserializeOwned,
{
    let file = std::fs::File::open(&path).map_err(|source| Error::OpenFile {
        path: path.as_ref().to_path_buf(),
        source,
    })?;
    let reader = std::io::BufReader::new(file);

    serde_json::from_reader(reader).map_err(|source| Error::Deserialize {
        path: path.as_ref().to_path_buf(),
        source,
    })
}

/// Serialize a value to a JSON file.
pub fn serialize_to_json_file<S, T>(path: S, value: &T) -> Result<File>
where
    T: ?Sized + Serialize,
    S: AsRef<Path>,
{
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)
        .map_err(|source| Error::OpenFile {
            path: path.as_ref().to_path_buf(),
            source,
        })?;

    serde_json::to_writer_pretty(&file, &value).map_err(|source| Error::Serialize {
        path: path.as_ref().to_path_buf(),
        source,
    })?;

    Ok(file)
}
