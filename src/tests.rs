use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;
use assert_cmd::prelude::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::{listener, main};
use crate::model::{Packet, PacketType};

const EMPTY_BUFFER: [u8; 1024] = [0u8; 1024];

#[tokio::test]
async fn test_basic() {
    // Spawn listener
    let addr = Ipv4Addr::new(0, 0, 0, 0);
    let port = 25575;
    let sock_addr = SocketAddr::new(IpAddr::V4(addr), port);
    let listener = tokio::spawn(listener(sock_addr, None));
    // Wait for listener to start
    tokio::time::sleep(Duration::from_secs(1)).await;
    let mut stream = TcpStream::connect("0.0.0.0:25575").await.expect("Failed to connect to RCon server");


    // TEST LOGIN
    let packet = Packet {
        length: None,
        request_id: 1337,
        packet_type: PacketType::Login,
        payload: Some("minecraft".into()),
    };
    stream.write(&packet.serialize()).await.expect("Failed to write to stream");

    let mut read = EMPTY_BUFFER;
    let read_bytes = stream.read(&mut read).await.expect("Failed to read from stream");
    let read = read[..read_bytes].to_vec();
    let packet = Packet::try_deserialize(read).expect("Failed to deserialize packet");

    assert_eq!(packet.length, Some(read_bytes as i32 - 4), "Length is not correct");
    assert_eq!(packet.request_id, 1337, "Request IDs are not the same");
    assert_eq!(*packet.packet_type.as_i32(), 2, "Packet type is not Auth");
    assert_eq!(packet.payload, None, "Payload is not empty");


    // TEST SEED COMMAND
    let packet = Packet {
        length: None,
        request_id: 1337,
        packet_type: PacketType::RunCommand,
        payload: Some(String::from("seed")),
    };
    stream.write(&packet.serialize()).await.expect("Failed to write to stream");

    let mut read = EMPTY_BUFFER;
    let read_bytes = stream.read(&mut read).await.expect("Failed to read from stream");
    let read = read[..read_bytes].to_vec();
    let packet = Packet::try_deserialize(read).expect("Failed to deserialize packet");

    assert_eq!(packet.length, Some(read_bytes as i32 - 4), "Length is not correct");
    assert_eq!(packet.request_id, 1337, "Request IDs are not the same");
    assert_eq!(packet.packet_type, PacketType::MultiPacketResponse, "Packet type is not MultiPacketResponse");
    let paylad = packet.payload.expect("Payload is empty");
    assert!(paylad.starts_with("Seed: [") && paylad.ends_with("]"), "Command response is not correct");
}

#[tokio::test]
async fn test_wrong_length() {
    // Spawn listener
    let addr = Ipv4Addr::new(0, 0, 0, 0);
    let port = 25575;
    let sock_addr = SocketAddr::new(IpAddr::V4(addr), port);
    tokio::spawn(listener(sock_addr, None));
    // Wait for listener to start
    tokio::time::sleep(Duration::from_secs(1)).await;
    let mut stream = TcpStream::connect("0.0.0.0:25575").await.unwrap();


    // TEST LENGTH TOO HIGH
    let packet = Packet {
        length: Some(20),
        request_id: 1337,
        packet_type: PacketType::Login,
        payload: Some(String::from("minecraft")),
    };
    stream.write(&packet.serialize()).await.expect("Failed to write to stream");

    let mut read = EMPTY_BUFFER;
    let res = tokio::time::timeout(Duration::from_secs(1), stream.read(&mut read)).await;
    assert!(matches!(res, Err(_)), "Read did not time out");

    stream.shutdown().await.expect("Failed to shutdown stream");
    let mut stream = TcpStream::connect("0.0.0.0:25575").await.expect("Failed to connect to RCon server");


    // TEST LENGTH TOO LOW
    println!("Testing LENGTH TOO LOW");
    let packet = Packet {
        length: Some(2),
        request_id: 1337,
        packet_type: PacketType::Login,
        payload: Some(String::from("minecraft")),
    };

    stream.write(&packet.serialize()).await.expect("Failed to write to stream");

    let mut read = EMPTY_BUFFER;
    let res = tokio::time::timeout(Duration::from_secs(1), stream.read(&mut read)).await;

    assert!(matches!(res, Err(_)), "Read did not time out");


    // TEST LENGTH NEGATIVE
    println!("Testing LENGTH NEGATIVE");
    let packet = Packet {
        length: Some(-99),
        request_id: 1337,
        packet_type: PacketType::Login,
        payload: Some(String::from("minecraft")),
    };
    stream.write(&packet.serialize()).await.expect("Failed to write to stream");

    let mut read = EMPTY_BUFFER;
    let res = tokio::time::timeout(Duration::from_secs(1), stream.read(&mut read)).await;

    assert!(matches!(res, Err(_)), "Read did not time out");
}

#[tokio::test]
async fn test_pre_auth_run_command() {
    // Spawn listener
    let addr = Ipv4Addr::new(0, 0, 0, 0);
    let port = 25575;
    let sock_addr = SocketAddr::new(IpAddr::V4(addr), port);
    tokio::spawn(listener(sock_addr, None));
    // Wait for listener to start
    tokio::time::sleep(Duration::from_secs(1)).await;
    let mut stream = TcpStream::connect("0.0.0.0:25575").await.expect("Failed to connect to RCon server");


    // TEST RUNNING COMMAND BEFORE LOGGING IN
    let packet = Packet {
        length: None,
        request_id: 1337,
        packet_type: PacketType::RunCommand,
        payload: Some(String::from("seed")),
    };
    stream.write(&packet.serialize()).await.expect("Failed to write to stream");

    let mut read = EMPTY_BUFFER;
    let read_bytes = stream.read(&mut read).await.expect("Failed to read from stream");
    let read = read[..read_bytes].to_vec();
    let packet = Packet::try_deserialize(read).expect("Failed to deserialize packet");

    assert_eq!(packet.request_id, -1, "Request ID is not -1");
}