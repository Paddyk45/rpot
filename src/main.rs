mod conventions;
mod generator;
mod model;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

use crate::model::*;

fn main() {
    let bind_addr = std::env::var("RPOT_BIND_ADDR").unwrap_or("127.0.0.1".to_string());
    let bind_port = std::env::var("RPOT_BIND_PORT").unwrap_or("25575".to_string());
    let listener =
        TcpListener::bind((bind_addr.clone(), bind_port.parse::<u16>().unwrap())).unwrap();

    println!("Listening on {}:{}", bind_addr, bind_port);
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
                    packet.packet_type,
                    packet.payload.unwrap_or("empty".to_string())
                );
                match packet.packet_type {
                    PacketType::Login => {
                        let response_packet = Packet::gen_auth_success(packet.request_id);
                        stream.write(&response_packet.to_u8_arr()).unwrap();
                    }

                    PacketType::RunCommand => {
                        let response_packet =
                            Packet::gen_response(packet.request_id, "".to_string());
                        stream.write(&response_packet.to_u8_arr()).unwrap();
                    }

                    _ => println!("Client sent invalid packet type"),
                }
            }
            Err(err) => panic!("{}", err),
        }
    }
}
