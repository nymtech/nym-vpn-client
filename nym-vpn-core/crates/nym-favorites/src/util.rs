// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

use serde::{Serialize, de::DeserializeOwned};

pub(crate) async fn persisted<T>(file_path: &PathBuf) -> Option<T>
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

pub(crate) async fn flush<T>(value: &T, file_path: &PathBuf)
where
    T: std::fmt::Debug + ?Sized + Serialize,
{
    let Some(contents) = serde_json::to_vec(value)
        .inspect_err(|err| tracing::warn!("Could not serialize {value:?}: {err:?}"))
        .ok()
    else {
        return;
    };
    if let Err(err) = tokio::fs::write(file_path, contents).await {
        tracing::warn!("Could not flush data to {file_path:?}: {err:?}");
    } else {
        tracing::debug!("Data written to {file_path:?}");
    }
}
