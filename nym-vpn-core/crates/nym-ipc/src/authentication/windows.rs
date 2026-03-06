use nym_windows::process::ClientProcess;
use tokio::net::windows::named_pipe::NamedPipeServer;
use tokio_stream::Stream;

use crate::{
    authentication::{AuthenticationLayer, error::AuthenticationError},
    named_pipe::Connector,
};

pub(crate) type Transport = Connector<tokio::net::windows::named_pipe::NamedPipeServer>;

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
    stream: &mut Transport,
    nym_certificate_serial_number: String,
) -> Result<(), AuthenticationError> {
    if let Err(err) = verify(stream, nym_certificate_serial_number) {
        tracing::debug!("Client certificate verification failed: {err:?}");
        Err(AuthenticationError::AuthorizationDenied)
    } else {
        Ok(())
    }
}

pub(crate) fn incoming(
    named_pipe: crate::named_pipe::NamedPipeListener,
    nym_certificate_serial_number: String,
) -> std::io::Result<impl Stream<Item = std::io::Result<Transport>>> {
    let listener = Box::pin(named_pipe.incoming()?);
    let auth_layer = AuthenticationLayer::new(listener, nym_certificate_serial_number);
    Ok(auth_layer.stream())
}
