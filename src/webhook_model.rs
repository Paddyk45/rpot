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
    pub fields: Vec<Field>,
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
            fields: vec![],
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
pub struct Field {
    pub name: String,
    pub value: String,
    pub inline: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct Footer {
    pub text: String,
}

#[derive(Deserialize, Debug)]
pub struct WebhookResponse {
    pub id: String,
}
