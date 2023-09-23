use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct WebhookRequest {
    pub embeds: Vec<Embed>,
}
impl WebhookRequest {
    pub fn new() -> WebhookRequest {
        WebhookRequest { embeds: vec![] }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Embed {
    pub author: Author,
    pub description: Option<String>,
    pub color: u32,
    pub footer: Footer,
}
impl Embed {
    pub fn new() -> Embed {
        Embed {
            author: Author {
                name: "".to_string(),
            },
            description: None,
            color: 0,
            footer: Footer {
                text: "".to_string(),
            },
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Author {
    pub name: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct Footer {
    pub text: String,
}

#[derive(Deserialize, Debug)]
pub struct WebhookResponse {
    pub id: String,
}

#[derive(Debug)]
pub struct Webhook {
    pub peer_addr: String,
    pub webhook_url: String,
    pub message_id: Option<String>,
    pub message_embed: Option<Embed>,
}

pub struct MaybeWebhook {
    pub webhook: Option<Webhook>,
}
