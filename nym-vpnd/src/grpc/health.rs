use tonic::{Request, Response, Status};

use nym_vpn_proto::{
    health_check_response::ServingStatus, health_server::Health, HealthCheckRequest,
    HealthCheckResponse,
};
use tokio_stream::wrappers::ReceiverStream;
use tracing::info;

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

        unimplemented!()
    }
}
