use crate::messages::message::{MessagePayload, MessageType};
use anyhow::Result;

pub struct WriteResponse {
    status: Vec<u8>,
}

impl MessagePayload for WriteResponse {
    fn get_message_type(&self) -> MessageType {
        MessageType::Write
    }

    fn serialize(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        buffer.extend_from_slice(&self.status);
        Ok(buffer)
    }

    fn deserialize(buffer: &[u8]) -> Result<Self> {
        let status = buffer.to_vec();
        Ok(WriteResponse { status })
    }
}
