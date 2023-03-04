
mod constants;
mod client_messages;
mod errors;
mod state_machine;
mod server_messages;

use std::{net::TcpStream, io::{BufReader, BufRead, Read, Write}, error::Error, str::Chars, time::Duration};
use constants::BTimeout;
use errors::BError;
use client_messages::ClientMessage;
use server_messages::ServerMessage;
use state_machine::BState;

use crate::client_messages::ClientMessageType;

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

pub fn handle_server(mut stream: TcpStream) {
    stream.set_read_timeout(Some(BTimeout::Normal.value())).unwrap();
    let mut state = BState::LoginUsername;

    loop {
        let res = read_message(&mut stream)
            .and_then(|mess| state.handle_message(mess));

        match res {
            Ok((new_state, action)) => {
                state = new_state;
                match action {
                    state_machine::PRes::SendMessage(message) => 
                        server_send_message(&mut stream, message),

                    state_machine::PRes::UpdateTimeout(timeout) => 
                        stream.set_read_timeout(Some(timeout.value())).unwrap(),

                    state_machine::PRes::Finish => {
                        stream.shutdown(std::net::Shutdown::Both).unwrap();
                        return;
                    }
                }
            }
            Err(e) => {
                server_send_error(&mut stream, e);
                return;
            }
        }
    }
}

fn read_message(buffer: &mut TcpStream) -> Result<ClientMessage, BError> {
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
                    return ClientMessageType::by_name(header.clone())
                        .and_then(|mem_type| mem_type.process(vec![]));
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
                    return mes_type.process(message);
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

fn server_send_message(stream: &mut TcpStream, message: ServerMessage) {
    let payload = message.to_payload();
    stream.write_all(&payload).unwrap();
}

fn server_send_error(stream: &mut TcpStream, e : BError) {

    println!("Error: {:?}", e);

    let to_send = e.server_response();
    server_send_message(stream, to_send);

    stream.shutdown(std::net::Shutdown::Both).unwrap();
}

