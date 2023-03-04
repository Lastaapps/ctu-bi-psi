
use crate::errors::BError;

#[derive(Debug)]
pub enum ClientMessage {
    Username(String),
    KeyID(i16),
    Confirmation(i16),
    NoProblemo { x: i32, y: i32, },
    Recharging,
    FullPower,
    Secret(String),
}

#[derive(Debug)]
pub enum ClientMessageType {
    Username,
    KeyID,
    Confirmation,
    NoProblemo,
    Recharging,
    FullPower,
    Secret,
}

impl ClientMessageType {
    pub fn max_len(&self) -> usize {
        let main = match self {
            ClientMessageType::Username => 20,
            ClientMessageType::KeyID => 5,
            ClientMessageType::Confirmation => 7,
            ClientMessageType::NoProblemo => 12,
            ClientMessageType::Recharging => 12,
            ClientMessageType::FullPower => 12,
            ClientMessageType::Secret => 100,
        };
        main - 2
    }

    pub fn by_name(name: Vec<u8>) -> Result<ClientMessageType, BError> {
        let str = String::from_utf8(name).unwrap();
        match str.as_str() {
            "CLIENT_USERNAME" => Ok(ClientMessageType::Username),
            "CLIENT_KEY_ID" => Ok(ClientMessageType::KeyID),
            "CLIENT_CONFIRMATION" => Ok(ClientMessageType::Confirmation),
            "CLIENT_OK" => Ok(ClientMessageType::NoProblemo),
            "CLIENT_RECHARGING" => Ok(ClientMessageType::Recharging),
            "CLIENT_FULL_POWER" => Ok(ClientMessageType::FullPower),
            "CLIENT_MESSAGE" => Ok(ClientMessageType::Secret),
            _ => Err(BError::UnknownMessage(str)),
        }
    }

    pub fn process(&self, payload: Vec<u8>) -> Result<ClientMessage, BError> {
        let str = String::from_utf8(payload).unwrap();
        let main = match self {
            ClientMessageType::Username => 
                ClientMessage::Username(str),
            ClientMessageType::KeyID => 
                ClientMessage::KeyID(Self::bparse(&str)?),
            ClientMessageType::Confirmation => 
                ClientMessage::Confirmation(Self::bparse(&str)?),
            ClientMessageType::NoProblemo => 
                Self::parse_xy(&str)
                .map(|(x,y)| ClientMessage::NoProblemo{x, y})?,
            ClientMessageType::Recharging =>
                ClientMessage::Recharging,
            ClientMessageType::FullPower =>
                ClientMessage::FullPower,
            ClientMessageType::Secret =>
                ClientMessage::Secret(str)
        };

        Ok(main)
    }

    fn bparse(str: &str) -> Result<i16, BError>{ 
        str.parse::<i16>()
            .map_err(|e| BError::FailedToParseNumber(e))
    }

    fn parse_xy(str: &str) -> Result<(i32, i32), BError> {
        let (x_str, y_str) = str.split_once(' ').ok_or(BError::FailedToSplit)?;
        let x = x_str.parse().map_err(|e| BError::FailedToParseNumber(e))?;
        let y = y_str.parse().map_err(|e| BError::FailedToParseNumber(e))?;

        Ok((x, y))
    }
}

