use std::{thread, net::TcpListener};


mod constants;
mod errors;
mod state_machine;
mod messages;
mod server;
mod path;

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
            server::handle_server(stream);
            println!("Connection closed!");
        });
    }
}

