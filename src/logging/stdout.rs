use crate::model::Webhook;

use super::traits::RpotLogger;

#[derive(Clone)]
pub struct StdOutLogger {}

impl RpotLogger for StdOutLogger {
    fn handler(event_type: crate::events::ClientEventType, payload: String, data: super::logging::LoggerData) {
        
    }

    fn create(data: super::logging::LoggerData) -> Self {
        let instance = StdOutLogger {};
        instance
    }
}