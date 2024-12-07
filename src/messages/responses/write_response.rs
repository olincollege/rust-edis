use crate::messages::message::{MessagePayload, MessageType};
use anyhow::{Context, Result};
use int_enum::IntEnum;

#[repr(u8)]
#[derive(Debug, Clone, Copy, IntEnum, PartialEq, Eq)]
pub enum WriteResponseError {
    NoError = 0,
    Error = 1,
}

pub struct WriteResponse {
    pub error: u8,
}

/// Layout of the WriteResponse
/// | 1 byte |
/// | error |
impl MessagePayload for WriteResponse {
    fn get_message_type(&self) -> MessageType {
        MessageType::Write
    }

    fn serialize(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        buffer.push(self.error);
        Ok(buffer)
    }

    fn deserialize(buffer: &[u8]) -> Result<Self> {
        let error = *buffer.get(0).context("failed to get error")?;
        Ok(WriteResponse { error })
    }
}
