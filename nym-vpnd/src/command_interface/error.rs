#[derive(Debug, thiserror::Error)]
pub(crate) enum CommandInterfaceError {
    #[error("failed to parse DNS IP address: {ip}")]
    FailedToParseDnsIp {
        ip: String,
        source: std::net::AddrParseError,
    },
}
