
use crate::constants::{BTimeout, ServerSecret};
use crate::errors::BError;
use crate::messages::{ServerMessage, ClientMessage};
use crate::path::PathState;

pub enum PRes {
    SendMessage(ServerMessage),
    SendMessages(Vec<ServerMessage>),
    UpdateTimeout(BTimeout),
    Finish(String, ServerMessage),
}

pub enum BState {
    LoginUsername,
    LoginKey { username: String },
    LoginValidation { expected_hash: u32 },
    FindPath(PathState),
    Extract,
    Recharging(Box<BState>),
}

impl BState {

    pub fn initial() -> BState {
        BState::LoginUsername
    }

    pub fn expected_mess_lenth(&self) -> usize {
        match self {
            Self::LoginUsername => 20,
            Self::LoginKey {..} => 5,
            Self::LoginValidation {..} => 7,
            Self::FindPath(_) => 12,
            Self::Recharging(_) => 12,
            Self::Extract => 100,
        }
    }

    pub fn handle_message(self, message: ClientMessage) -> Result<(BState, PRes), BError> {
        
        match message.0.as_str() {
            "RECHARGING" => {
                if let Self::Recharging(_) = self {
                    return Err(BError::ChargingInCharging);
                }
                return Ok((BState::Recharging(Box::new(self)), PRes::UpdateTimeout(BTimeout::Refilling)));
            },
            "FULL POWER" => {
                if let Self::Recharging(next_state) = self {
                    return Ok((*next_state, PRes::UpdateTimeout(BTimeout::Normal)));
                }
                return Err(BError::ChargingFullInvalidState);
            },
            _ => {}
        }

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

                let next_state = Self::FindPath(PathState::FindingPosition);
                let message = PRes::SendMessages(
                    vec![ServerMessage::NoProblemo, ServerMessage::Left]
                    );

                Ok((next_state, message))
            }
            Self::FindPath(state) => state.handle_message(message),
            Self::Extract => Ok((self, PRes::Finish(message.0, ServerMessage::Logout))),
            Self::Recharging(_) => Err(BError::MessageWhileCharging),
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
        .map_err(|e| BError::FailedToParseNumber(Some(e)))
}

fn parse_confirmation(str: &str) -> Result<i32, BError> {
    str.parse::<i32>()
        .map_err(|e| BError::FailedToParseNumber(Some(e)))
}

