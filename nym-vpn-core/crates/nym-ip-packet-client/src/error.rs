// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::current::response::{DynamicConnectFailureReason, StaticConnectFailureReason};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    SdkError(#[from] nym_sdk::Error),

    #[error("received response with version v{received}, the client is too new and can only understand v{expected}")]
    ReceivedResponseWithOldVersion { expected: u8, received: u8 },

    #[error("received response with version v{received}, the client is too old and can only understand v{expected}")]
    ReceivedResponseWithNewVersion { expected: u8, received: u8 },

    #[error("got reply for connect request, but it appears intended for the wrong address?")]
    GotReplyIntendedForWrongAddress,

    #[error("unexpected connect response")]
    UnexpectedConnectResponse,

    #[error("mixnet client stopped returning responses")]
    NoMixnetMessagesReceived,

    #[error("timeout waiting for connect response from exit gateway (ipr)")]
    TimeoutWaitingForConnectResponse,

    #[error("connection cancelled")]
    Cancelled,

    #[error("connect request denied: {reason}")]
    StaticConnectRequestDenied { reason: StaticConnectFailureReason },

    #[error("connect request denied: {reason}")]
    DynamicConnectRequestDenied { reason: DynamicConnectFailureReason },

    #[error("failed to get version from message")]
    NoVersionInMessage,

    #[error("already connected to the mixnet")]
    AlreadyConnected,
}

// Result type based on our error type
pub type Result<T> = std::result::Result<T, Error>;
