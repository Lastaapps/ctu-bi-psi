
mod messages;
mod errors;

use std::{net::TcpStream, io::{BufReader, BufRead, Read}, error::Error, str::Chars};
use errors::BError;

use crate::messages::ClientMessageType;

use std::{net::TcpListener, thread};

fn main() {
    let host = "127.0.0.1";
    let port = 42069;
    // let addr = host.to_owned() + ":" + &port.to_string();
    let addr = format!("{}:{}", host, port);
    let listener = TcpListener::bind(addr.clone()).unwrap();

    println!("Starting Bobika!");
    println!("Listening on {addr}");

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        thread::spawn(move || {
            println!("Connection established!");
            handle_server(stream);
        });
    }
}

type Buffer<'a> = BufReader<&'a mut TcpStream>;

pub fn handle_server(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
}

fn read_message(buffer: &mut Buffer) -> Result<ClientMessageType, BError> {
    let mut header = Vec::<u8>::new();
    loop {
        let mut byte = [0; 1];
        unwrap_io(buffer.read(&mut byte))?;
        match byte[0] {
            b' ' => break,
            8u8 => { // \b
                let last = match header.last() {
                    None => panic!("The message starts with \\b"),
                    Some(last) => last.to_owned()
                };

                if last == 7u8 {
                    header.pop();
                    return ClientMessageType::by_name(header.clone());
                } else {
                    panic!("\\b alone in message name");
                }
            },
            a => header.push(a),
        }
    };

    let mes_type = ClientMessageType::by_name(header)?;
    let max_len = mes_type.max_len();
    
    let mut message = Vec::<u8>::new();
    loop {
        let len = message.len();
        if max_len < len {
            if !(message[max_len] == 7u8 && max_len + 1 == len) {
                return Err(BError::MessageToLong(String::from_utf8(message).unwrap(), len));
            }
        }

        let mut byte = [0; 1];
        unwrap_io(buffer.read(&mut byte))?;
        match byte[0] {
            8u8 => { // \b
                let last = match message.last() {
                    None => panic!("The body starts with \\b"),
                    Some(last) => last.to_owned()
                };

                if last == 7u8 {
                    message.pop();
                    return ClientMessageType::by_name(message);
                } else {
                    panic!("\\b alone in the body");
                }
            },
            a => message.push(a),
        }
    }
}

fn unwrap_io<T>(res: Result<T, std::io::Error>) -> Result<T, BError> {
    match res {
        Ok(data) => Ok(data),
        Err(e) => Err(BError::Io(e)),
    }
}

