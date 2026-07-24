// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

use serde::{Serialize, de::DeserializeOwned};

use crate::FavoritesError;

pub(crate) async fn parse_from_file<T>(file_path: &PathBuf) -> Option<T>
where
    T: DeserializeOwned,
{
    let bytes = tokio::fs::read(file_path)
        .await
        .inspect_err(|err| {
            tracing::warn!("Could not load data from {file_path:?}, clearing it out: {err:?}")
        })
        .ok()?;
    serde_json::from_slice(&bytes)
        .inspect_err(|err| {
            tracing::warn!("Could not decode data from {file_path:?}, clearing it out: {err:?}")
        })
        .ok()
}

pub(crate) async fn save_to_file<T>(value: &T, file_path: &PathBuf) -> Result<(), FavoritesError>
where
    T: std::fmt::Debug + ?Sized + Serialize,
{
    let contents = serde_json::to_vec(value)
        .inspect_err(|err| tracing::warn!("Could not serialize {value:?}: {err:?}"))?;
    let ret = tokio::fs::write(file_path, contents).await;
    if let Err(err) = &ret {
        tracing::warn!("Could not flush data to {file_path:?}: {err:?}");
    } else {
        tracing::debug!("Data written to {file_path:?}");
    }
    Ok(ret?)
}
