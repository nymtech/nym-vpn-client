// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

// This module primarily handles conversions to protobuf types

pub(crate) mod account;
pub(crate) mod connection_state;
pub(crate) mod error;
pub(crate) mod info_response;
pub(crate) mod state_response;
pub(crate) mod status_update;
pub(crate) mod tunnel_state;

/// Infallible conversion to protobuf type
pub trait IntoProtobuf {
    type ProtobufType;

    fn to_protobuf(self) -> Self::ProtobufType;
}
