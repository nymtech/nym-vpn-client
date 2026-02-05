// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(test, derive(strum::EnumIter))]
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

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn enum_to_u8() {
        for (idx, res) in AuthenticaticationResult::iter().enumerate() {
            assert_eq!(idx as u8, res.into());
        }
    }

    #[test]
    fn u8_to_enum() {
        let zero = AuthenticaticationResult::from(0);
        assert!(matches!(zero, AuthenticaticationResult::Accepted));
        assert!(zero.accepted());
        for idx in 1u8..255 {
            let other = AuthenticaticationResult::from(idx);
            assert!(matches!(other, AuthenticaticationResult::Denied));
            assert!(!other.accepted());
        }
    }

    #[tokio::test]
    async fn send_recv() {
        let (mut client, mut server) = UnixStream::pair().unwrap();

        for sent in AuthenticaticationResult::iter() {
            sent.send(&mut server).await;
            let received = AuthenticaticationResult::recv(&mut client).await;
            assert_eq!(sent, received);
        }
    }

    #[tokio::test]
    async fn no_value_means_denied() {
        let (mut client, _) = UnixStream::pair().unwrap();
        let received = AuthenticaticationResult::recv(&mut client).await;
        assert_eq!(received, AuthenticaticationResult::Denied);
    }
}
