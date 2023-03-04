
use crate::errors::BError;

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
}
