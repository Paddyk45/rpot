use crate::model::Webhook;
use crate::webhook_model::*;
use crate::events::ClientEventType;

use super::traits::RpotLogger;
fn gen_codeblock(inp: String) -> String {
    format!("```{}```", inp)
}

impl RpotLogger for Webhook {
    fn handler(event_type: ClientEventType, payload: String, data: super::logging::LoggerData) {
        
    }
    fn create(data: super::logging::LoggerData) -> Self {
        
    }
}

impl Webhook {
    pub fn new(peer_addr: String, webhook_url: &String) -> Webhook {
        Webhook {
            peer_addr: peer_addr,
            webhook_url: webhook_url.to_string(),
            message_id: None,
            message_embed: None,
        }
    }

    pub async fn push(
        &mut self,
        event: ClientEventType,
        payload: Option<String>,
    ) -> anyhow::Result<()> {
        match self.message_id.clone() {
            None => {
                match event {
                    ClientEventType::Connect => {}
                    _ => panic!(
                        "You can only push to a new Webhook when the event type is ClientConnect"
                    ),
                }
                let mut embed: Embed = Embed::new();
                embed.author = Author {
                    name: self.peer_addr.clone(),
                };
                embed.color = 16746240; // orange
                embed.description = Some(gen_codeblock("Client connected".to_string()));
                embed.footer = Footer {
                    text: "RPot \u{2022} https://github.com/Paddyk45/rpot".to_string(),
                };
                self.message_embed = Some(embed);
                self.create_or_update().await.unwrap();
            }
            Some(_) => {
                let mut desc = self
                    .message_embed
                    .clone()
                    .unwrap()
                    .description
                    .unwrap_or("".to_string());
                let placeholder = match event {
                    ClientEventType::Connect | ClientEventType::Disconnect => "",
                    ClientEventType::Auth => "\n Password: ",
                    ClientEventType::RunCommand => "\n Command: ",
                };
                desc.push_str(&gen_codeblock(format!(
                    "\n{}{placeholder}{}",
                    event.to_string(),
                    payload.clone().unwrap_or("".to_string().replace("`", "")) // remove backticks so you can't end codeblock
                )));
                match event {
                    ClientEventType::Disconnect => {
                        self.message_embed.as_mut().unwrap().color = 15672064
                    } // change color to red if client disconnected
                    _ => {}
                }
                self.message_embed.as_mut().unwrap().description = Some(desc);
                self.create_or_update().await.unwrap();
            }
        }
        Ok(())
    }

    async fn create_or_update(&mut self) -> anyhow::Result<()> {
        if self.message_embed.as_ref().is_none() {
            panic!("Empty embed")
        }
        let (method, url) = match self.message_id {
            None => (reqwest::Method::POST, self.webhook_url.clone()),
            Some(_) => (
                reqwest::Method::PATCH,
                format!("{}/messages/{}", self.webhook_url, self.message_id.clone().unwrap()),
            ),
        };
        let mut webhook_request: WebhookRequest = WebhookRequest::new();
        webhook_request
            .embeds
            .push(self.message_embed.clone().unwrap());
        let response: WebhookResponse = reqwest::Client::new()
                    .request(method, url)
                    .json(&webhook_request)
                    .query(&[("wait", "true")])
                    .send()
                    .await?
                    .json()
                    .await?;
        self.message_id = Some(response.id);
        Ok(())
    }
}
