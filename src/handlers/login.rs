use crate::model::Packet;

pub fn handler_login(packet: Packet) -> Packet {
    Packet::gen_auth_success(packet.request_id)
}
