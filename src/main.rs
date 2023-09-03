mod model;
mod conventions;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

use crate::model::*;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:25575").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    println!("Connection to {} opened", stream.peer_addr().unwrap());
                    handle_client(stream);
                });
            }
            Err(_) => {
                println!("Error");
            }
        }
    }
}

fn handle_client(mut stream: TcpStream) {
    loop {
        let mut read = [0; 1024];
        match stream.read(&mut read) {
            Ok(n) => {
                if n == 0 {
                    // connection was closed
                    println!("Connection to {} closed", stream.peer_addr().unwrap());
                    break;
                }
                let packet: Packet = Packet::from_u8_arr(&read).unwrap();
                println!(
                    "Packet from {}:\n Length: {}\n Request ID: {}\n Request Type: {:?}\n Payload: {}",
                    stream.peer_addr().unwrap(),
                    packet.length.unwrap(),
                    packet.request_id,
                    packet.request_type,
                    packet.payload
                );
                let response_packet = Packet {
                    length: None,
                    request_id: packet.request_id,
                    request_type: PacketType::AuthSuccess,
                    payload: String::new()
                };
                stream.write(&response_packet.to_u8_arr()).unwrap();
            }
            Err(err) => panic!("{}", err),
        }
    }
}
