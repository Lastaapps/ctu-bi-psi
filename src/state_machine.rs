
use crate::constants::{BTimeout, ServerSecret};
use crate::errors::BError;
use crate::messages::{ServerMessage, ClientMessage};

enum Orientation {
    NORTH, SOUTH, EAST, WEST,
}

struct MapInfo {
    position: (i32, i32),
    orientation: Orientation,
}

pub enum PRes {
    SendMessage(ServerMessage),
    UpdateTimeout(BTimeout),
    Finish(String),
}

pub enum BState {
    LoginUsername,
    LoginKey { username: String },
    LoginValidation { expected_hash: u32 },
    FindDirection
}

impl BState {

    pub fn expected_mess_lenth(&self) -> usize {
        match self {
            Self::LoginUsername => 20,
            Self::LoginKey {..} => 5,
            Self::LoginValidation {..} => 7,
            Self::FindDirection => 12,
            // other movement 12
            // client message 100
        }
    }

    pub fn handle_message(self, message: ClientMessage) -> Result<(BState, PRes), BError> {
        match self {
            Self::LoginUsername => {

                println!("x Mach: Processing username");
                println!("x Mach: Requesting key index");

                let username = message.0;
                let next_state = Self::LoginKey { username };
                let message = PRes::SendMessage(ServerMessage::KeyRequest);

                Ok((next_state, message))
            }
            Self::LoginKey { username } => {

                println!("x Mach: Processing key");
                println!("x Mach: Sending hash");

                let key = parse_key_id(&message.0)?;

                let secrets = ServerSecret::secrets();
                let index = usize::try_from(key)
                    .map_err(|_| BError::InvalidKeyIndex(key))?;
                let secret = secrets.get(index)
                    .ok_or(BError::InvalidKeyIndex(key))?;

                login_hash("Mnau!", &ServerSecret{s:54621, c:45328});
                let hash = login_hash(&username, secret);

                let next_state = Self::LoginValidation { expected_hash: hash.1 };
                let message = PRes::SendMessage(ServerMessage::Confirm(hash.0));

                Ok((next_state, message))
            }

            Self::LoginValidation { expected_hash } => {

                println!("x Mach: Validating hash");
                println!("x Mach: Sending ok");

                let client_hash = parse_confirmation(&message.0)?;

                if expected_hash != client_hash.try_into().unwrap_or(100_000u32) {
                    return Err(BError::HashMismatch {
                        expected: expected_hash, actual: client_hash.try_into().unwrap()
                    })
                }

                let next_state = Self::FindDirection;
                let message = PRes::SendMessage(ServerMessage::NoProblemo);

                Ok((next_state, message))
            }
            _ => panic!(),
        }
    }
}

fn login_hash(username: &str, secret: &ServerSecret) -> (u32, u32){
    let modulo = 65536u32;
    let sum = username.bytes().map(Into::<u32>::into).fold(0u32, |acu, x| acu + x);
    let core = (sum * 1000) % modulo;
    let ServerSecret { c, s } = secret;
    ((s + core) % modulo, (c + core) % modulo)
}

fn parse_key_id(str: &str) -> Result<i32, BError>{ 
    str.parse::<i32>()
        .map_err(|e| BError::FailedToParseNumber(e))
}

fn parse_confirmation(str: &str) -> Result<i32, BError> {
    str.parse::<i32>()
        .map_err(|e| BError::FailedToParseNumber(e))
}

fn parse_xy(str: &str) -> Result<(i32, i32), BError> {
    let (x_str, y_str) = str.split_once(' ').ok_or(BError::FailedToSplit)?;
    let x = x_str.parse().map_err(|e| BError::FailedToParseNumber(e))?;
    let y = y_str.parse().map_err(|e| BError::FailedToParseNumber(e))?;

    Ok((x, y))
}

