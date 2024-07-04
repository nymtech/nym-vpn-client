#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("got reply for connect request, but it appears intended for the wrong address?")]
    GotReplyIntendedForWrongAddress,

    #[error("mixnet client stopped returning responses")]
    NoMixnetMessagesReceived,

    #[error("failed to get version from message")]
    NoVersionInMessage,

    #[error("received response with version v{received}, the client is too new and can only understand v{expected}")]
    ReceivedResponseWithOldVersion { expected: u8, received: u8 },

    #[error("received response with version v{received}, the client is too old and can only understand v{expected}")]
    ReceivedResponseWithNewVersion { expected: u8, received: u8 },

    #[error("timeout waiting for connect response from exit gateway (authenticator)")]
    TimeoutWaitingForConnectResponse,
}

// Result type based on our error type
pub type Result<T> = std::result::Result<T, Error>;
