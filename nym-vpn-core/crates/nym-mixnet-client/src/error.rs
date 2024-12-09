#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    SdkError(#[from] nym_sdk::Error),
}

// Result type based on our error type
pub type Result<T> = std::result::Result<T, Error>;
