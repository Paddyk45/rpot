use crate::model::{EventType, Webhook};
use crate::webhook_model::*;
use std::fmt;
use std::ops::Deref;

fn gen_codeblock(inp: String) -> String {
    format!("```{}```", inp)
}

impl Webhook {
    pub fn new(peer_addr: String, webhook_url: String) -> Webhook {
        Webhook {
            peer_addr: peer_addr,
            webhook_url: webhook_url,
            message_id: None,
            message_embed: None,
        }
    }

    pub fn push(
        &mut self,
        event: EventType,
        payload: Option<String>,
    ) -> Result<(), failure::Error> {
        match self.message_id.clone() {
            None => {
                match event {
                    EventType::ClientConnect => {}
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
                self.create().unwrap();
                println!("{:?}", self.message_id);
            }
            Some(msg_id) => {
                let mut descr = self
                    .message_embed
                    .clone()
                    .unwrap()
                    .description
                    .unwrap_or("".to_string());
                let placeholder = match event {
                    EventType::ClientConnect | EventType::ClientDisconnect => "",
                    EventType::Auth => "\n Password: ",
                    EventType::RunCommand => "\n Command:",
                };
                descr.push_str(&gen_codeblock(format!(
                    "\n{}{placeholder}{}",
                    event.to_string(),
                    payload.clone().unwrap_or("".to_string())
                )));
                self.message_embed.as_mut().unwrap().description = Some(descr);
                self.update().unwrap();
            }
        }
        Ok(())
    }

    fn create(&mut self) -> Result<(), failure::Error> {
        if self.message_embed.as_ref().is_none() {
            panic!("Empty embed")
        }
        let mut webhook_request: WebhookRequest = WebhookRequest::new();
        webhook_request
            .embeds
            .push(self.message_embed.clone().unwrap());
        let response: WebhookResponse =
            ureq::post(format!("{}?wait=true", &self.webhook_url).as_str())
                .send_json(webhook_request)?
                .into_json()?;
        self.message_id = Some(response.id);
        Ok(())
    }

    fn update(&mut self) -> Result<(), failure::Error> {
        if self.message_embed.as_ref().is_none() {
            panic!("Empty embed")
        }
        let mut webhook_request: WebhookRequest = WebhookRequest::new();
        webhook_request
            .embeds
            .push(self.message_embed.clone().unwrap());
        let response: WebhookResponse = ureq::patch(
            format!(
                "{}/messages/{}?wait=true",
                &self.webhook_url,
                self.message_id.as_ref().unwrap()
            )
            .as_str(),
        )
        .send_json(webhook_request)?
        .into_json()?;
        self.message_id = Some(response.id);
        Ok(())
    }
}
