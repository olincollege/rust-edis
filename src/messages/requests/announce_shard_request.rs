use crate::messages::message::{MessagePayload, MessageType};
use anyhow::{Context, Result};
use int_enum::IntEnum;

#[repr(u8)]
#[derive(Debug, Clone, Copy, IntEnum, PartialEq, Eq)]
pub enum ShardType {
    ReadShard = 0,
    WriteShard = 1,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, IntEnum, PartialEq, Eq)]
pub enum AnnounceMessageType {
    NewAnnounce = 0,
    ReAnnounce = 1,
}

pub struct AnnounceShardRequest {
    pub shard_type: ShardType,
    pub message_type: u8,
    pub ip: u128,
    pub port: u16,
}

/// Layout of the AnnounceShardRequest
/// | 1 byte     | 1 byte       | 16 bytes | 2 bytes |
/// | Shard Type | message type |   IP     |   port  |
impl MessagePayload for AnnounceShardRequest {
    fn is_request(&self) -> bool {
        true
    }

    fn get_message_type(&self) -> MessageType {
        MessageType::AnnounceShard
    }
    fn serialize(&self) -> Result<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();

        // Add shard type (1 byte)
        buffer.push(self.shard_type.into());

        // Add message type (1 byte)
        buffer.push(self.message_type);

        // Add IP (16 bytes)
        buffer.extend_from_slice(&self.ip.to_le_bytes());

        // Add port (2 bytes, little-endian)
        buffer.extend_from_slice(&self.port.to_le_bytes());
        Ok(buffer)
    }

    fn deserialize(buffer: &[u8]) -> Result<Self> {
        let mut offset = 0;

        // Read shard type (1 byte)
        let shard_type = ShardType::try_from(buffer[offset]).unwrap();
        offset += 1;

        // Read message type (1 byte)
        let message_type = buffer[offset];
        offset += 1;

        // Read IP (16 bytes)
        let ip = u128::from_le_bytes(
            <[u8; 16]>::try_from(&buffer[offset..offset + 16]).context("failed to get IP bytes")?,
        );
        offset += 16;

        // Read port (2 bytes, little-endian)
        let port = u16::from_le_bytes(
            <[u8; 2]>::try_from(&buffer[offset..offset + 2]).context("failed to get port bytes")?,
        );

        Ok(AnnounceShardRequest {
            shard_type,
            message_type,
            ip,
            port,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_roundtrip_basic() {
        let original = AnnounceShardRequest {
            message_type: AnnounceMessageType::NewAnnounce as u8,
            shard_type: ShardType::WriteShard,
            ip: 1,
            port: 8080,
        };
        let serialized = original.serialize().unwrap();
        let deserialized = AnnounceShardRequest::deserialize(&serialized).unwrap();
        assert_eq!(original.shard_type, deserialized.shard_type);
        assert_eq!(original.ip, deserialized.ip);
        assert_eq!(original.port, deserialized.port);
    }

    #[test]
    fn test_roundtrip_random() {
        for _ in 0..1000 {
            let mut rng = rand::thread_rng();
            let ip: u128 = rng.gen();
            let port: u16 = rng.gen();
            let original = AnnounceShardRequest {
                message_type: AnnounceMessageType::NewAnnounce as u8,
                shard_type: ShardType::WriteShard,
                ip,
                port,
            };
            let serialized = original.serialize().unwrap();
            let deserialized = AnnounceShardRequest::deserialize(&serialized).unwrap();
            assert_eq!(original.shard_type, deserialized.shard_type);
            assert_eq!(original.ip, deserialized.ip);
            assert_eq!(original.port, deserialized.port);
        }
    }
}
