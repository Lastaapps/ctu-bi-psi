use crate::{messages::{ClientMessage, ServerMessage}, state_machine::{BState, PRes}, errors::BError};

#[derive(PartialEq)]
enum Orient {
    NORTH, SOUTH, EAST, WEST,
}

enum PathMessage {
    Position(i32, i32),
    Recharging,
    FullPower,
}

pub enum PathState {
    Recharging(Box<PathState>),

    FindingPosition,
    FindingOrientation((i32, i32)),
    FindPath((i32, i32), Orient),
    FollowAxis((i32, i32), Orient),

    DoLeft(Box<PathState>),
    DoRight(Box<PathState>),
    DoMove(Box<PathState>),
}

impl PathState {
    pub fn handle_message(self, message: ClientMessage) -> Result<(BState, PRes), BError> {
        let message = parse_message(&message.0)?;
        match message {
            PathMessage::Recharging => {
                match self {
                    Self::Recharging(_) =>
                        Err(BError::ChargingInCharging),
                    _ => Ok((
                            BState::FindPath(Self::Recharging(Box::new(self))),
                            PRes::UpdateTimeout(crate::constants::BTimeout::Refilling))
                           ),
                }
            }
            PathMessage::FullPower => {
                match self {
                    Self::Recharging(next_state) =>
                        Ok((BState::FindPath(*next_state), PRes::UpdateTimeout(crate::constants::BTimeout::Normal))),
                    _ => Err(BError::ChargingFullInvalidState),
                }
            }
            PathMessage::Position(x, y) => {
                match self {
                    Self::Recharging(_) => Err(BError::MessageWhileCharging),

                    Self::FindingPosition => {
                        
                        let next_state = Self::FindingOrientation((x, y));
                        let message = ServerMessage::Move;

                        Ok((wp(next_state), wm(message)))
                    },

                    Self::FindingOrientation((my_x, my_y)) => {
                        if my_x == x && my_y == y {
                            let next_state = Self::FindingPosition;
                            let message = ServerMessage::Right;

                            Ok((wp(next_state), wm(message)))
                        } else {
                            let orient = match x - my_x {
                                1 => Orient::EAST,
                                -1 => Orient::WEST,
                                0 => if y - my_y == 1 {
                                    Orient::NORTH
                                } else {
                                    Orient::SOUTH
                                },
                                _ => panic!("Wtf just happened"),
                            };

                            match (x, y) {
                                (0, 0) => {
                                    let next_state = BState::Extract;
                                    let message = ServerMessage::PickUp;
                                    Ok((next_state, wm(message)))
                                },
                                (0, _) | (_, 0) => {
                                    let next_state = Self::FollowAxis((x, y), orient);
                                    let message = ServerMessage::Left;
                                    Ok((wp(next_state), wm(message)))
                                },
                                _ => {
                                    let is_valid =
                                        (x > 0 && orient == Orient::SOUTH)
                                        || (x < 0 && orient == Orient::NORTH)
                                        || (y > 0 && orient == Orient::WEST)
                                        || (y < 0 && orient == Orient::EAST);

                                    if is_valid {
                                        let next_state = Self::FindPath((x, y), orient);
                                        let message = ServerMessage::Move;
                                        Ok((wp(next_state), wm(message)))
                                    } else{
                                        let next_state = Self::FindPath((x, y), orient);
                                        let message = ServerMessage::Left;
                                        let wrapper = Self::DoLeft(Box::new(Self::DoMove(Box::new(next_state))));
                                        Ok((wp(wrapper), wm(message)))
                                    }
                                }
                            }
                        }
                    },
                    _ => panic!()
                }
            }
        }
    }
}

fn wp(state: PathState) -> BState {
    BState::FindPath(state)
}
fn wm(msg: ServerMessage) -> PRes {
    PRes::SendMessage(msg)
}

fn parse_message(str: &str) -> Result<PathMessage, BError> {
    match str {
        "RECHARGING" => Ok(PathMessage::Recharging),
        "FULL POWER" => Ok(PathMessage::FullPower),
        _ => {
            if &str[..3] == "OK " {
                parse_xy(&str[3..])
                    .map(|(x, y)| PathMessage::Position(x, y))
            } else {
                Err(BError::UnexpectedResponse(str.to_string()))
            }
        }
    }
}

fn parse_xy(str: &str) -> Result<(i32, i32), BError> {
    let (x_str, y_str) = str.split_once(' ').ok_or(BError::FailedToSplit)?;
    let x = x_str.parse().map_err(|e| BError::FailedToParseNumber(e))?;
    let y = y_str.parse().map_err(|e| BError::FailedToParseNumber(e))?;

    Ok((x, y))
}

