use crate::messages::message::{MessagePayload, MessageType};
use anyhow::{Context, Result};

pub struct ReadResponse {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

/// Layout of the ReadResponse
/// | 2 bytes | N bytes| 2 bytes | M bytes |
/// | keylen  |   key  | valuelen|  value  |
/// Integers are are always encoded in little-endian order
impl MessagePayload for ReadResponse {
    fn get_message_type(&self) -> MessageType {
        MessageType::Read
    }

    fn serialize(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        let key_len = u16::try_from(self.key.len()).context("key length overflow")?;
        let value_len = u16::try_from(self.value.len()).context("value length overflow")?;
        buffer.extend_from_slice(&key_len.to_le_bytes());
        buffer.extend_from_slice(&self.key);
        buffer.extend_from_slice(&value_len.to_le_bytes());
        buffer.extend_from_slice(&self.value);
        Ok(buffer)
    }

    fn deserialize(buffer: &[u8]) -> Result<Self> {
        let key_len = u16::from_le_bytes(
            buffer
                .get(0..2)
                .context("failed to get key length")?
                .try_into()?,
        ) as usize;
        let key = buffer
            .get(2..2 + key_len)
            .context("failed to get key")?
            .to_vec();
        let value_len = u16::from_le_bytes(
            buffer
                .get(2 + key_len..2 + key_len + 2)
                .context("failed to get value length")?
                .try_into()?,
        ) as usize;
        let value = buffer
            .get(2 + key_len + 2..2 + key_len + 2 + value_len)
            .context("failed to get value")?
            .to_vec();
        Ok(ReadResponse { key, value })
    }
}
