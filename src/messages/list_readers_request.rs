use crate::messages::message::{MessagePayload, MessageType};
use anyhow::{Context, Result};
use rand::Rng;

pub struct ListReadersRequest {}

impl MessagePayload for ListReadersRequest {
    fn get_message_type(&self) -> MessageType {
        MessageType::ListReaders
    }

    fn serialize(&self) -> Result<Vec<u8>> {
        Ok(Vec::new())
    }

    fn deserialize(_buffer: &[u8]) -> Result<Self> {
        Ok(ListReadersRequest {})
    }
}
