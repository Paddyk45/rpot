use anyhow::bail;

use crate::model::EventType;
use crate::webhook_model::{
    Author, Embed, Footer, MaybeWebhook, Webhook, WebhookRequest, WebhookResponse,
};
fn gen_codeblock<T: ToString>(inp: &T) -> String {
    format!("```{}```", inp.to_string())
}

pub fn print_webhook_err(err: anyhow::Error) {
    println!("Error sending to webhook: {}", err)
}

impl From<Option<Webhook>> for MaybeWebhook {
    fn from(value: Option<Webhook>) -> Self {
        Self(value)
    }
}

impl MaybeWebhook {
    pub async fn send_if_some(
        &mut self,
        event_type: EventType,
        payload: Option<String>,
    ) -> anyhow::Result<()> {
        if let Some(webhook) = self.0.as_mut() {
            webhook.push(event_type, payload).await?;
        }
        Ok(())
    }
}

impl Webhook {
    pub fn new(peer_addr: String, webhook_url: String) -> Self {
        Self {
            peer_addr,
            webhook_url,
            message_id: None,
            message_embed: None,
        }
    }

    pub async fn push(&mut self, event: EventType, payload: Option<String>) -> anyhow::Result<()> {
        match self.message_id.clone() {
            None => {
                if !matches!(event, EventType::ClientConnect) {
                    anyhow::bail!(
                        "You can only push to a new Webhook when the event type is ClientConnect"
                    )
                }
                self.message_embed = Some(Embed {
                    author: Author {
                        name: self.peer_addr.clone(),
                    },
                    color: 0xFF_87_00, // orange
                    description: Some(gen_codeblock(&"Client connected")),
                    footer: Footer {
                        text: "RPot \u{2022} https://github.com/Paddyk45/rpot".to_string(),
                    },
                });

                self.create_or_update().await?;
            }
            Some(msgid) => {
                if msgid == *"ERROR".to_string() {
                    bail!("Initial Webhook request returned error")
                }
                let mut desc = self
                    .message_embed
                    .clone()
                    .unwrap()
                    .description
                    .unwrap_or_default();
                let placeholder = match event {
                    EventType::ClientConnect | EventType::ClientDisconnect | EventType::Invalid=> "",
                    EventType::Auth => "\n Password: ",
                    EventType::RunCommand => "\n Command: ",
                };
                desc.push_str(&gen_codeblock(&format!(
                    "\n{}{placeholder}{}",
                    event.to_string(),
                    payload.clone().unwrap_or_default().replace('`', "") // remove backticks so you can't end codeblock
                )));
                // change color to red if client disconnected
                if matches!(event, EventType::ClientDisconnect) {
                    self.message_embed.as_mut().unwrap().color = 15672064;
                }

                self.message_embed.as_mut().unwrap().description = Some(desc);
                self.create_or_update().await?;
            }
        }
        Ok(())
    }

    async fn create_or_update(&mut self) -> anyhow::Result<()> {
        if self.message_embed.as_ref().is_none() {
            bail!("Empty Embed")
        }
        let (method, url) = match self.message_id.clone() {
            None => (reqwest::Method::POST, self.webhook_url.clone()),
            Some(msgid) => (
                reqwest::Method::PATCH,
                format!("{}/messages/{}", self.webhook_url, msgid),
            ),
        };
        let mut webhook_request: WebhookRequest = WebhookRequest::new();
        webhook_request
            .embeds
            .push(self.message_embed.clone().unwrap());
        let response = reqwest::Client::new()
            .request(method, url)
            .json(&webhook_request)
            .query(&[("wait", "true")])
            .send()
            .await?;
        if response.status() != 200 {
            self.message_id = Some("ERROR".to_string());
            bail!(format!(
                "Discord API returned non-200 status code. Body: {}",
                response.text().await?
            ))
        }
        let response: WebhookResponse = response.json().await?;
        self.message_id = Some(response.id);
        Ok(())
    }
}
