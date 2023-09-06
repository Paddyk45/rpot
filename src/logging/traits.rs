use crate::events::ClientEventType;

use super::logging::LoggerData;

pub trait RpotLogger {
    fn handler(event_type: ClientEventType, payload: String, data: LoggerData);
    fn create(data: LoggerData) -> Self;
}