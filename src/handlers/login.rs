use crate::model::Packet;

pub fn handler_login(packet: Packet) -> Packet {
    match packet.payload {
        Some(_) => Packet::gen_auth_success(packet.request_id),
        None => Packet::gen_auth_fail()
    }
}