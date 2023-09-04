use std::fmt;

pub enum EventType {
    ClientConnect,
    Auth,
    RunCommand,
    ClientDisconnect,
}
impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EventType::ClientConnect => write!(f, "Client connected"),
            EventType::RunCommand => write!(f, "Client executed command"),
            EventType::ClientDisconnect => write!(f, "Client disconnected"),
            EventType::Auth => write!(f, "Client logged in")
        }
    }
}

pub fn webhook_handler(
    webhook_url: String,
    peer_addr: String,
    event: EventType,
    payload: Option<String>,
) -> Result<(), failure::Error> {
    let color = match event {
        EventType::ClientConnect => 37120,
        EventType::RunCommand => 12692023,
        EventType::ClientDisconnect => 10092544,
        EventType::Auth => 3088371
    };
    let payload_field_name = match event {
        EventType::ClientConnect => "",
        EventType::Auth => "Password",
        EventType::RunCommand => "Command",
        EventType::ClientDisconnect => ""
    }.to_string();
    let response = ureq::post(webhook_url.as_str())
        .send_json(ureq::json!({
            "embeds": [
                {
                    "author": {
                        "name": peer_addr
                    },
                    "color": color,
                    "fields":
                    [
                        {
                            "name": "Event Type",
                            "value": event.to_string(),
                            "inline": false
                        },
                        {
                            "name": payload_field_name,
                            "value": payload.unwrap_or("".to_string()),
                            "inline": false
                        }
                    ],
                    "footer": {
                        "text": "RPot - https://github.com/Paddyk45/rpot"
                    }
                }
            ]
        }))?
        .into_string()?;
    Ok(())
}
