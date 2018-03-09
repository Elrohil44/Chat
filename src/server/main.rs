use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::process::exit;

fn handle_client(mut stream: TcpStream) {
    println!("New client");
    let mut buf = [0; 1024];
    match stream.read(&mut buf) {
        Ok(size) => {
            match String::from_utf8(buf[0 .. size].to_vec()) {
                Ok(str) => println!("{}", str),
                Err(_) => {}
            }
        },
        Err(_) => {}
    }

    let s = String::from("Hello");
    let _ = stream.write(s.as_bytes());
}

static SOCKET_ADDRESS: &'static str = "127.0.0.1:12345";

fn main() {
    match TcpListener::bind(SOCKET_ADDRESS) {
        Ok(listener) => {
            for stream in listener.incoming() {
                match stream {
                    Ok(s) => handle_client(s),
                    Err(_) => println!("Connection error")
                }
            }
        },
        Err(_) => {
            println!("Couldn't bind to {}", SOCKET_ADDRESS);
            exit(1);
        }
    }

}