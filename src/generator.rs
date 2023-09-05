use crate::model::Packet;

impl Packet {
    pub fn gen_auth_success(request_id: i32) -> Packet {
        Packet {
            length: None,
            request_id: request_id,
            packet_type: crate::model::PacketType::Auth,
            payload: None,
        }
    }
    #[allow(dead_code)]
    pub fn gen_auth_fail() -> Packet {
        Packet {
            length: None,
            request_id: -1,
            packet_type: crate::model::PacketType::Auth,
            payload: None,
        }
    }

    pub fn gen_response(request_id: i32, payload: String) -> Packet {
        Packet {
            length: None,
            request_id: request_id,
            packet_type: crate::model::PacketType::Auth,
            payload: Some(payload),
        }
    }
}
