#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    SdkError(#[from] nym_sdk::Error),

    #[error("failed to obtain websocket for bypass")]
    NoWebSocket,
}

// Result type based on our error type
pub type Result<T> = std::result::Result<T, Error>;
