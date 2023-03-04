use std::time::Duration;


pub enum BTimeout { Normal, Refilling, }
impl BTimeout {
    pub fn value(self) -> Duration {
        let secs = match self {
            Self::Normal => 1,
            Self::Refilling => 5,
        };
        Duration::from_secs(secs)
    }
}

pub struct ServerSecret {
    pub s: u32, 
    pub c: u32
}
impl ServerSecret {
    pub fn secrets() -> Vec<ServerSecret> {
        vec![
            ServerSecret {s: 23019, c: 32037},
            ServerSecret {s: 32037, c: 29295},
            ServerSecret {s: 18789, c: 13603},
            ServerSecret {s: 16443, c: 29533},
            ServerSecret {s: 18189, c: 21952},
        ]
    }
}

