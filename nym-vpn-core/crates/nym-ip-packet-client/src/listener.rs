// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use bytes::{Bytes, BytesMut};
use nym_ip_packet_requests::codec::MultiIpPacketCodec;
use nym_sdk::mixnet::{Recipient, ReconstructedMessage};
use tokio_util::codec::Decoder;
use tracing::{debug, error, info, warn};

use crate::{
    current::{
        request::{IpPacketRequest, IpPacketRequestData},
        response::{InfoLevel, IpPacketResponse, IpPacketResponseData},
    },
    helpers::check_ipr_message_version,
};

pub enum MixnetMessageOutcome {
    IpPackets(Vec<Bytes>),
    MixnetSelfPing,
}

pub struct IprListener {
    our_address: Recipient,
    decoder: MultiIpPacketCodec,
}

#[derive(Debug, thiserror::Error)]
pub enum IprListenerError {
    #[error(transparent)]
    IprClientError(#[from] crate::Error),
}

impl IprListener {
    pub fn new(our_address: Recipient) -> Self {
        let decoder = MultiIpPacketCodec::new(nym_ip_packet_requests::codec::BUFFER_TIMEOUT);
        Self {
            our_address,
            decoder,
        }
    }

    fn is_mix_self_ping(&self, request: &IpPacketRequest) -> bool {
        match request.data {
            IpPacketRequestData::Ping(ref ping_request)
                if ping_request.reply_to == self.our_address =>
            {
                true
            }
            ref request => {
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
            Ok(response) => match response.data {
                IpPacketResponseData::StaticConnect(_) => {
                    info!("Received static connect response when already connected - ignoring");
                }
                IpPacketResponseData::DynamicConnect(_) => {
                    info!("Received dynamic connect response when already connected - ignoring");
                }
                IpPacketResponseData::Disconnect(_) => {
                    // Disconnect is not yet handled on the IPR side anyway
                    info!("Received disconnect response, ignoring for now");
                }
                IpPacketResponseData::UnrequestedDisconnect(_) => {
                    info!("Received unrequested disconnect response, ignoring for now");
                }
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
                IpPacketResponseData::Pong(_) => {
                    info!("Received pong response, ignoring for now");
                }
                IpPacketResponseData::Health(_) => {
                    info!("Received health response, ignoring for now");
                }
                IpPacketResponseData::Info(info) => {
                    let msg = format!("Received info response from the mixnet: {}", info.reply);
                    match info.level {
                        InfoLevel::Info => info!("{msg}"),
                        InfoLevel::Warn => warn!("{msg}"),
                        InfoLevel::Error => error!("{msg}"),
                    }
                }
            },
            Err(err) => {
                // The exception to when we are not expecting a response, is when we
                // are sending a ping to ourselves.
                if let Ok(request) = IpPacketRequest::from_reconstructed_message(&message) {
                    if self.is_mix_self_ping(&request) {
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
