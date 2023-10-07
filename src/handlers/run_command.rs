use crate::model::Packet;

pub fn handler_runcommand(packet: Packet) -> Packet {
    let split_args = packet.payload.clone().unwrap_or_default();
    let mut split_args = split_args.split_whitespace();
    let args: Vec<&str> = split_args.clone().skip(1).collect();
    let command = split_args.next();
    if let Some(cmd) = command {
        let resp = match cmd {
            "seed" => "Seed: [69420]".to_owned(),
            "say" | "" => "".to_owned(),
            "help" => "§e--------- §fHelp: Index (1/11) §e--------------------
§7Use /help [n] to get page n of help.
§7§6Aliases: §fLists command aliases
§f§6Bukkit: §fAll commands for Bukkit
§f§6Minecraft: §fAll commands for Minecraft
§f§6/advancement: §fA Mojang provided command.
§f§6/attribute: §fA Mojang provided command.
§f§6/ban: §fA Mojang provided command.
§f§6/ban-ip: §fA Mojang provided command.
§f§6/banlist: §fA Mojang provided command.\n"
                .to_owned(),
            "op" => args.first().map_or_else(
                || "Unknown or incomplete command, see below for error\nop<--[HERE]".to_owned(),
                |name| format!("Made {} a server operator", name),
            ),
            _ => "Unknown command. Type \"/help\" for help.\n".to_owned(),
        };
        Packet::gen_response(packet.request_id, resp)
    } else {
        Packet::gen_response(packet.request_id, String::new())
    }
}
