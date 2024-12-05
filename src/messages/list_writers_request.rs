use crate::messages::message::{MessagePayload, MessageType};
use anyhow::{Context, Result};
use rand::Rng;

pub struct ListWritersRequest {}

impl MessagePayload for ListWritersRequest {
    fn get_message_type(&self) -> MessageType {
        MessageType::ListWriters
    }

    fn serialize(&self) -> Result<Vec<u8>> {
        Ok(Vec::new())
    }

    fn deserialize(_buffer: &[u8]) -> Result<Self> {
        Ok(ListWritersRequest {})
    }
}
