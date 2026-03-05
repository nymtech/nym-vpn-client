use nym_windows::process::ProcessCertVerifier;
use tokio::net::windows::named_pipe::NamedPipeServer;
use tokio_stream::Stream;

use crate::{
    authentication::{AuthenticationLayer, AuthenticationMaterial, error::AuthenticationError},
    named_pipe::Connector,
};

pub(crate) type Transport = Connector<tokio::net::windows::named_pipe::NamedPipeServer>;

fn verify(
    stream: &Connector<NamedPipeServer>,
    nym_certificate_serial_number: &str,
) -> Result<(), nym_windows::process::Error> {
    let client_process = ProcessCertVerifier::try_from(&stream.0)?;
    client_process.verify_certificate_signature(nym_certificate_serial_number)?;
    Ok(())
}

// Check the stream is from a binary signed by Nym
pub(crate) async fn is_authenticated(
    stream: &mut Transport,
    auth_material: AuthenticationMaterial,
) -> Result<(), AuthenticationError> {
    if let Err(err) = verify(stream, &auth_material.nym_certificate_serial_number) {
        tracing::debug!("Client certificate verification failed: {err:?}");
        Err(AuthenticationError::AuthorizationDenied)
    } else {
        Ok(())
    }
}

fn skip_authentication_checks(nym_certificate_serial_number: &str) -> bool {
    let proc = nym_windows::process::ProcessCertVerifier {
        pid: std::process::id(),
    };
    // if windows daemon process was signed, we expect the clients to be too
    if let Err(err) = proc.verify_certificate_signature(nym_certificate_serial_number) {
        tracing::debug!(
            "Own certificate signature verification failed: {err:?}, skipping client verification"
        );
        true
    } else {
        tracing::debug!(
            "Own binary with PID {} is signed, verifying client",
            proc.pid
        );
        false
    }
}

pub(crate) fn incoming(
    named_pipe: crate::named_pipe::NamedPipeListener,
    auth_material: AuthenticationMaterial,
) -> std::io::Result<impl Stream<Item = std::io::Result<Transport>>> {
    let listener = Box::pin(named_pipe.incoming()?);
    let auth_material = if skip_authentication_checks(&auth_material.nym_certificate_serial_number)
    {
        None
    } else {
        Some(auth_material)
    };
    let auth_layer = AuthenticationLayer::new(listener, auth_material);
    Ok(auth_layer.stream())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // Test builds are not signed and are authorized
    fn unsigned_build_authorized() {
        assert!(skip_authentication_checks(
            "4ec9356d8c87f9cf3ccf60e7bdad022f"
        ));
    }
}
