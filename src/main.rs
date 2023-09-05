mod conversions;
mod generator;
mod model;
mod webhook;
mod webhook_model;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use crate::model::*;

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
        tokio::spawn(async move {
            match handle_client(
                stream,
                &mut webhook_url.map(|url| Webhook::new(peer_addr, url)),
            )
            .await
            {
                Ok(()) => {}
                Err(err) => println!("Error handling client: {}", err),
            };
        });
    }
}

async fn handle_client(
    mut stream: TcpStream,
    webhook: &mut Option<Webhook>,
) -> anyhow::Result<()> {
    if let Some(webhook) = webhook {
        webhook
            .push(EventType::ClientConnect, None)
            .await
            .expect("Failed to push event to Webhook");
    }
    loop {
        let mut read = [0; 1024];
        match stream.read(&mut read).await {
            Ok(n) => {
                if n == 0 {
                    // connection was closed
                    println!("Connection to {} closed", stream.peer_addr()?);
                    if let Some(webhook) = webhook {
                        webhook
                            .push(EventType::ClientDisconnect, None)
                            .await
                            .expect("Failed to push event to Webhook");
                    }
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
                        if let Some(webhook) = webhook {
                            webhook
                                .push(EventType::Auth, packet.payload)
                                .await
                                .expect("Failed to push event to Webhook");
                        }

                        let response_packet = Packet::gen_auth_success(packet.request_id);
                        stream
                            .write(&response_packet.to_bytes())
                            .await
                            .expect("Failed to write to stream");
                    }

                    PacketType::RunCommand => {
                        let command = packet.payload.clone().unwrap_or("".to_string());
                        if let Some(webhook) = webhook {
                            webhook
                                .push(EventType::RunCommand, packet.payload)
                                .await
                                .expect("Failed to push event to Webhook");
                        }
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
