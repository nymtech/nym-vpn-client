#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum UtilsError {
    #[error(transparent)]
    Bip39(#[from] bip39::Error),

    #[error(transparent)]
    Privy(#[from] nym_privy::error::PrivyError),
}
