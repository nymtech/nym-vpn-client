#[derive(Debug, thiserror::Error)]
pub enum VpnApiClientError {
    #[error(transparent)]
    HttpClientError(#[from] nym_http_api_client::HttpClientError),
}

pub type Result<T> = std::result::Result<T, VpnApiClientError>;
