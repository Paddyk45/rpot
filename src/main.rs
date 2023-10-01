mod conversions;
mod generator;
mod handlers;
mod model;
mod webhook;
mod webhook_model;

use handlers::*;
use std::time::Duration;

use anyhow::bail;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use webhook_model::{MaybeWebhook, Webhook};

use crate::model::{EventType, Packet, PacketType};

// This is based on https://gist.github.com/fortruce/828bcc3499eb291e7e17
#[tokio::main]
async fn main() {
    let bind_addr = std::env::var("RPOT_BIND_ADDR").unwrap_or("0.0.0.0".to_string());
    let bind_port = std::env::var("RPOT_BIND_PORT").unwrap_or("25575".to_string());
    let webhook_url = std::env::var("RPOT_WEBHOOK_URL").ok();
    let listener = TcpListener::bind((bind_addr.clone(), bind_port.parse::<u16>().unwrap()))
        .await
        .unwrap();

    println!("Listening on {}:{}", bind_addr, bind_port);
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
                let packet: Packet =
                    Packet::from_u8_arr(&read).expect("Failed to deserialize received packet");
                println!(
                    "Packet from {}:\n Length: {}\n Request ID: {}\n Request Type: {:?}\n Payload: {}",
                    stream.peer_addr()?,
                    packet.length.unwrap(),
                    packet.request_id,
                    packet.packet_type,
                    strip_ansi_escapes::strip_str(packet.payload.clone().unwrap_or("empty".to_string()))
                );
                let _ = webhook
                    .send_if_some(packet.packet_type.into(), packet.payload.clone())
                    .await
                    .map_err(webhook::print_webhook_err);

                let handler: fn(Packet) -> Packet = match packet.packet_type {
                    PacketType::Login => handler_login,

                    PacketType::RunCommand => {
                        tokio::time::sleep(Duration::from_secs(1)).await; // Simulate delay for commands
                        handler_runcommand
                    }
                    _ => handler_invalid,
                };

                let response_packet = &handler(packet).into_vec();
                stream
                    .write_all(response_packet)
                    .await
                    .expect("Failed to write to stream");
            }
            Err(err) => bail!(err),
        }
    }
    Ok(())
}
