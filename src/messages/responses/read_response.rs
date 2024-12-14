use crate::messages::message::{MessagePayload, MessageType};
use anyhow::{Context, Result};
use int_enum::IntEnum;

#[repr(u8)]
#[derive(Debug, Clone, Copy, IntEnum, PartialEq, Eq)]
pub enum ReadResponseError {
    NoError = 0,
    KeyNotFound = 1,
}

#[derive(Clone)]
pub struct ReadResponse {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
    pub error: u8,
}

/// Layout of the ReadResponse
/// | 1 byte | 2 bytes | N bytes| 2 bytes | M bytes |
/// | error  | keylen  |   key  | valuelen|  value  |
/// Integers are are always encoded in little-endian order
impl MessagePayload for ReadResponse {
    fn is_request(&self) -> bool {
        false
    }

    fn get_message_type(&self) -> MessageType {
        MessageType::Read
    }

    fn serialize(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        buffer.push(self.error);
        let key_len = u16::try_from(self.key.len()).context("key length overflow")?;
        let value_len = u16::try_from(self.value.len()).context("value length overflow")?;
        buffer.extend_from_slice(&key_len.to_le_bytes());
        buffer.extend_from_slice(&self.key);
        buffer.extend_from_slice(&value_len.to_le_bytes());
        buffer.extend_from_slice(&self.value);
        Ok(buffer)
    }

    fn deserialize(buffer: &[u8]) -> Result<Self> {
        let error = *buffer.get(0).context("failed to get error")?;
        let key_len = u16::from_le_bytes(
            buffer
                .get(1..3)
                .context("failed to get key length")?
                .try_into()?,
        ) as usize;
        let key = buffer
            .get(3..3 + key_len)
            .context("failed to get key")?
            .to_vec();
        let value_len = u16::from_le_bytes(
            buffer
                .get(3 + key_len..3 + key_len + 2)
                .context("failed to get value length")?
                .try_into()?,
        ) as usize;
        let value = buffer
            .get(3 + key_len + 2..3 + key_len + 2 + value_len)
            .context("failed to get value")?
            .to_vec();
        Ok(ReadResponse { error, key, value })
    }
}
