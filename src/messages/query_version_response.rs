use crate::messages::message::{MessagePayload, MessageType};
use anyhow::{Context, Result};
use rand::Rng;

/// Layout of the QueryVersionResponse
/// | 8 bytes |
/// | version |
pub struct QueryVersionResponse {
    pub version: u64,
}

impl MessagePayload for QueryVersionResponse {
    fn get_message_type(&self) -> MessageType {
        MessageType::QueryVersion
    }

    fn serialize(&self) -> Result<Vec<u8>> {
        Ok(self.version.to_le_bytes().to_vec())
    }

    fn deserialize(buffer: &[u8]) -> Result<Self> {
        Ok(QueryVersionResponse {
            version: u64::from_le_bytes(buffer.try_into().context("Invalid buffer size")?),
        })
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_basic() {
        let response = QueryVersionResponse { version: 100 };
        let serialized = response.serialize().unwrap();
        let deserialized = QueryVersionResponse::deserialize(&serialized).unwrap();
        assert_eq!(response.version, deserialized.version);
    }

    #[test]
    fn test_roundtrip_random() {
        for _ in 0..10000 {
            let mut rng = rand::thread_rng();
            let version = rng.gen_range(0..u64::MAX);
            let response = QueryVersionResponse { version };
            let serialized = response.serialize().unwrap();
            let deserialized = QueryVersionResponse::deserialize(&serialized).unwrap();
            assert_eq!(response.version, deserialized.version);
        }
    }
}
