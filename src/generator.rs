use crate::model::Packet;

impl Packet {
    pub fn gen_auth_success(request_id: i32) -> Self {
        Self {
            length: None,
            request_id,
            packet_type: crate::model::PacketType::Auth,
            payload: None,
        }
    }
    #[allow(dead_code)]
    pub fn gen_auth_fail() -> Self {
        Self {
            length: None,
            request_id: -1,
            packet_type: crate::model::PacketType::Auth,
            payload: None,
        }
    }

    pub fn gen_response(request_id: i32, payload: String) -> Self {
        Self {
            length: None,
            request_id,
            packet_type: crate::model::PacketType::MultiPacketResponse,
            payload: Some(payload),
        }
    }
}
