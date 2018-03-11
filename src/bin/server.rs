use std::io::prelude::*;
use std::thread;
use std::net::{TcpListener, TcpStream, UdpSocket};
use std::collections::HashMap;
use std::sync::Mutex;
use std::process::exit;
use std::sync::Arc;
use std::net::SocketAddr;

static SOCKET_ADDRESS: &'static str = "127.0.0.1:12345";

fn main() {
    let clients = Arc::new(Mutex::new(HashMap::new()));
    let clients_cloned = clients.clone();
    let tcp_listener = thread::spawn(move || tcp_listener(clients_cloned));
    let clients_cloned = clients.clone();
    let udp_socket = thread::spawn(move || udp_socket(clients_cloned));

    let _ = tcp_listener.join();
    let _ = udp_socket.join();
}

fn handle_client(mut stream: TcpStream, clients: Arc<Mutex<HashMap<SocketAddr, TcpStream>>>) {
    println!("New client from: {}",  stream.peer_addr().unwrap());
    let mut buf = [0; 1024];
    {
        let mut clients_locked = clients.lock().unwrap();
        (*clients_locked).insert(stream.peer_addr().unwrap(), stream.try_clone().unwrap());
    }
    loop {
        match stream.read(&mut buf) {
            Ok(size) => {
                if size > 0 {
                    match String::from_utf8(buf[0..size].to_vec()) {
                        Ok(str) => {
                            let mut clients_locked = clients.lock().unwrap();
                            for (address, s) in (*clients_locked).iter_mut() {
                                println!("{}", s.peer_addr().unwrap());
                                if stream.peer_addr().unwrap() != *address {
                                   let _ = (*s).write_fmt(format_args!("{}: {}", stream.peer_addr().unwrap(), str));
                                }
                            }
                        },
                        Err(_) => println!("Error casting to string")
                    }
                } else {
                    println!("{} disconnected", stream.peer_addr().unwrap());
                    let mut clients_locked = clients.lock().unwrap();
                    (*clients_locked).remove(&stream.peer_addr().unwrap());
                    break;
                }
            },
            Err(_) => println!("Error while reading, client disconnected")
        }

    }
}

fn tcp_listener(clients: Arc<Mutex<HashMap<SocketAddr, TcpStream>>>) {
    match TcpListener::bind(SOCKET_ADDRESS) {
        Ok(listener) => {
            for stream in listener.incoming() {
                let c = clients.clone();
                match stream {
                    Ok(s) => {
                        thread::spawn(move || handle_client(s, c));
                    },
                    Err(_) => println!("Connection error")
                }
            }
        },
        Err(_) => {
            println!("Couldn't bind TCP to {}", SOCKET_ADDRESS);
            exit(1);
        }
    }
}

fn udp_socket(clients: Arc<Mutex<HashMap<SocketAddr, TcpStream>>>) {
    match UdpSocket::bind(SOCKET_ADDRESS) {
        Ok(socket) => {
            let mut buf = [0; 1024];
            loop {
                match socket.recv_from(&mut buf) {
                    Ok((amt, src)) => {
                        match String::from_utf8(buf[0 .. amt].to_vec()) {
                            Ok(str) => {
                                println!("Received UDP message: \"{}\", from {}", str, src);
                                let mut clients_locked = clients.lock().unwrap();
                                for (_, s) in &(*clients_locked) {
                                    println!("{}", s.peer_addr().unwrap());
                                    if src != s.peer_addr().unwrap() {
                                        let _ = socket.send_to(format!("{}: {}", src, str).as_bytes(), s.peer_addr().unwrap());
                                    }
                                }
                            },
                            Err(_) => println!("Error while casting to string")
                        }
                    },
                    Err(_) => println!("Error while receiving UDP message")
                }
            }
        },
        Err(_) => {
            println!("Couldn't bind UDP to {}", SOCKET_ADDRESS);
            exit(1);
        }
    }
}


