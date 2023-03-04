use std::num::ParseIntError;


pub enum BError {
    Io(std::io::Error),
    UnknownMessage(String),
    MessageToLong(String, usize),
    FailedToParseNumber(ParseIntError),
    FailedToSplit,
}

