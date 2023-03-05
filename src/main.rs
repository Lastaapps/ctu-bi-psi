
mod constants;
mod errors;
mod state_machine;
mod messages;
mod path;

use constants::BTimeout;
use errors::BError;
use messages::{ClientMessage, ServerMessage};
use state_machine::BState;

use std::{net::{TcpListener, TcpStream}, thread, io::{Read, Write}};

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
            println!("Connection closed!");
        });
    }
}

pub fn handle_server(mut stream: TcpStream) {
    stream.set_write_timeout(Some(BTimeout::Normal.value())).unwrap();
    stream.set_read_timeout(Some(BTimeout::Normal.value())).unwrap();
    stream.set_nodelay(true).unwrap();

    let mut state = BState::LoginUsername;

    loop {
        let max_len = state.expected_mess_lenth();
        let res = read_message(&mut stream, max_len)
            .and_then(|mess| state.handle_message(mess));

        match res {
            Ok((new_state, action)) => {
                state = new_state;
                match action {
                    state_machine::PRes::SendMessage(message) => 
                        server_send_message(&mut stream, message),

                    state_machine::PRes::SendMessages(messages) =>
                        for message in messages {
                            server_send_message(&mut stream, message)
                        },

                    state_machine::PRes::UpdateTimeout(timeout) =>  {
                        stream.set_write_timeout(Some(timeout.value())).unwrap();
                        stream.set_read_timeout(Some(timeout.value())).unwrap();
                    },

                    state_machine::PRes::Finish(message) => {
                        println!("The message was \"{}\"", message);
                        server_send_message(&mut stream, ServerMessage::Logout);
                        server_shutdown(&stream);
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

fn read_message(stream: &mut TcpStream, max_len: usize) -> Result<ClientMessage, BError> {
    let mut message = Vec::<u8>::new();

    loop {
        let len = message.len();
        if max_len - 2 < len {
            if !(message[max_len - 2] == 7u8 && max_len - 1 == len) {
                return Err(BError::MessageToLong(String::from_utf8(message).unwrap(), len));
            }
        }

        let mut bytes = [0; 1];
        let bytes_num = unwrap_io(stream.read(&mut bytes))?;
        if bytes_num == 0 {
            return Err(BError::ConnectionClosed);
        }

        let byte = bytes[0];
        if byte == 8u8 { // \b
            if let Some(last) = message.last() {
                if last == &7u8 {
                    message.pop();
                    let str = String::from_utf8(message).unwrap();
                    println!("> Read: {}", str);
                    return Ok(ClientMessage(str));
                }
            }
        }

        // println!("? Push: {} - {}", byte, String::from_utf8(bytes.to_vec()).unwrap());
        message.push(byte)
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

    let str = String::from_utf8(payload.clone()).unwrap();
    println!("# Send: {}", str);

    stream.write_all(&payload).unwrap();
}

fn server_send_error(stream: &mut TcpStream, error : BError) {

    println!("Error: {:?}", error);

    if error.should_send() {
        let to_send = error.server_response();
        server_send_message(stream, to_send);
    }

    server_shutdown(stream);
}

fn server_shutdown(stream: &TcpStream) {
    println!("Stopping a stream");
    match stream.shutdown(std::net::Shutdown::Both) {
        Ok(_) => {}
        Err(e) => println!("Server didn't shudown as expected: {}", e),
    }
}

