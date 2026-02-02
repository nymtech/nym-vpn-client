// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};

#[repr(u8)]
pub enum AuthenticaticationResult {
    Accepted = 0,
    Denied = 1,
}

impl From<AuthenticaticationResult> for u8 {
    fn from(value: AuthenticaticationResult) -> Self {
        value as u8
    }
}

impl From<u8> for AuthenticaticationResult {
    fn from(value: u8) -> Self {
        if value == 0 {
            AuthenticaticationResult::Accepted
        } else {
            AuthenticaticationResult::Denied
        }
    }
}

impl AuthenticaticationResult {
    pub async fn send(self, stream: &mut UnixStream) {
        stream.write_u8(self.into()).await.ok();
    }

    pub async fn recv(stream: &mut UnixStream) -> Self {
        stream
            .read_u8()
            .await
            .map(Into::into)
            .unwrap_or(Self::Denied)
    }

    pub fn accepted(&self) -> bool {
        matches!(self, Self::Accepted)
    }
}
