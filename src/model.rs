use core::fmt;

#[derive(Debug, Default)]
pub struct Packet {
    pub length: Option<i32>,
    pub request_id: i32,
    pub packet_type: PacketType,
    pub payload: Option<String>,
}

#[derive(Debug, Default, Copy, Clone)]
pub enum PacketType {
    Login, // Packet type: 3
    Auth,  // Packet type: 2
    #[default]
    RunCommand, // Packet type: 2
    MultiPacketResponse, // Packet type: 0
}

impl Into<EventType> for PacketType {
    fn into(self) -> EventType {
        match self {
            Self::RunCommand => EventType::RunCommand,
            Self::Auth => EventType::Auth,
            Self::Login => EventType::Auth,
            _ => EventType::RunCommand,
        }
    }
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
            Self::ClientConnect => write!(f, "Client connected"),
            Self::RunCommand => write!(f, "Client executed command"),
            Self::ClientDisconnect => write!(f, "Client disconnected"),
            Self::Auth => write!(f, "Client logged in"),
        }
    }
}
