use nym_windows::process::ClientProcess;
use tokio::net::windows::named_pipe::NamedPipeServer;
use tokio_util::sync::CancellationToken;

use crate::{authentication::error::AuthenticationError, named_pipe::Connector};

fn verify(
    stream: &Connector<NamedPipeServer>,
    nym_certificate_serial_number: String,
) -> Result<(), nym_windows::process::Error> {
    let client_process = ClientProcess::try_from(&stream.0)?;
    client_process.verify_certificate_signature(nym_certificate_serial_number)?;
    Ok(())
}

// Check the stream is from a binary signed by Nym
pub(crate) async fn is_authenticated(
    stream: &mut Connector<NamedPipeServer>,
    nym_certificate_serial_number: String,
    _shutdown_token: CancellationToken,
) -> Result<(), AuthenticationError> {
    if let Err(err) = verify(stream, nym_certificate_serial_number) {
        tracing::debug!("Client certificate verification failed: {err:?}");
        Err(AuthenticationError::AuthorizationDenied)
    } else {
        Ok(())
    }
}
