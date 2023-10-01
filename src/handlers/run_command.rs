use crate::model::Packet;

pub fn handler_runcommand(packet: Packet) -> Packet {
    let command = packet.payload.clone().unwrap_or_default();
    let command_response = match command
        .as_str()
        .split_whitespace()
        .next()
        .unwrap_or_default()
    {
        "seed" => "Seed: [69420]",
        "say" | "" => "",
        _ => "Unknown command. Type \"/help\" for help.\n",
    };
    Packet::gen_response(packet.request_id, command_response.to_string())
}
