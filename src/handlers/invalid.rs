use crate::model::Packet;

pub fn handler_invalid(packet: Packet) -> Packet {
    let packet_id = packet.packet_type.as_i32();

    println!("Client sent an invalid packet type: {}", packet_id);
    Packet::gen_response(packet.request_id, format!("Unknown request {}", packet_id))
}
