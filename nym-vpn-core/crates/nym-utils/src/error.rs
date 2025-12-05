#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum UtilsError {
    #[error(transparent)]
    Hex(#[from] hex::FromHexError),

    #[error(transparent)]
    Bip39(#[from] bip39::Error),
}
