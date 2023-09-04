mod conventions;
mod generator;
mod model;
mod webhook;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

use crate::model::*;
use crate::webhook::webhook_handler;

fn main() {
    let bind_addr = std::env::var("RPOT_BIND_ADDR").unwrap_or("0.0.0.0".to_string());
    let bind_port = std::env::var("RPOT_BIND_PORT").unwrap_or("25575".to_string());
    let webhook_url = match std::env::var("RPOT_WEBHOOK_URL") {
        Ok(val) => Some(val),
        Err(_) => None,
    };
    let listener =
        TcpListener::bind((bind_addr.clone(), bind_port.parse::<u16>().unwrap())).unwrap();

    println!("Listening on {}:{}", bind_addr, bind_port);
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let wh_url = webhook_url.clone();
                thread::spawn(move || {
                    println!("Connection to {} opened", stream.peer_addr().unwrap());

                    if wh_url.clone().is_some() {
                        println!("a");
                        webhook_handler(
                            wh_url.clone().unwrap(),
                            stream
                                .peer_addr()
                                .unwrap_or("0.0.0.0:1".parse().unwrap())
                                .to_string(),
                            webhook::EventType::ClientConnect,
                            Some(String::new()),
                        ).unwrap();
                    }

                    handle_client(stream, wh_url.clone());
                });
            }
            Err(_) => {
                println!("Error");
            }
        }
    }
}

fn handle_client(mut stream: TcpStream, webhook_url: Option<String>) {
    loop {
        let mut read = [0; 1024];
        match stream.read(&mut read) {
            Ok(n) => {
                if n == 0 {
                    // connection was closed
                    println!("Connection to {} closed", stream.peer_addr().unwrap());
                    webhook_handler(
                        webhook_url.clone().unwrap(),
                        stream
                            .peer_addr()
                            .unwrap_or("0.0.0.0:1".parse().unwrap())
                            .to_string(),
                        webhook::EventType::ClientDisconnect,
                        None,
                    ).unwrap();
                    break;
                }
                let packet: Packet = Packet::from_u8_arr(&read).unwrap();
                println!(
                    "Packet from {}:\n Length: {}\n Request ID: {}\n Request Type: {:?}\n Payload: {}",
                    stream.peer_addr().unwrap(),
                    packet.length.unwrap(),
                    packet.request_id,
                    packet.packet_type,
                    strip_ansi_escapes::strip_str(packet.payload.clone().unwrap_or("empty".to_string()))
                );
                match packet.packet_type {
                    PacketType::Login => {
                        webhook_handler(
                            webhook_url.clone().unwrap(),
                            stream
                                .peer_addr()
                                .unwrap_or("0.0.0.0:1".parse().unwrap())
                                .to_string(),
                            webhook::EventType::Auth,
                            packet.payload,
                        ).unwrap();
                        
                        let response_packet = Packet::gen_auth_success(packet.request_id);
                        stream.write(&response_packet.to_u8_arr()).unwrap();
                    }

                    PacketType::RunCommand => {
                        let mut command = packet.payload.clone().unwrap_or("".to_string());
                        webhook_handler(
                            webhook_url.clone().unwrap(),
                            stream
                                .peer_addr()
                                .unwrap_or("0.0.0.0:1".parse().unwrap())
                                .to_string(),
                            webhook::EventType::RunCommand,
                            packet.payload,
                        ).unwrap();
                        let command_response =
                            match command.as_str().split_whitespace().next().unwrap_or("") {
                                "seed" => "Seed: [69420]",
                                "say" => "",
                                "" => "",
                                _ => "Unknown command. Type \"/help\" for help.",
                            };

                        stream
                            .write(
                                &Packet::gen_response(
                                    packet.request_id.clone(),
                                    command_response.to_string(),
                                )
                                .to_u8_arr(),
                            )
                            .expect("Failed to write to stream");
                    }

                    _ => println!("Client sent invalid packet type"),
                }
            }
            Err(err) => panic!("{}", err),
        }
    }
}
