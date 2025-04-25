#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Mixnet client stopped returning responses")]
    NoMixnetMessagesReceived,

    #[error("Failed to get version from message")]
    NoVersionInMessage,

    #[error("Received response with version v{received}, the client is too new and can only understand v{expected}")]
    ReceivedResponseWithOldVersion { expected: u8, received: u8 },

    #[error("Received response with version v{received}, the client is too old and can only understand v{expected}")]
    ReceivedResponseWithNewVersion { expected: u8, received: u8 },

    #[error("Failed to send mixnet message")]
    SendMixnetMessage(#[source] nym_sdk::Error),

    #[error("Timeout waiting for connect response from exit gateway (authenticator)")]
    TimeoutWaitingForConnectResponse,

    #[error("Unable to get mixnet handle when sending authenticator message")]
    UnableToGetMixnetHandle,

    #[error("Unknown version number")]
    UnknownVersion,

    #[error(transparent)]
    Bincode(#[from] bincode::Error),

    #[error("Gateway doesn't support this type of message")]
    UnsupportedMessage,

    #[error(transparent)]
    AuthenticatorRequests(#[from] nym_authenticator_requests::Error),
}

// Result type based on our error type
pub type Result<T> = std::result::Result<T, Error>;
