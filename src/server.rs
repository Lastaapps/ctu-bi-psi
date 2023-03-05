
use crate::constants::BTimeout;
use crate::errors::BError;
use crate::messages::{ClientMessage, ServerMessage};
use crate::state_machine::BState;

use std::cmp::min;
use std::{net::TcpStream, io::{Read, Write}};

use crate::state_machine;

pub fn handle_server(mut stream: TcpStream) {
    stream.set_write_timeout(Some(BTimeout::Normal.value())).unwrap();
    stream.set_read_timeout(Some(BTimeout::Normal.value())).unwrap();
    stream.set_nodelay(true).unwrap();

    let mut state = BState::initial();

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

                    state_machine::PRes::Finish(message, response) => {
                        println!("The message was \"{}\"", message);
                        server_send_message(&mut stream, response);
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

fn check_max_len_overflow(max_len: usize, message: &Vec<u8>) -> bool {
    let len = message.len();
    (max_len - 2 < len) && !(message[max_len - 2] == 7u8 && max_len - 1 == len)
}

fn prefix_match(msg1: &Vec<u8>, msg2: &Vec<u8>) -> bool {
    let len = min(msg1.len(), msg2.len());
    for i in 0..len {
        if msg1[i] != msg2[i] {
            return false
        }
    }
    true
}

fn read_message(stream: &mut TcpStream, max_len: usize) -> Result<ClientMessage, BError> {
    let mut message = Vec::<u8>::new();
    let recharching_bytes = "RECHARGING".as_bytes().to_vec();
    let full_power_bytes = "FULL POWER".as_bytes().to_vec();

    loop {
        let is_normal_overflow = check_max_len_overflow(max_len, &message);
        let is_charging = !check_max_len_overflow(12, &message) && 
            (prefix_match(&message, &recharching_bytes)
             || prefix_match(&message, &full_power_bytes));

        if is_normal_overflow && !is_charging {
            let len = message.len();
            return Err(BError::MessageToLong(String::from_utf8(message).unwrap(), len));
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
