use std::{net::TcpStream, io::BufReader};

use crate::messages::ClientMessage;

pub struct Server<'a> {
    stream: TcpStream<'a>,
    buffer: BufReader<&'a mut TcpStream>,
}

impl Server {
    fn new(stream: TcpStream) -> Server {
        Server{ stream: stream, buffer: buf_reader }
    }

    fn read_message() -> Result<ClientMessage, String> {
        panic!();
    }
}

