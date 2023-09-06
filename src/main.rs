mod conversions;
mod generator;
mod model;
mod logging;
mod webhook_model;
mod events;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use crate::{model::*, logging::logging::{EventLoggerType, Logger}};
use crate::events::ClientEventType;

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
        let peer_addr = stream.peer_addr().unwrap().to_string();
        let mut event_loggers: Vec<EventLoggerType> = vec![];
        event_loggers.push(EventLoggerType::StdOut);
        if let Some(webhook_url) = webhook_url.clone() {
            event_loggers.push(EventLoggerType::DiscordWebhook(
                Webhook::new(stream.peer_addr().unwrap().to_string(), &webhook_url)
            ))
        }
        let logger = Logger::new(event_loggers, peer_addr);
        logger.log(ClientEventType::Connect, None).await.expect("Failed to log event");

        let peer_addr = stream.peer_addr().unwrap().to_string();
        tokio::spawn(async move {
            match handle_client(
                stream,
                &mut webhook_url.clone().map(|url| Webhook::new(peer_addr, &url)),
                logger
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
    logger: Logger
) -> anyhow::Result<()> {
    let peer_addr: String = stream.peer_addr().unwrap().to_string();
    logger.log(ClientEventType::Connect, None).await.expect("Failed to log event");
    loop {
        let mut read = [0; 1024];
        match stream.read(&mut read).await {
            Ok(n) => {
                if n == 0 {
                    // connection was closed
                    println!("Connection to {} closed", stream.peer_addr()?);
                    break;
                }
                let packet: Packet =
                    Packet::from_u8_arr(&read).expect("Failed to deserialize recieved packet");
                match packet.packet_type {
                    PacketType::Login => {
                        logger.log(ClientEventType::Auth, packet.payload).await.expect("Failed to log event");

                        let response_packet = Packet::gen_auth_success(packet.request_id);
                        stream
                            .write(&response_packet.to_bytes())
                            .await
                            .expect("Failed to write to stream");
                    }

                    PacketType::RunCommand => {
                        let command = packet.payload.clone().unwrap_or("".to_string());
                        logger.log(ClientEventType::RunCommand, packet.payload).await.expect("Failed to log event");
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
