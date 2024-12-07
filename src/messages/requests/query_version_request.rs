use crate::messages::message::{MessagePayload, MessageType};
use anyhow::Result;

pub struct QueryVersionRequest {}

impl MessagePayload for QueryVersionRequest {
    fn get_message_type(&self) -> MessageType {
        MessageType::QueryVersion
    }

    fn serialize(&self) -> Result<Vec<u8>> {
        Ok(Vec::new())
    }

    fn deserialize(_buffer: &[u8]) -> Result<Self> {
        Ok(QueryVersionRequest {})
    }
}
