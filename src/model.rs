use core::fmt;

#[derive(Debug)]
pub struct Packet {
    pub length: Option<i32>,
    pub request_id: i32,
    pub packet_type: PacketType,
    pub payload: Option<String>,
}

#[derive(Debug, Copy, Clone)]
pub enum PacketType {
    Login,               // Packet type: 3
    Auth,                // Packet type: 2
    RunCommand,          // Packet type: 2
    MultiPacketResponse, // Packet type: 0
    Invalid(i32),
}

impl From<PacketType> for EventType {
    fn from(val: PacketType) -> Self {
        match val {
            PacketType::RunCommand => Self::RunCommand,
            PacketType::Auth => Self::Auth,
            PacketType::Login => Self::Auth,
            _ => Self::Invalid,
        }
    }
}

pub enum EventType {
    ClientConnect,
    Auth,
    RunCommand,
    ClientDisconnect,
    Invalid,
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ClientConnect => write!(f, "Client connected"),
            Self::RunCommand => write!(f, "Client executed command"),
            Self::ClientDisconnect => write!(f, "Client disconnected"),
            Self::Auth => write!(f, "Client logged in"),
            Self::Invalid => write!(f, "Invalid Event Type"),
        }
    }
}
