
pub enum BError {
    Io(std::io::Error),
    UnknownMessage(String),
    MessageToLong(String, usize),
}

