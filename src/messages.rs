
#[derive(Debug)]
pub struct ClientMessage(pub String);


#[derive(Debug)]
pub enum ServerMessage {
    Confirm(u32),
    Move,
    Left,
    Right,
    PickUp,
    Logout,
    KeyRequest,
    NoProblemo,
    LoginFailed,
    SyntaxError,
    LogicError,
    KeyOutOfRangeError,
}

impl ServerMessage {
    pub fn to_payload(&self) -> Vec<u8> {
        let body = match self {
            Self::Confirm(key) => key.to_string(),
            Self::Move => "102 MOVE".to_string(),
            Self::Left => "103 TURN LEFT".to_string(),
            Self::Right => "104 TURN RIGHT".to_string(),
            Self::PickUp => "105 GET MESSAGE".to_string(),
            Self::Logout => "106 LOGOUT".to_string(),
            Self::KeyRequest => "107 KEY REQUEST".to_string(),
            Self::NoProblemo => "200 OK".to_string(),
            Self::LoginFailed => "300 LOGIN FAILED".to_string(),
            Self::SyntaxError => "301 SYNTAX ERROR".to_string(),
            Self::LogicError => "302 LOGIC ERROR".to_string(),
            Self::KeyOutOfRangeError => "03 KEY OUT OF RANGE".to_string(),
        };
        let mut bytes = body.as_bytes().to_vec();
        bytes.push(7); // \a
        bytes.push(8); // \b
        bytes
    }
}

