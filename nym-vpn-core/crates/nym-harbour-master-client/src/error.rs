use crate::client::HarbourMasterApiError;

#[derive(Debug, thiserror::Error)]
pub enum HarbourMasterError {
    #[error("api error: {0}")]
    HarbourMasterApiError(#[from] HarbourMasterApiError),
}

pub type Result<T> = std::result::Result<T, HarbourMasterError>;
