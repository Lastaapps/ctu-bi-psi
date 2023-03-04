use std::num::ParseIntError;

use crate::messages::{ClientMessage, ServerMessage};

#[derive(Debug)]
pub enum BError {
    Io(std::io::Error),
    UnknownMessage(String),
    MessageToLong(String, usize),
    FailedToParseNumber(ParseIntError),
    FailedToSplit,
    UnexpectedResponse(ClientMessage),
    InvalidKeyIndex(i32),
    HashMismatch{expected: u32, actual: u32},
    ConnectionClosed,
}

impl BError {
    pub fn server_response(&self) -> ServerMessage {
        match self {
            Self::Io(_) => ServerMessage::LogicError,
            Self::UnknownMessage(String) => ServerMessage::SyntaxError,
            Self::MessageToLong(String, usize) => ServerMessage::SyntaxError,
            Self::FailedToParseNumber(ParseIntError) => ServerMessage::SyntaxError,
            Self::FailedToSplit => ServerMessage::SyntaxError,
            Self::UnexpectedResponse(_) => ServerMessage::LogicError,
            Self::InvalidKeyIndex(_) => ServerMessage::KeyOutOfRangeError,
            Self::HashMismatch{..} => ServerMessage::LoginFailed,
            Self::ConnectionClosed => ServerMessage::LoginFailed,

        }
    }
}

