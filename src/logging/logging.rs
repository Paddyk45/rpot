use crate::{model::Webhook, events::ClientEventType};

use super::{stdout::StdOutLogger, traits::RpotLogger};

type WebhookURL = String;

#[derive(Clone)]
pub enum EventLoggerType {
    DiscordWebhook(WebhookURL),
    //AbuseIPDB,
    StdOut,
}

#[derive(Clone)]
pub enum EventLogger {
    DiscordWebhook(Webhook),
    //AbuseIPDB,
    StdOut(StdOutLogger),
}

#[derive(Clone)]
pub struct Logger {
    logger_instances: Vec<Box<EventLoggerType>>,
    data: LoggerData,
}

#[derive(Clone)]
pub struct LoggerData {
    pub peer_addr: String
}

pub struct RpotLoggerWrapper<T: RpotLogger> {
    instance: T
}

impl Logger {
    pub fn new(event_loggers: Vec<EventLoggerType>, peer_addr: String) -> Logger {
        let data = LoggerData { peer_addr: peer_addr };
        let mut instances: Vec<Box<dyn RpotLogger>>;
        for logger_type in event_loggers {
            match logger_type {
                EventLoggerType::DiscordWebhook(url) => instances.push(RpotLoggerWrapper::<Webhook> { instance: Webhook::create(data) }),
                EventLoggerType::StdOut => instances.push(RpotLoggerWrapper::<StdOutLogger> { instance: StdOutLogger{} })
            }
        }
        Logger { logger_instances: instances, data: data }
    }

    pub async fn log(&self, event_type: ClientEventType, payload: Option<String>) -> anyhow::Result<()> {
        for event_logger in self.event_loggers.clone() {
            match event_logger {
                EventLoggerType::DiscordWebhook(mut webhook) => webhook.push(event_type.clone(), payload.clone()).await?,
                EventLoggerType::StdOut(StdOutLogger) => {}
            };
        };
        Ok(())
    }
}