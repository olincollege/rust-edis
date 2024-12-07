use crate::messages::message::{MessagePayload, MessageType};
use anyhow::{Context, Result};
use int_enum::IntEnum;

#[repr(u8)]
#[derive(Debug, Clone, Copy, IntEnum, PartialEq, Eq)]
pub enum ReadResponseError {
    NoError = 0,
    KeyNotFound = 1,
}

pub struct ReadResponse {
    pub value: Vec<u8>,
    pub error: u8,
}

/// Layout of the ReadResponse
/// | 1 byte  | 2 bytes | N bytes |
/// | error   | valuelen|  value  |
/// Integers are are always encoded in little-endian order
impl MessagePayload for ReadResponse {
    fn get_message_type(&self) -> MessageType {
        MessageType::Read
    }

    fn serialize(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        buffer.push(self.error);
        let value_len = u16::try_from(self.value.len()).context("value length overflow")?;
        buffer.extend_from_slice(&value_len.to_le_bytes());
        buffer.extend_from_slice(&self.value);
        Ok(buffer)
    }

    fn deserialize(buffer: &[u8]) -> Result<Self> {
        let error = *buffer.get(0).context("failed to get error")?;
        let value_len = u16::from_le_bytes(
            buffer
                .get(1..3)
                .context("failed to get value length")?
                .try_into()?,
        );
        let value = buffer
            .get(3..3 + value_len as usize)
            .context("failed to get value")?
            .to_vec();
        Ok(ReadResponse { value, error })
    }
}
