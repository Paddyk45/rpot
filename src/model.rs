#[derive(Debug, Default)]
pub struct Packet {
    pub length: Option<i32>,
    pub request_id: i32,
    pub request_type: PacketType,
    pub payload: String,
}

#[derive(Debug, Default)]
pub enum PacketType {
    Login,               // Packet type: 3
    AuthSuccess,         // Packet type: 2
    #[default]
    RunCommand,          // Packet type: 2
    MultiPacketResponse, // Packet type: 0
}
