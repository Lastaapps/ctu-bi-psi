use std::num::ParseIntError;

use crate::messages::ServerMessage;

#[derive(Debug)]
pub enum BError {
    Io(std::io::Error),
    ConnectionClosed,

    MessageToLong(String, usize),
    FailedToParseNumber(ParseIntError),
    FailedToSplit,

    UnexpectedResponse(String),
    InvalidKeyIndex(i32),
    HashMismatch{expected: u32, actual: u32},

    MessageWhileCharging,
    ChargingInCharging,
    ChargingFullInvalidState,
}

impl BError {
    pub fn should_send(&self) -> bool {
        match self {
            Self::Io(_) | Self::ConnectionClosed => false,
            _ => true,
        }
    }

    pub fn server_response(&self) -> ServerMessage {
        match self {
            Self::Io(_) => ServerMessage::Empty,
            Self::ConnectionClosed => ServerMessage::Empty,

            Self::MessageToLong(_, _) => ServerMessage::SyntaxError,
            Self::FailedToParseNumber(_) => ServerMessage::SyntaxError,
            Self::FailedToSplit => ServerMessage::SyntaxError,

            Self::UnexpectedResponse(_) => ServerMessage::LogicError,
            Self::InvalidKeyIndex(_) => ServerMessage::KeyOutOfRangeError,
            Self::HashMismatch {..} => ServerMessage::LoginFailed,

            Self::MessageWhileCharging => ServerMessage::LoginFailed,
            Self::ChargingInCharging => ServerMessage::LoginFailed,
            Self::ChargingFullInvalidState => ServerMessage::LoginFailed,
        }
    }
}

