use std::num::ParseIntError;

use crate::{client_messages::ClientMessage, server_messages::ServerMessage};

#[derive(Debug)]
pub enum BError {
    Io(std::io::Error),
    UnknownMessage(String),
    MessageToLong(String, usize),
    FailedToParseNumber(ParseIntError),
    FailedToSplit,
    UnexpectedResponse(ClientMessage),
    InvalidKeyIndex(i16),
    HashMismatch{expected: u32, actual: u32},
}

impl BError {
    pub fn server_response(&self) -> ServerMessage {
        match self {
            Self::Io(_) => ServerMessage::LogicError,
            Self::UnknownMessage(String) => ServerMessage::SyntaxError,
            Self::MessageToLong(String, usize) => ServerMessage::SyntaxError,
            Self::FailedToParseNumber(ParseIntError) => ServerMessage::SyntaxError,
            Self::FailedToSplit => ServerMessage::SyntaxError,
            Self::UnexpectedResponse(ClientMessage) => ServerMessage::LogicError,
            Self::InvalidKeyIndex(i16) => ServerMessage::KeyOutOfRangeError,
            Self::HashMismatch{..} => ServerMessage::LoginFailed,
        }
    }
}

