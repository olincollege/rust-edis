use crate::messages::message::{MessagePayload, MessageType};
use anyhow::Result;

pub struct GetClientShardInfoRequest {}

impl MessagePayload for GetClientShardInfoRequest {
    fn get_message_type(&self) -> MessageType {
        MessageType::GetClientShardInfo
    }

    fn serialize(&self) -> Result<Vec<u8>> {
        Ok(Vec::new())
    }

    fn deserialize(_buffer: &[u8]) -> Result<Self> {
        Ok(GetClientShardInfoRequest {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
        let original = GetClientShardInfoRequest {};
        let serialized = original.serialize().unwrap();
        let _deserialized = GetClientShardInfoRequest::deserialize(&serialized).unwrap();
    }
}
