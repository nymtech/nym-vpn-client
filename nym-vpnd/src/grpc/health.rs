use std::time::Duration;

use tokio::{sync::mpsc, time::sleep};
use tonic::{Request, Response, Status};

use nym_vpn_proto::{
    health_check_response::ServingStatus, health_server::Health, HealthCheckRequest,
    HealthCheckResponse,
};
use tokio_stream::wrappers::ReceiverStream;
use tracing::info;

const HEALTH_CHECK_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Debug, Default)]
pub struct HealthService {}

#[tonic::async_trait]
impl Health for HealthService {
    type WatchStream = ReceiverStream<Result<HealthCheckResponse, Status>>;

    #[tracing::instrument]
    async fn check(
        &self,
        request: Request<HealthCheckRequest>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        info!("received request {:?}", request);

        // TODO check if the `NymVpnd` service is running/healthy
        Ok(Response::new(HealthCheckResponse {
            status: ServingStatus::Serving as i32,
        }))
    }

    #[tracing::instrument]
    async fn watch(
        &self,
        request: Request<HealthCheckRequest>,
    ) -> Result<Response<Self::WatchStream>, Status> {
        info!("received request {:?}", request);

        let (tx, rx) = mpsc::channel(32);

        tokio::spawn(async move {
            loop {
                // TODO check if the `NymVpnd` service is running/healthy
                tx.send(Ok(HealthCheckResponse {
                    status: ServingStatus::Serving as i32,
                }))
                .await
                .unwrap();

                sleep(HEALTH_CHECK_INTERVAL).await;
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
