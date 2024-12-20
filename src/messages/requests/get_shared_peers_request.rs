use crate::messages::message::{MessagePayload, MessageType};
use anyhow::{Context, Result};

pub struct GetSharedPeersRequest {
    pub writer_number: u16,
}

/// Layout of the GetSharedPeersRequest
/// | 2 bytes |
/// | Writer Number |
impl MessagePayload for GetSharedPeersRequest {
    fn is_request(&self) -> bool {
        true
    }

    fn get_message_type(&self) -> MessageType {
        MessageType::GetSharedPeers
    }
    fn serialize(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();

        // Add writer number (2 bytes, little-endian)
        buffer.extend_from_slice(&self.writer_number.to_le_bytes());
        Ok(buffer)
    }

    fn deserialize(buffer: &[u8]) -> Result<Self> {
        let writer_number = u16::from_le_bytes(buffer.try_into().context("Invalid buffer size")?);
        Ok(GetSharedPeersRequest { writer_number })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_roundtrip_basic() {
        let response = GetSharedPeersRequest { writer_number: 100 };
        let serialized = response.serialize().unwrap();
        let deserialized = GetSharedPeersRequest::deserialize(&serialized).unwrap();
        assert_eq!(response.writer_number, deserialized.writer_number);
    }

    #[test]
    fn test_roundtrip_random() {
        for _ in 0..1000 {
            let mut rng = rand::thread_rng();
            let writer_number = rng.gen_range(0..u16::MAX);
            let response = GetSharedPeersRequest { writer_number };
            let serialized = response.serialize().unwrap();
            let deserialized = GetSharedPeersRequest::deserialize(&serialized).unwrap();
            assert_eq!(response.writer_number, deserialized.writer_number);
        }
    }
}
