mod conversions;
mod generator;
mod model;
mod webhook;
mod webhook_model;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use webhook_model::{MaybeWebhook, Webhook};

use crate::model::*;

// This is based on https://gist.github.com/fortruce/828bcc3499eb291e7e17
#[tokio::main]
async fn main() {
    let bind_addr = std::env::var("RPOT_BIND_ADDR").unwrap_or("0.0.0.0".to_string());
    let bind_port = std::env::var("RPOT_BIND_PORT").unwrap_or("25575".to_string());
    let webhook_url = match std::env::var("RPOT_WEBHOOK_URL") {
        Ok(val) => Some(val),
        Err(_) => None,
    };
    let listener = TcpListener::bind((bind_addr.clone(), bind_port.parse::<u16>().unwrap()))
        .await
        .unwrap();

    println!("Listening on {}:{}", bind_addr, bind_port);
    while let Ok(stream) = listener.accept().await {
        let stream = stream.0;
        let webhook_url = webhook_url.clone();
        println!(
            "Connection to {} opened",
            stream.peer_addr().unwrap_or("0.0.0.0:1".parse().unwrap())
        );
        let peer_addr = stream.peer_addr().unwrap().to_string();
        let x = webhook_url.map(|url| Webhook::new(peer_addr, url));
        let mut webhook: MaybeWebhook = x.into();
        tokio::spawn(async move {
            match handle_client(stream, &mut webhook).await {
                Ok(()) => {}
                Err(err) => println!("Error handling client: {}", err),
            };
        });
    }
}

async fn handle_client(mut stream: TcpStream, webhook: &mut MaybeWebhook) -> anyhow::Result<()> {
    webhook.send_if_some(EventType::ClientConnect, None).await?;

    loop {
        let mut read = [0; 1024];
        match stream.read(&mut read).await {
            Ok(n) => {
                if n == 0 {
                    // connection was closed
                    println!("Connection to {} closed", stream.peer_addr()?);
                    webhook
                        .send_if_some(EventType::ClientDisconnect, None)
                        .await?;

                    break;
                }
                let packet: Packet =
                    Packet::from_u8_arr(&read).expect("Failed to deserialize recieved packet");
                println!(
                    "Packet from {}:\n Length: {}\n Request ID: {}\n Request Type: {:?}\n Payload: {}",
                    stream.peer_addr()?,
                    packet.length.unwrap(),
                    packet.request_id,
                    packet.packet_type,
                    strip_ansi_escapes::strip_str(packet.payload.clone().unwrap_or("empty".to_string()))
                );
                match packet.packet_type {
                    PacketType::Login => {
                        webhook
                            .send_if_some(EventType::Auth, packet.payload)
                            .await?;
                        let response_packet = Packet::gen_auth_success(packet.request_id);
                        stream
                            .write(&response_packet.to_bytes())
                            .await
                            .expect("Failed to write to stream");
                    }

                    PacketType::RunCommand => {
                        let command = packet.payload.clone().unwrap_or("".to_string());
                        webhook
                            .send_if_some(EventType::RunCommand, packet.payload)
                            .await?;
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
                                .to_bytes(),
                            )
                            .await
                            .expect("Failed to write to stream");
                    }

                    _ => println!("Client sent invalid packet type"),
                }
            }
            Err(err) => panic!("{}", err),
        }
    }
    Ok(())
}
