use crate::{messages::{ClientMessage, ServerMessage}, state_machine::{BState, PRes}, errors::BError};

#[derive(Debug)]
#[derive(PartialEq)]
pub enum Orient {
    NORTH, SOUTH, EAST, WEST,
}

impl Orient {
    fn left(&self) -> Orient {
        match self {
            Self::NORTH => Self::WEST,
            Self::SOUTH =>Self::EAST,
            Self::EAST =>Self::NORTH,
            Self::WEST =>Self::SOUTH,
        }
    }
    fn right(&self) -> Orient {
        self.left().left().left()
    }
    fn is_valid_for(&self, x: i32, y: i32) -> bool {
        false
            || (x > 0 && self == &Orient::WEST)
            || (x < 0 && self == &Orient::EAST)
            || (y > 0 && self == &Orient::SOUTH)
            || (y < 0 && self == &Orient::NORTH)
    }
    fn move_in(&self, (x, y): (i32, i32)) -> (i32, i32) {
        match self {
            Self::NORTH => (x + 0, y + 1),
            Self::SOUTH => (x + 0, y - 1),
            Self::EAST  => (x + 1, y + 0),
            Self::WEST  => (x - 1, y + 0),
        }
    }
}

#[derive(Debug)]
pub enum PathState {
    FindingPosition,
    FindingOrientation((i32, i32)),
    FindPath((i32, i32), Orient),
    SetupAxis((i32, i32), Orient),
    FollowAxis((i32, i32), Orient),

    DoLeft(Box<PathState>),
    DoRight(Box<PathState>),
    DoMove(Box<PathState>),
}

impl PathState {
    pub fn handle_message(self, message: ClientMessage) -> Result<(BState, PRes), BError> {
        let (x, y) = parse_message(&message.0)?;

        println!("+ Path: State {:?}", self);

        match self {
            Self::FindingPosition => {

                println!("+ Path: Processing first rotation");
                println!("+ Path: Trying the first move");

                let next_state = Self::FindingOrientation((x, y));
                let message = ServerMessage::Move;

                Ok((wp(next_state), wm(message)))
            },

            Self::FindingOrientation((px, py)) => {

                if px == x && py == y {

                    println!("+ Path: First move blocked");
                    println!("+ Path: Trying again");

                    let next_state = Self::FindingPosition;
                    let message = ServerMessage::Right;

                    Ok((wp(next_state), wm(message)))
                } else {

                    println!("+ Path: First move succeed");

                    let orient = match x - px {
                        1 => Orient::EAST,
                        -1 => Orient::WEST,
                        0 => if y - py == 1 {
                            Orient::NORTH
                        } else {
                            Orient::SOUTH
                        },
                        _ => panic!("Wtf just happened"),
                    };
                    println!("+ Path: Orientation is {:?}", orient);

                    match (x, y) {
                        (0, 0) => {
                            println!("+ Path: Ready to extract");

                            let next_state = BState::Extract;
                            let message = ServerMessage::PickUp;
                            Ok((next_state, wm(message)))
                        },
                        (0, _) | (_, 0) => {
                            println!("+ Path: Ready to follow axis");

                            let next_state = Self::SetupAxis((x, y), orient.left());
                            let message = ServerMessage::Left;
                            Ok((wp(next_state), wm(message)))
                        },
                        _ => {
                            let is_valid = orient.is_valid_for(x, y);

                            if is_valid {
                                println!("+ Path: Orientation is valid");

                                let next_state = Self::FindPath((x, y), orient);
                                let message = ServerMessage::Move;
                                Ok((wp(next_state), wm(message)))
                            } else{
                                println!("+ Path: Rotating by π");

                                let next_state = Self::FindPath((x, y), orient.left().left());
                                let message = ServerMessage::Left;
                                let wrapper = Self::DoLeft(Box::new(Self::DoMove(Box::new(next_state))));
                                Ok((wp(wrapper), wm(message)))
                            }
                        }
                    }
                }
            },

            Self::FindPath((px, py), orient) => {
                if x == 0 || y == 0 {
                    println!("+ Path: Moving to axis");

                    let next_state = Self::SetupAxis((x, y), orient.left());
                    let message = ServerMessage::Left;
                    Ok((wp(next_state), wm(message)))
                } else if px == x && py == y {
                    if orient.left().is_valid_for(x, y) {

                        println!("+ Path: Obsticke hit! Rotating left");

                        let next_state = Self::DoMove(Box::new(Self::FindPath((x, y), orient.left())));
                        let message = ServerMessage::Left;
                        Ok((wp(next_state), wm(message)))
                    } else {

                        println!("+ Path: Obsticke hit! Rotating right");

                        let next_state = Self::DoMove(Box::new(Self::FindPath((x, y), orient.right())));
                        let message = ServerMessage::Right;
                        Ok((wp(next_state), wm(message)))
                    }
                } else {

                    println!("+ Path: No problemo, let's move again");

                    let next_state = Self::FindPath((x, y), orient);
                    let message = ServerMessage::Move;
                    Ok((wp(next_state), wm(message)))
                }
            }

            Self::SetupAxis((_, _), orient) => {
                if x == 0 && y == 0 {

                    println!("+ Path: Extracting");

                    let next_state = BState::Extract;
                    let message = ServerMessage::PickUp;
                    Ok((next_state, wm(message)))
                } else {
                    if orient.is_valid_for(x, y) {
                        println!("+ Path: Got the right direction, moving");

                        let next_state = Self::FollowAxis((x, y), orient);
                        let message = ServerMessage::Move;
                        Ok((wp(next_state), wm(message)))
                    } else {

                        println!("+ Path: Wrong direction, moving");

                        let next_state = Self::SetupAxis((x, y), orient.left());
                        let message = ServerMessage::Left;
                        Ok((wp(next_state), wm(message)))
                    }
                }
            }

            Self::FollowAxis((px, py), orient) => {
                if x == 0 && y == 0 {
                    println!("+ Path: Extracting");

                    let next_state = BState::Extract;
                    let message = ServerMessage::PickUp;
                    Ok((next_state, wm(message)))
                } else if px == x && py == y {
                    println!("+ Path: Obsticle hit, going around");

                    let coord = orient.move_in((x, y));
                    let next_state = Self::FollowAxis(coord, orient);
                    let wrapped = do_m(do_r(do_m(do_m(do_r(do_m(do_l(next_state)))))));
                    let message = ServerMessage::Left;
                    Ok((wp(wrapped), wm(message)))
                } else {
                    println!("+ Path: No problemo, lets move");

                    let next_state = Self::FollowAxis((x, y), orient);
                    let message = ServerMessage::Move;
                    Ok((wp(next_state), wm(message)))
                }
            }

            Self::DoLeft(next_state) => {
                let message = ServerMessage::Left;
                Ok((wp(*next_state), wm(message)))
            }
            Self::DoRight(next_state) => {
                let message = ServerMessage::Right;
                Ok((wp(*next_state), wm(message)))
            }
            Self::DoMove(next_state) => {
                let message = ServerMessage::Move;
                Ok((wp(*next_state), wm(message)))
            }
        }
    }
}

fn do_m(state: PathState) -> PathState {
    PathState::DoMove(Box::new(state))
}

fn do_l(state: PathState) -> PathState {
    PathState::DoLeft(Box::new(state))
}

fn do_r(state: PathState) -> PathState {
    PathState::DoRight(Box::new(state))
}

fn wp(state: PathState) -> BState {
    BState::FindPath(state)
}
fn wm(msg: ServerMessage) -> PRes {
    PRes::SendMessage(msg)
}

fn parse_message(str: &str) -> Result<(i32, i32), BError> {
    if &str[..3] == "OK " {
        parse_xy(&str[3..])
    } else {
        Err(BError::FailedToParseNumber(None))
    }
}


fn parse_xy(str: &str) -> Result<(i32, i32), BError> {
    let (x_str, y_str) = str.split_once(' ').ok_or(BError::FailedToSplit)?;
    let x = x_str.parse().map_err(|e| BError::FailedToParseNumber(Some(e)))?;
    let y = y_str.parse().map_err(|e| BError::FailedToParseNumber(Some(e)))?;

    Ok((x, y))
}

