use nym_windows::process::ClientProcess;
use tokio::net::windows::named_pipe::NamedPipeServer;
use tokio_util::sync::CancellationToken;

use crate::{
    auth_result::{authorize, deny},
    authentication::error::AuthenticationError,
    named_pipe::Connector,
};

fn verify(stream: &Connector<NamedPipeServer>) -> Result<(), nym_windows::process::Error> {
    let client_process = ClientProcess::try_from(&stream.0)?;
    client_process.verify()?;
    Ok(())
}

pub(crate) async fn is_authenticated(
    mut stream: Connector<NamedPipeServer>,
    _shutdown_token: CancellationToken,
) -> Result<Connector<NamedPipeServer>, AuthenticationError> {
    if let Err(err) = verify(&stream) {
        tracing::debug!("Client certificate verification failed: {err:?}");
        deny(stream).await;
        Err(AuthenticationError::AuthorizationDenied)
    } else {
        authorize(&mut stream).await;
        Ok(stream)
    }
}
