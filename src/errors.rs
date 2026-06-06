use std::{num::ParseIntError, str::FromStr, sync::Arc};
use thiserror::Error;
use tokio_util::codec::LinesCodecError;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, Error)]
pub enum Error {
    #[error("Failed parsing argument of packet.")]
    Parse(#[from] ParseIntError),
    #[error("Packet did not have name field.")]
    NoName,
    #[error("Packet was not in the expected shape.")]
    MalformedPacket,
    #[error("IO error: {0}")]
    Io(#[from] Arc<std::io::Error>),
    #[error("Line codec error: {0}")]
    LinesCodec(#[from] Arc<LinesCodecError>),
    #[error("Failed to connect to {addr}: {source}")]
    Connect {
        addr: String,
        source: Arc<std::io::Error>,
    },
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(Arc::new(err))
    }
}

impl From<LinesCodecError> for Error {
    fn from(err: LinesCodecError) -> Self {
        Error::LinesCodec(Arc::new(err))
    }
}

/// Errors and warnings reported by the server.
#[derive(Debug, Clone, Copy, Error)]
pub enum ServerError {
    #[error("You are spamming the server.")]
    Spam,
    #[error("You are sending more than 1024 bytes to the server.")]
    PacketOverflow,
    #[error(
        "You didn't send a move packet. Note: This takes either your last move or UP (default)."
    )]
    NoMove,
    #[error("Your ip is already connected.")]
    MaxConnections,
    #[error("You didn't send the join packet.")]
    JoinTimeout,
    #[error("Instead of join you sent some other packet.")]
    ExpectedJoin,
    #[error("You used an invalid username.")]
    InvalidUsername,
    #[error("Username is too short.")]
    UsernameTooShort,
    #[error("Username is too long.")]
    UsernameTooLong,
    #[error("You used invalid symbols for the username.")]
    UsernameInvalidSymbols,
    #[error("You used an invalid password.")]
    InvalidPassword,
    #[error("Password is too short.")]
    PasswordTooShort,
    #[error("Password is too long.")]
    PasswordTooLong,
    #[error("You should not do this :^)")]
    NoPermission,
    #[error("Your password was wrong.")]
    WrongPassword,
    #[error("You are already connected with this player.")]
    AlreadyConnected,
    #[error("You did not send a valid move packet.")]
    UnknownMove,
    #[error("You are dead so shush :^)")]
    DeadCannotChat,
    #[error("Something is wrong with your chat message.")]
    InvalidChatMessage,
    #[error("Server does not know what you want :D")]
    UnknownPacket,
}

impl ServerError {
    /// Returns the wire-format code string for this error.
    pub fn code(&self) -> &'static str {
        match self {
            ServerError::Spam => "ERROR_SPAM",
            ServerError::PacketOverflow => "ERROR_PACKET_OVERFLOW",
            ServerError::NoMove => "ERROR_NO_MOVE",
            ServerError::MaxConnections => "ERROR_MAX_CONNECTIONS",
            ServerError::JoinTimeout => "ERROR_JOIN_TIMEOUT",
            ServerError::ExpectedJoin => "ERROR_EXPECTED_JOIN",
            ServerError::InvalidUsername => "ERROR_INVALID_USERNAME",
            ServerError::UsernameTooShort => "ERROR_USERNAME_TOO_SHORT",
            ServerError::UsernameTooLong => "ERROR_USERNAME_TOO_LONG",
            ServerError::UsernameInvalidSymbols => "ERROR_USERNAME_INVALID_SYMBOLS",
            ServerError::InvalidPassword => "ERROR_INVALID_PASSWORD",
            ServerError::PasswordTooShort => "ERROR_PASSWORD_TOO_SHORT",
            ServerError::PasswordTooLong => "ERROR_PASSWORD_TOO_LONG",
            ServerError::NoPermission => "ERROR_NO_PERMISSION",
            ServerError::WrongPassword => "ERROR_WRONG_PASSWORD",
            ServerError::AlreadyConnected => "ERROR_ALREADY_CONNECTED",
            ServerError::UnknownMove => "WARNING_UNKNOWN_MOVE",
            ServerError::DeadCannotChat => "ERROR_DEAD_CANNOT_CHAT",
            ServerError::InvalidChatMessage => "ERROR_INVALID_CHAT_MESSAGE",
            ServerError::UnknownPacket => "ERROR_UNKNOWN_PACKET",
        }
    }
}

impl FromStr for ServerError {
    type Err = Error;

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        let err = match s {
            "ERROR_SPAM" => ServerError::Spam,
            "ERROR_PACKET_OVERFLOW" => ServerError::PacketOverflow,
            "ERROR_NO_MOVE" => ServerError::NoMove,
            "ERROR_MAX_CONNECTIONS" => ServerError::MaxConnections,
            "ERROR_JOIN_TIMEOUT" => ServerError::JoinTimeout,
            "ERROR_EXPECTED_JOIN" => ServerError::ExpectedJoin,
            "ERROR_INVALID_USERNAME" => ServerError::InvalidUsername,
            "ERROR_USERNAME_TOO_SHORT" => ServerError::UsernameTooShort,
            "ERROR_USERNAME_TOO_LONG" => ServerError::UsernameTooLong,
            "ERROR_USERNAME_INVALID_SYMBOLS" => ServerError::UsernameInvalidSymbols,
            "ERROR_INVALID_PASSWORD" => ServerError::InvalidPassword,
            "ERROR_PASSWORD_TOO_SHORT" => ServerError::PasswordTooShort,
            "ERROR_PASSWORD_TOO_LONG" => ServerError::PasswordTooLong,
            "ERROR_NO_PERMISSION" => ServerError::NoPermission,
            "ERROR_WRONG_PASSWORD" => ServerError::WrongPassword,
            "ERROR_ALREADY_CONNECTED" => ServerError::AlreadyConnected,
            "WARNING_UNKNOWN_MOVE" => ServerError::UnknownMove,
            "ERROR_DEAD_CANNOT_CHAT" => ServerError::DeadCannotChat,
            "ERROR_INVALID_CHAT_MESSAGE" => ServerError::InvalidChatMessage,
            "ERROR_UNKNOWN_PACKET" => ServerError::UnknownPacket,
            _ => return Err(Error::MalformedPacket),
        };
        Ok(err)
    }
}
