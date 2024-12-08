use crate::messages::message::{MessagePayload, MessageType};
use anyhow::{Context, Result};

pub struct GetVersionRequest {
    pub version: u64,
}

/// Layout of the GetVersionRequest
/// | 8 bytes |
/// | version |
impl MessagePayload for GetVersionRequest {
    fn get_message_type(&self) -> MessageType {
        MessageType::GetVersion
    }

    fn is_request(&self) -> bool {
        true
    }

    fn serialize(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        buffer.extend_from_slice(&self.version.to_le_bytes());
        Ok(buffer)
    }
    fn deserialize(buffer: &[u8]) -> Result<Self> {
        let version = u64::from_le_bytes(
            buffer
                .get(0..8)
                .context("failed to get version")?
                .try_into()?,
        );

        Ok(GetVersionRequest { version })
    }
}
