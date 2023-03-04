
use crate::client_messages::ClientMessage;
use crate::constants::{BTimeout, ServerSecret};
use crate::errors::BError;
use crate::server_messages::ServerMessage;

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
    Finish,
}

pub enum BState {
    LoginUsername,
    LoginKey { username: String },
    LoginValidation { expected_hash: u32 },
    FindDirection
}

impl BState {
    pub fn handle_message(self, message: ClientMessage) -> Result<(BState, PRes), BError> {
        match self {
            Self::LoginUsername => {

                if let ClientMessage::Username(username) = message {

                    let next_state = Self::LoginKey { username };
                    let message = PRes::SendMessage(ServerMessage::KeyRequest);

                    Ok((next_state, message))

                } else {
                    Err(BError::UnexpectedResponse(message))
                }
            }
            Self::LoginKey { username } => {

                if let ClientMessage::KeyID(key) = message {

                    let secrets = ServerSecret::secrets();
                    let index = usize::try_from(key)
                        .map_err(|_| BError::InvalidKeyIndex(key))?;
                     let secret = secrets.get(index)
                        .ok_or(BError::InvalidKeyIndex(key))?;

                    let hash = login_hash(&username, secret);

                    let next_state = Self::LoginValidation { expected_hash: hash.1 };
                    let message = PRes::SendMessage(ServerMessage::Confirm(hash.0));

                    Ok((next_state, message))
                } else {
                    Err(BError::UnexpectedResponse(message))
                }
            }

            Self::LoginValidation { expected_hash } => {

                if let ClientMessage::Confirmation(client_hash) = message {

                    if expected_hash != client_hash.try_into().unwrap_or(100_000u32) {
                        return Err(BError::HashMismatch {
                            expected: expected_hash, actual: client_hash.try_into().unwrap()
                        })
                    }

                    let next_state = Self::FindDirection;
                    let message = PRes::SendMessage(ServerMessage::NoProblemo);

                    Ok((next_state, message))
                } else {
                    Err(BError::UnexpectedResponse(message))
                }
            }
            _ => panic!(),
        }
    }
}

fn login_hash(username: &str, secret: &ServerSecret) -> (u32, u32){
    let sum = username.bytes().map(Into::<u32>::into).fold(1u32, |acu, x| acu + x);
    let core = sum + 1000 % 65536;
    let ServerSecret { s, c } = secret;
    ((s + core) % 65536, (c + core) % 65536)
}

