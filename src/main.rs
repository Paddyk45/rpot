#![warn(clippy::all, clippy::nursery)]
#![allow(clippy::missing_const_for_fn, clippy::redundant_pub_crate)]

use std::net::{IpAddr, SocketAddr};
use std::process::exit;
use std::time::Duration;

use anyhow::bail;
use tokio::signal::unix::{signal, SignalKind};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    select,
};

use handlers::*;
use webhook_model::{MaybeWebhook, Webhook};

use crate::model::{EventType, Packet, PacketType};

mod de_serialize;
mod generator;
mod handlers;
mod model;
mod webhook;
mod webhook_model;
mod tests;

// This is based on https://gist.github.com/fortruce/828bcc3499eb291e7e17
#[tokio::main]
async fn main() {
    let bind_addr = std::env::var("RPOT_BIND_ADDR").unwrap_or_else(|_| "0.0.0.0".to_string());
    let bind_port = std::env::var("RPOT_BIND_PORT").unwrap_or_else(|_| "25575".to_string());
    let webhook_url = std::env::var("RPOT_WEBHOOK_URL").ok();
    let ip_addr = IpAddr::V4(bind_addr.parse().expect("Failed to parse RPOT_BIND_ADDR as IPv4"));
    let port: u16 = bind_port.parse().expect("Failed to parse RPOT_BIND_PORT as port");
    let addr = SocketAddr::new(ip_addr, port);

    tokio::spawn(async move {
        let mut sigterm = signal(SignalKind::terminate()).unwrap();
        select! {
             _ = sigterm.recv() => {
                    println!("Received SIGTERM, exiting...");
                    exit(0)
                }
        }
    });

    listener(addr, webhook_url).await;
}

async fn listener(addr: SocketAddr, webhook_url: Option<String>) {
    let listener = TcpListener::bind(addr)
        .await
        .unwrap();
    println!("Listening on {}:{}", addr.ip(), addr.port());
    while let Ok(stream) = listener.accept().await {
        let stream = stream.0;
        let webhook_url = webhook_url.clone();
        println!("Connection to {} opened", stream.peer_addr().unwrap());
        let peer_addr = stream.peer_addr().unwrap().to_string();
        let x = webhook_url.map(|url| Webhook::new(peer_addr.clone(), url));
        let mut webhook: MaybeWebhook = x.into();
        tokio::spawn(async move {
            match handle_client(stream, &mut webhook).await {
                Ok(()) => {}
                Err(err) => println!("Error handling client: {}", err),
            };
            println!("Connection to {} closed", peer_addr.clone());
            let _ = webhook
                .send_if_some(EventType::ClientDisconnect, None)
                .await
                .map_err(webhook::print_webhook_err);
        });
    }
}

async fn handle_client(mut stream: TcpStream, webhook: &mut MaybeWebhook) -> anyhow::Result<()> {
    let mut is_authenticated = false;
    let _ = webhook
        .send_if_some(EventType::ClientConnect, None)
        .await
        .map_err(webhook::print_webhook_err);

    loop {
        let mut read = [0; 1024];
        match stream.read(&mut read).await {
            Ok(n) => {
                if n == 0 {
                    // connection was closed
                    break;
                }

                let mut read = read[..n].to_vec();
                let expected_len = i32::from_le_bytes(read[0..4].try_into().unwrap()) + 4;
                if expected_len != read.len() as i32 {
                    continue;
                }
                let packet: Packet = match Packet::try_deserialize(read) {
                    Ok(p) => p,
                    Err(e) => {
                        stream.write_all(&0_i32.to_le_bytes()).await.expect("Failed to write to stream");
                        println!("Error deserializing packet: {}", e.to_string());
                        continue;
                    }
                };
                println!(
                    "Packet from {}:\n Length: {}\n Request ID: {}\n Request Type: {:?}\n Payload: {}",
                    stream.peer_addr()?,
                    packet.length.unwrap(),
                    packet.request_id,
                    packet.packet_type,
                    strip_ansi_escapes::strip_str(packet.payload.clone().unwrap_or_else(|| "empty".to_string()))
                );
                let _ = webhook
                    .send_if_some(packet.packet_type.into(), packet.payload.clone())
                    .await
                    .map_err(webhook::print_webhook_err);

                let handler: fn(Packet) -> Packet = match packet.packet_type {
                    PacketType::Login => {
                        if packet.payload != None {
                            is_authenticated = true;
                        }
                        handler_login
                    }

                    PacketType::RunCommand => {
                        match is_authenticated {
                            false => |_| Packet::gen_auth_fail(),
                            true => {
                                tokio::time::sleep(Duration::from_millis(500)).await; // Simulate delay for commands
                                handler_runcommand
                            }
                        }
                    }
                    _ => handler_invalid,
                };

                let response_packet = handler(packet).serialize();
                stream
                    .write_all(&response_packet)
                    .await
                    .expect("Failed to write to stream");
            }
            Err(err) => bail!(err),
        }
    }
    Ok(())
}
