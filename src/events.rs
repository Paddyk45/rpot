use std::fmt;

use anyhow::Ok;

use crate::model::Webhook;

#[derive(Debug, Clone)]
pub enum ClientEventType {
    Connect,
    Auth,
    RunCommand,
    Disconnect,
}

impl fmt::Display for ClientEventType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ClientEventType::Connect => write!(f, "Client connected"),
            ClientEventType::RunCommand => write!(f, "Client executed command"),
            ClientEventType::Disconnect => write!(f, "Client disconnected"),
            ClientEventType::Auth => write!(f, "Client logged in"),
        }
    }
}

