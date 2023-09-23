use core::fmt;

#[derive(Debug, Default)]
pub struct Packet {
    pub length: Option<i32>,
    pub request_id: i32,
    pub packet_type: PacketType,
    pub payload: Option<String>,
}

#[derive(Debug, Default)]
pub enum PacketType {
    Login, // Packet type: 3
    Auth,  // Packet type: 2
    #[default]
    RunCommand, // Packet type: 2
    MultiPacketResponse, // Packet type: 0
}

pub enum EventType {
    ClientConnect,
    Auth,
    RunCommand,
    ClientDisconnect,
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EventType::ClientConnect => write!(f, "Client connected"),
            EventType::RunCommand => write!(f, "Client executed command"),
            EventType::ClientDisconnect => write!(f, "Client disconnected"),
            EventType::Auth => write!(f, "Client logged in"),
        }
    }
}
