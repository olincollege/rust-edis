use crate::messages::message::{MessagePayload, MessageType};
use anyhow::{Context, Result};
use rand::Rng;

pub struct WriteRequest {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

/// Layout of the WriteRequest as described in architecture
/// | 2 bytes | N bytes | 2 bytes | M bytes |
/// | keylen  |   key   | valuelen|  value  |
/// Integers are are always encoded in little-endian order
impl MessagePayload for WriteRequest {
    fn get_message_type(&self) -> MessageType {
        MessageType::Write
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
        Ok(WriteRequest { key, value })
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_basic() {
        let original = WriteRequest {
            key: b"key".to_vec(),
            value: b"value".to_vec(),
        };
        let serialized = original.serialize().unwrap();
        let deserialized = WriteRequest::deserialize(&serialized).unwrap();
        assert_eq!(original.key, deserialized.key);
        assert_eq!(original.value, deserialized.value);
    }

    #[test]
    fn test_roundtrip_random() {
        for _ in 0..1000 {
            let mut rng = rand::thread_rng();
            let key_len = rng.gen_range(0..1001);
            let key: Vec<u8> = (0..key_len).map(|_| rng.gen()).collect();

            let value_len = rng.gen_range(0..1001);
            let value: Vec<u8> = (0..value_len).map(|_| rng.gen()).collect();

            let original = WriteRequest { key, value };
            let serialized = original.serialize().unwrap();
            let deserialized = WriteRequest::deserialize(&serialized).unwrap();

            assert_eq!(original.key, deserialized.key);
            assert_eq!(original.value, deserialized.value);
        }
    }
}
