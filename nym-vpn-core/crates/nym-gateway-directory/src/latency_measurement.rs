// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use futures::{SinkExt as _, StreamExt as _};
use rand::{seq::SliceRandom, Rng};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tracing::{debug, info, trace, warn};
use tungstenite::Message;

use crate::Gateway;

// This latency check is basically lifted from nym-client-core, but modified to work with
// the types we use and removing the stuff we don't use. Obviously this is ripe for
// deduplication.

type WsConn = WebSocketStream<MaybeTlsStream<TcpStream>>;

const CONCURRENT_GATEWAYS_MEASURED: usize = 20;
const MEASUREMENTS: usize = 3;
const CONN_TIMEOUT: Duration = Duration::from_millis(1500);
const PING_TIMEOUT: Duration = Duration::from_millis(1000);

#[derive(thiserror::Error, Debug)]
pub enum LatencyMeasurementError {
    #[error("Gateway is missing address: {identity}")]
    MissingAddress {
        identity: Box<nym_sdk::mixnet::NodeIdentity>,
    },

    #[error("failed to connect to gateway: {0}")]
    GatewayConnectionError(#[from] tungstenite::Error),

    #[error("timeout trying to connect to gateway")]
    GatewayConnectionTimeout,

    #[error("gateway connection abruptly closed")]
    GatewayConnectionAbruptlyClosed,

    #[error("no gateway measurements for {identity} performed")]
    NoGatewayMeasurements { identity: String },
}

async fn connect(endpoint: &str) -> Result<WsConn, LatencyMeasurementError> {
    match tokio::time::timeout(CONN_TIMEOUT, connect_async(endpoint)).await {
        Err(_elapsed) => Err(LatencyMeasurementError::GatewayConnectionTimeout),
        Ok(Err(conn_failure)) => Err(conn_failure.into()),
        Ok(Ok((stream, _))) => Ok(stream),
    }
}

async fn measure_latency(gateway: &Gateway) -> Result<GatewayWithLatency, LatencyMeasurementError> {
    let addr = gateway
        .address
        .as_ref()
        .ok_or_else(|| LatencyMeasurementError::MissingAddress {
            identity: Box::new(*gateway.identity()),
        })?;

    trace!(
        "establishing connection to {} ({addr})...",
        gateway.identity
    );
    let mut stream = connect(addr).await?;

    let mut results = Vec::new();
    for _ in 0..MEASUREMENTS {
        let measurement_future = async {
            let ping_content = vec![1, 2, 3];
            let start = Instant::now();
            stream.send(Message::Ping(ping_content.clone())).await?;

            match stream.next().await {
                Some(Ok(Message::Pong(content))) => {
                    if content == ping_content {
                        let elapsed = Instant::now().duration_since(start);
                        trace!("current ping time: {elapsed:?}");
                        results.push(elapsed);
                    } else {
                        warn!("received a pong message with different content? wtf.")
                    }
                }
                Some(Ok(_)) => warn!("received a message that's not a pong!"),
                Some(Err(err)) => return Err(err.into()),
                None => return Err(LatencyMeasurementError::GatewayConnectionAbruptlyClosed),
            }

            Ok::<(), LatencyMeasurementError>(())
        };

        let timeout = tokio::time::sleep(PING_TIMEOUT);
        tokio::pin!(timeout);

        tokio::select! {
            _ = &mut timeout => {
                warn!("timed out while trying to perform measurement...")
            }
            res = measurement_future => res?,
        }
    }

    let count = results.len() as u64;
    if count == 0 {
        return Err(LatencyMeasurementError::NoGatewayMeasurements {
            identity: gateway.identity().to_base58_string(),
        });
    }

    let sum: Duration = results.into_iter().sum();
    let avg = Duration::from_nanos(sum.as_nanos() as u64 / count);

    Ok(GatewayWithLatency::new(gateway, avg))
}

pub(crate) async fn choose_gateway_by_latency<R: Rng>(
    rng: &mut R,
    gateways: &[Gateway],
) -> Result<Gateway, LatencyMeasurementError> {
    let gateways_with_latency = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    futures::stream::iter(gateways)
        .for_each_concurrent(CONCURRENT_GATEWAYS_MEASURED, |gateway| async {
            let id = *gateway.identity();
            trace!("measuring latency to {id}...");
            match measure_latency(gateway).await {
                Ok(with_latency) => {
                    debug!("{id}: {:?}", with_latency.latency);
                    gateways_with_latency.lock().await.push(with_latency);
                }
                Err(err) => {
                    warn!("failed to measure {id}: {err}");
                }
            };
        })
        .await;

    let gateways_with_latency = gateways_with_latency.lock().await;
    let chosen = gateways_with_latency
        .choose_weighted(rng, |item| 1. / item.latency.as_secs_f32())
        .expect("invalid selection weight!");

    info!(
        "chose gateway {} with average latency of {:?}",
        chosen.gateway.identity, chosen.latency
    );

    Ok(chosen.gateway.clone())
}

struct GatewayWithLatency<'a> {
    gateway: &'a Gateway,
    latency: Duration,
}

impl<'a> GatewayWithLatency<'a> {
    fn new(gateway: &'a Gateway, latency: Duration) -> Self {
        GatewayWithLatency { gateway, latency }
    }
}
