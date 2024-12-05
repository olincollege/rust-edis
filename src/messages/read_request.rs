use crate::messages::message::{MessagePayload, MessageType};
use anyhow::{Context, Result};
use rand::Rng;

pub struct ReadRequest {
    pub key: Vec<u8>,
}

/// Layout of the ReadRequest
/// | 2 bytes | N bytes |
/// | keylen  |   key   |
impl MessagePayload for ReadRequest {
    fn get_message_type(&self) -> MessageType {
        MessageType::Read
    }
    fn serialize(&self) -> Result<Vec<u8>> {
        let key_len = u16::try_from(self.key.len()).context("key length overflow")?;
        let mut buffer = Vec::new();
        buffer.extend_from_slice(&key_len.to_le_bytes());
        buffer.extend_from_slice(&self.key);
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
        Ok(ReadRequest { key })
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_basic() {
        let original = ReadRequest {
            key: b"key".to_vec(),
        };
        let serialized = original.serialize().unwrap();
        let deserialized = ReadRequest::deserialize(&serialized).unwrap();
        assert_eq!(original.key, deserialized.key);
    }

    #[test]
    fn test_roundtrip_random() {
        for _ in 0..1000 {
            let mut rng = rand::thread_rng();
            let key_len = rng.gen_range(0..1001);
            let key: Vec<u8> = (0..key_len).map(|_| rng.gen()).collect();

            let original = ReadRequest { key };
            let serialized = original.serialize().unwrap();
            let deserialized = ReadRequest::deserialize(&serialized).unwrap();

            assert_eq!(original.key, deserialized.key);
        }
    }
}
