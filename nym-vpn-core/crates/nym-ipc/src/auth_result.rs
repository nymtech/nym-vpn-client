// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(test, derive(strum::EnumIter))]
pub enum AuthenticaticationQuery {
    Undefined = 0,
    Status = 1,
}

impl From<AuthenticaticationQuery> for u8 {
    fn from(value: AuthenticaticationQuery) -> Self {
        value as u8
    }
}

impl From<u8> for AuthenticaticationQuery {
    fn from(value: u8) -> Self {
        if value == 1 {
            AuthenticaticationQuery::Status
        } else {
            AuthenticaticationQuery::Undefined
        }
    }
}

impl AuthenticaticationQuery {
    pub async fn query(mut stream: impl AsyncWrite + Unpin) {
        stream
            .write_u8(AuthenticaticationQuery::Status.into())
            .await
            .ok();
    }

    pub async fn recv(mut stream: impl AsyncRead + Unpin) -> Self {
        stream
            .read_u8()
            .await
            .map(Into::into)
            .unwrap_or(Self::Undefined)
    }

    pub fn status(&self) -> bool {
        matches!(self, Self::Status)
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(test, derive(strum::EnumIter))]
pub enum AuthenticaticationResult {
    Closed = 0,
    Accepted = 1,
    Denied = 2,
}

impl From<AuthenticaticationResult> for u8 {
    fn from(value: AuthenticaticationResult) -> Self {
        value as u8
    }
}

impl From<u8> for AuthenticaticationResult {
    fn from(value: u8) -> Self {
        if value == 0 {
            AuthenticaticationResult::Closed
        } else if value == 1 {
            AuthenticaticationResult::Accepted
        } else {
            AuthenticaticationResult::Denied
        }
    }
}

impl AuthenticaticationResult {
    pub async fn send(self, mut stream: impl AsyncWrite + Unpin) {
        stream.write_u8(self.into()).await.ok();
    }

    pub async fn recv(mut stream: impl AsyncRead + Unpin) -> Self {
        stream
            .read_u8()
            .await
            .map(Into::into)
            .unwrap_or(Self::Closed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn enum_to_u8() {
        for (idx, res) in AuthenticaticationQuery::iter().enumerate() {
            assert_eq!(idx as u8, res.into());
        }
        for (idx, res) in AuthenticaticationResult::iter().enumerate() {
            assert_eq!(idx as u8, res.into());
        }
    }

    #[test]
    fn u8_to_enum() {
        let zero = AuthenticaticationQuery::from(0);
        assert!(matches!(zero, AuthenticaticationQuery::Undefined));
        assert!(!zero.status());
        let one = AuthenticaticationQuery::from(1);
        assert!(matches!(one, AuthenticaticationQuery::Status));
        assert!(one.status());

        for idx in 2u8..255 {
            let other = AuthenticaticationQuery::from(idx);
            assert!(matches!(other, AuthenticaticationQuery::Undefined));
            assert!(!other.status());
        }

        let zero = AuthenticaticationResult::from(0);
        assert!(matches!(zero, AuthenticaticationResult::Closed));
        let one = AuthenticaticationResult::from(1);
        assert!(matches!(one, AuthenticaticationResult::Accepted));
        for idx in 2u8..255 {
            let other = AuthenticaticationResult::from(idx);
            assert!(matches!(other, AuthenticaticationResult::Denied));
        }
    }

    #[tokio::test]
    async fn send_recv() {
        let (mut client, mut server) = tokio::io::duplex(64);

        AuthenticaticationQuery::query(&mut client).await;
        let received = AuthenticaticationQuery::recv(&mut server).await;
        assert!(received.status());

        let (mut client, mut server) = tokio::io::duplex(64);

        for sent in AuthenticaticationResult::iter() {
            sent.send(&mut server).await;
            let received = AuthenticaticationResult::recv(&mut client).await;
            assert_eq!(sent, received);
        }
    }

    #[tokio::test]
    async fn no_value_means_closed() {
        let (mut client, _) = tokio::io::duplex(64);
        let received = AuthenticaticationResult::recv(&mut client).await;
        assert_eq!(received, AuthenticaticationResult::Closed);
    }

    #[tokio::test]
    async fn zero_means_undefined() {
        let (mut server, _) = tokio::io::duplex(64);
        let received = AuthenticaticationQuery::recv(&mut server).await;
        assert_eq!(received, AuthenticaticationQuery::Undefined);
    }
}
