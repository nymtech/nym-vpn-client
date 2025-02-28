// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use bytes::{Bytes, BytesMut};
use nym_ip_packet_requests::{codec::MultiIpPacketCodec, v8::response::ControlResponse};
use nym_sdk::mixnet::ReconstructedMessage;
use tokio_util::codec::Decoder;
use tracing::{debug, error, info, warn};

use crate::{
    current::{
        request::{ControlRequest, IpPacketRequest, IpPacketRequestData},
        response::{InfoLevel, IpPacketResponse, IpPacketResponseData},
    },
    helpers::check_ipr_message_version,
};

pub enum MixnetMessageOutcome {
    IpPackets(Vec<Bytes>),
    MixnetSelfPing,
}

pub struct IprListener {
    decoder: MultiIpPacketCodec,
}

#[derive(Debug, thiserror::Error)]
pub enum IprListenerError {
    #[error(transparent)]
    IprClientError(#[from] crate::Error),
}

impl IprListener {
    pub fn new() -> Self {
        let decoder = MultiIpPacketCodec::new(nym_ip_packet_requests::codec::BUFFER_TIMEOUT);
        Self { decoder }
    }

    fn is_mix_ping(&self, request: &IpPacketRequest) -> bool {
        match request.data {
            IpPacketRequestData::Control(ref control) => {
                matches!(**control, ControlRequest::Ping(_))
            }
            _ => {
                debug!("Received unexpected request: {request:?}");
                false
            }
        }
    }

    pub async fn handle_reconstructed_message(
        &mut self,
        message: ReconstructedMessage,
    ) -> Result<Option<MixnetMessageOutcome>, IprListenerError> {
        check_ipr_message_version(&message)?;

        match IpPacketResponse::from_reconstructed_message(&message) {
            Ok(response) => {
                match response.data {
                    IpPacketResponseData::Data(data_response) => {
                        // Un-bundle the mixnet message and send the individual IP packets
                        // to the tun device
                        let mut bytes = BytesMut::from(&*data_response.ip_packet);
                        let mut responses = vec![];
                        while let Ok(Some(packet)) = self.decoder.decode(&mut bytes) {
                            responses.push(packet);
                        }
                        return Ok(Some(MixnetMessageOutcome::IpPackets(responses)));
                    }
                    IpPacketResponseData::Control(control_response) => match *control_response {
                        ControlResponse::Connect(_) => {
                            info!("Received connect response when already connected - ignoring");
                        }
                        ControlResponse::Disconnect(_) => {
                            // Disconnect is not yet handled on the IPR side anyway
                            info!("Received disconnect response, ignoring for now");
                        }
                        ControlResponse::UnrequestedDisconnect(_) => {
                            info!("Received unrequested disconnect response, ignoring for now");
                        }
                        ControlResponse::Pong(_) => {
                            info!("Received pong response, ignoring for now");
                        }
                        ControlResponse::Health(_) => {
                            info!("Received health response, ignoring for now");
                        }
                        ControlResponse::Info(info) => {
                            let msg =
                                format!("Received info response from the mixnet: {}", info.reply);
                            match info.level {
                                InfoLevel::Info => info!("{msg}"),
                                InfoLevel::Warn => warn!("{msg}"),
                                InfoLevel::Error => error!("{msg}"),
                            }
                        }
                    },
                }
            }
            Err(err) => {
                // The exception to when we are not expecting a response, is when we
                // are sending a ping to ourselves.
                if let Ok(request) = IpPacketRequest::from_reconstructed_message(&message) {
                    if self.is_mix_ping(&request) {
                        return Ok(Some(MixnetMessageOutcome::MixnetSelfPing));
                    }
                } else {
                    warn!("Failed to deserialize reconstructed message: {err}");
                }
            }
        }
        Ok(None)
    }
}

impl Default for IprListener {
    fn default() -> Self {
        Self::new()
    }
}
